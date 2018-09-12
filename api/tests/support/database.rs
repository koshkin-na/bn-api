use bigneon_api::config::{Config, Environment};
use bigneon_db::dev::builders::*;
use diesel::Connection;
use diesel::PgConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct TestDatabase {
    pub connection: Arc<PgConnection>,
}

#[allow(dead_code)]
impl TestDatabase {
    pub fn new() -> TestDatabase {
        let config = Config::new(Environment::Test);

        let connection = PgConnection::establish(&config.database_url).expect(&format!(
            "Connection to {} could not be established.",
            config.database_url
        ));

        connection.begin_test_transaction().unwrap();

        TestDatabase {
            connection: Arc::new(connection),
        }
    }

    pub fn create_artist(&self) -> ArtistBuilder {
        ArtistBuilder::new(&self.connection)
    }

    pub fn create_event(&self) -> EventBuilder {
        EventBuilder::new(&self.connection)
    }

    pub fn create_organization(&self) -> OrganizationBuilder {
        OrganizationBuilder::new(&self.connection)
    }

    pub fn create_organization_invite(&self) -> OrgInviteBuilder {
        OrgInviteBuilder::new(&self.connection)
    }

    pub fn create_region(&self) -> RegionBuilder {
        RegionBuilder::new(&self.connection)
    }

    pub fn create_user(&self) -> UserBuilder {
        UserBuilder::new(&self.connection)
    }

    pub fn create_venue(&self) -> VenueBuilder {
        VenueBuilder::new(&self.connection)
    }

    pub fn create_fee_schedule(&self) -> FeeScheduleBuilder {
        FeeScheduleBuilder::new(&self.connection)
    }
}