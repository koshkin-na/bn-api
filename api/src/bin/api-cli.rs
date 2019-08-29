#![deny(unused_extern_crates)]
extern crate bigneon_api;
extern crate bigneon_db;
extern crate dotenv;
#[macro_use]
extern crate log;
#[macro_use]
extern crate logging;
extern crate clap;
extern crate diesel;
extern crate uuid;

use bigneon_api::config::Config;
use bigneon_api::db::Database;
use bigneon_api::utils::spotify;
use bigneon_api::utils::ServiceLocator;
use bigneon_db::prelude::*;
use bigneon_db::schema::transfers;
use clap::*;
use diesel::prelude::*;
use dotenv::dotenv;
use log::Level::*;
use std::{thread, time};
use uuid::Uuid;

pub fn main() {
    logging::setup_logger();
    info!("Loading environment");
    dotenv().ok();

    let environment =
        Config::parse_environment().unwrap_or_else(|_| panic!("Environment is invalid."));
    jlog!(Info, &format!("Environment loaded {:?}", environment));

    let config = Config::new(environment);
    let service_locator = ServiceLocator::new(&config).expect("Expected service locator to load");
    let database = Database::from_config(&config);

    let matches = App::new("Big Neon API CLI")
        .author("Big Neon")
        .about("Command Line Interface for running tasks for the Big Neon API")
        .subcommand(
            SubCommand::with_name("sync-purchase-metadata").about("Syncs purchase metadata"),
        )
        .subcommand(
            SubCommand::with_name("sync-spotify-genres")
                .about("Syncs spotify genres across artist records appending any missing genres"),
        )
        .subcommand(
            SubCommand::with_name("regenerate-interaction-records")
                .about("Regenerate interaction records for organization users"),
        )
        .subcommand(
            SubCommand::with_name("backpopulate-temporary-user-data")
                .about("Backpopulate temporary user data"),
        )
        .get_matches();

    match matches.subcommand() {
        ("sync-purchase-metadata", Some(_)) => sync_purchase_metadata(database, service_locator),
        ("sync-spotify-genres", Some(_)) => sync_spotify_genres(config, database),
        ("regenerate-interaction-records", Some(_)) => regenerate_interaction_records(database),
        ("backpopulate-temporary-user-data", Some(_)) => backpopulate_temporary_user_data(database),
        _ => {
            eprintln!("Invalid subcommand '{}'", matches.subcommand().0);
        }
    }
}

fn backpopulate_temporary_user_data(database: Database) {
    info!("Backpopulating temporary user data");
    let connection = database
        .get_connection()
        .expect("Expected connection to establish");
    let connection = connection.get();
    let transfers: Vec<Transfer> = transfers::table
        .filter(
            transfers::transfer_address
                .is_not_null()
                .and(transfers::destination_temporary_user_id.is_null())
                .and(transfers::direct.eq(false)),
        )
        .load(connection)
        .expect("Expected to load transfers");
    for transfer in transfers {
        if let Some(temporary_user) =
            TemporaryUser::find_or_build_from_transfer(&transfer, connection)
                .expect("Expected to create temporary user")
        {
            if let Some(destination_user_id) = transfer.destination_user_id {
                temporary_user
                    .associate_user(destination_user_id, connection)
                    .expect("Expected to associate temporary user with destination user");
            }

            diesel::update(transfers::table.filter(transfers::id.eq(transfer.id)))
                .set(transfers::destination_temporary_user_id.eq(temporary_user.id))
                .execute(connection)
                .unwrap();
        }
    }
}

fn regenerate_interaction_records(database: Database) {
    info!("Regenerating interaction records");
    let connection = database
        .get_connection()
        .expect("Expected connection to establish");
    let connection = connection.get();
    let organizations = Organization::all(connection).expect("Expected to find organizations");

    for organization in organizations {
        let mut page = 0;
        loop {
            let search_fans = organization
                .search_fans(
                    None,
                    None,
                    page,
                    40,
                    FanSortField::Email,
                    SortingDir::Desc,
                    connection,
                )
                .expect("Expected to find organization fans");

            if search_fans.data.len() == 0 {
                break;
            } else {
                for fan in search_fans.data {
                    organization
                        .regenerate_interaction_data(fan.user_id, connection)
                        .expect("Expected to regenerate interaction data");
                }

                thread::sleep(time::Duration::from_secs(2));
            }
            page += 1;
        }
    }
}

