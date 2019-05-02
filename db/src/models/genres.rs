use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::sql_types::{Array, Text, Uuid as dUuid};
use schema::{artist_genres, genres};
use std::collections::HashMap;
use utils::errors::*;
use uuid::Uuid;

#[derive(Identifiable, Clone, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "genres"]
pub struct Genre {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Genre {
    pub fn find_by_artist_ids(
        artist_ids: &Vec<Uuid>,
        conn: &PgConnection,
    ) -> Result<HashMap<Uuid, Vec<Genre>>, DatabaseError> {
        let artist_genres: Vec<(Uuid, Genre)> = artist_genres::table
            .inner_join(genres::table.on(genres::id.eq(artist_genres::genre_id)))
            .filter(artist_genres::artist_id.eq_any(artist_ids))
            .select((artist_genres::artist_id, genres::all_columns))
            .then_order_by(genres::name.asc())
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all genres")?;

        let mut artist_genre_mapping: HashMap<Uuid, Vec<Genre>> = HashMap::new();
        for (artist_id, genre) in artist_genres {
            artist_genre_mapping
                .entry(artist_id)
                .or_insert(Vec::new())
                .push(genre.clone());
        }
        Ok(artist_genre_mapping)
    }

    pub fn find_or_create(
        names: &Vec<String>,
        conn: &PgConnection,
    ) -> Result<Vec<Uuid>, DatabaseError> {
        #[derive(QueryableByName)]
        struct R {
            #[sql_type = "dUuid"]
            id: Uuid,
        };

        let formatted_genres = Genre::format_names(names);
        let query = r#"
            INSERT INTO genres (name)
            SELECT gn as name
            FROM unnest($1) gn
            LEFT JOIN genres g ON g.name = gn
            WHERE g.id IS NULL;
        "#;
        diesel::sql_query(query)
            .bind::<Array<Text>, _>(formatted_genres.clone())
            .execute(conn)
            .to_db_error(ErrorCode::QueryError, "Could not set genres")?;

        let query = r#"
            SELECT id FROM genres WHERE name = ANY($1);
        "#;
        Ok(diesel::sql_query(query)
            .bind::<Array<Text>, _>(formatted_genres)
            .get_results::<R>(conn)
            .to_db_error(ErrorCode::QueryError, "Could not get genres")?
            .into_iter()
            .map(|g| g.id)
            .collect())
    }

    pub fn format_name(name: &String) -> String {
        name.to_lowercase().trim().replace(" ", "-")
    }

    pub fn format_names(names: &Vec<String>) -> Vec<String> {
        let mut formatted_names: Vec<String> =
            names.into_iter().map(|n| Genre::format_name(n)).collect();
        formatted_names.sort();
        formatted_names.dedup();

        formatted_names
    }

    pub fn all(conn: &PgConnection) -> Result<Vec<Genre>, DatabaseError> {
        genres::table
            .then_order_by(genres::name.asc())
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all genres")
    }
}