fn sync_spotify_genres(config: Config, database: Database) {
    info!("Syncing spotify genres data");
    let connection = database
        .get_connection()
        .expect("Expected connection to establish");
    let connection = connection.get();
    let artists = Artist::find_spotify_linked_artists(connection)
        .expect("Expected to find all artists linked to spotify");

    let artist_ids: Vec<Uuid> = artists.iter().map(|a| a.id).collect();
    let genre_mapping = Genre::find_by_artist_ids(&artist_ids, connection)
        .expect("Expected to find all genres for spotify linked artists");

    if config.spotify_auth_token.is_some() {
        let token = config.spotify_auth_token.clone().unwrap();
        spotify::SINGLETON.set_auth_token(&token);
    }

    let spotify_client = &spotify::SINGLETON;
    let mut i = 0;
    for artist in artists {
        i += 1;
        if let Some(spotify_id) = artist.spotify_id.clone() {
            let result = spotify_client.read_artist(&spotify_id);
            match result {
                Ok(spotify_artist_result) => match spotify_artist_result {
                    Some(spotify_artist) => {
                        let mut genres = spotify_artist.genres.clone().unwrap_or(Vec::new());
                        let mut artist_genres = genre_mapping
                            .get(&artist.id)
                            .map(|m| m.into_iter().map(|g| g.name.clone()).collect())
                            .unwrap_or(Vec::new());
                        genres.append(&mut artist_genres);

                        let result = artist.set_genres(&genres, None, connection);

                        match result {
                            Ok(_) => {
                                let mut exit_outer_loop = false;
                                for event in artist
                                    .events(connection)
                                    .expect("Expected to find artist events")
                                {
                                    if let Err(error) = event.update_genres(None, connection) {
                                        error!("Error: {}", error);
                                        exit_outer_loop = true;
                                        break;
                                    };
                                }

                                if exit_outer_loop {
                                    break;
                                }

                                if i % 5 == 0 {
                                    thread::sleep(time::Duration::from_secs(1))
                                }
                            }
                            Err(error) => {
                                error!("Error: {}", error);
                                break;
                            }
                        }
                    }
                    None => error!("Error no spotify artist returned"),
                },
                Err(error) => {
                    error!("Error: {}", error);
                    break;
                }
            }
        }
    }
}

fn sync_purchase_metadata(database: Database, service_locator: ServiceLocator) {
    info!("Syncing purchase metadata");
    let connection = database
        .get_connection()
        .expect("Expected connection to establish");
    let mut i = 0;
    let mut exit = false;

    loop {
        if exit {
            break;
        }

        let payments = Payment::find_all_with_orders_paginated_by_provider(
            PaymentProviders::Stripe,
            i,
            50,
            connection.get(),
        )
        .expect("Expected to find all payments with orders");
        i += 1;

        if payments.len() == 0 {
            break;
        }

        for (payment, order) in payments {
            let organizations = order.organizations(connection.get()).unwrap();

            let stripe = service_locator
                .create_payment_processor(PaymentProviders::Stripe, &organizations[0])
                .expect("Expected Stripe processor");

            if let Some(external_reference) = payment.external_reference {
                let purchase_metadata = order
                    .purchase_metadata(connection.get())
                    .expect("Expected purchase metadata for order");
                let result = stripe.update_metadata(&external_reference, purchase_metadata);

                match result {
                    // Sleep to avoid hammering Stripe API
                    Ok(_) => thread::sleep(time::Duration::from_secs(1)),
                    Err(error) => {
                        error!("Error: {}", error);
                        exit = true;
                        break;
                    }
                }
            }
        }
    }
}
