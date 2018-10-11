use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::Organization;
use schema::{artists, organization_users, organizations};
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;
use validator::Validate;
use validators;

#[derive(Associations, Deserialize, Identifiable, Queryable, Serialize, Debug, PartialEq, Clone)]
pub struct Artist {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub is_private: bool,
    pub name: String,
    pub bio: String,
    pub image_url: Option<String>,
    pub thumb_image_url: Option<String>,
    pub website_url: Option<String>,
    pub youtube_video_urls: Vec<String>,
    pub facebook_username: Option<String>,
    pub instagram_username: Option<String>,
    pub snapchat_username: Option<String>,
    pub soundcloud_username: Option<String>,
    pub bandcamp_username: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Default, Deserialize, Validate)]
#[table_name = "artists"]
pub struct NewArtist {
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub bio: String,
    #[validate(url)]
    pub image_url: Option<String>,
    #[validate(url)]
    pub thumb_image_url: Option<String>,
    #[validate(url)]
    pub website_url: Option<String>,
    #[validate(custom = "validators::validate_urls")]
    pub youtube_video_urls: Option<Vec<String>>,
    pub facebook_username: Option<String>,
    pub instagram_username: Option<String>,
    pub snapchat_username: Option<String>,
    pub soundcloud_username: Option<String>,
    pub bandcamp_username: Option<String>,
}

impl NewArtist {
    pub fn commit(&self, conn: &PgConnection) -> Result<Artist, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new artist",
            diesel::insert_into(artists::table)
                .values(self)
                .get_result(conn),
        )
    }
}

impl Artist {
    pub fn create(
        name: &str,
        organization_id: Option<Uuid>,
        bio: &str,
        website_url: &str,
    ) -> NewArtist {
        NewArtist {
            organization_id,
            name: String::from(name),
            bio: String::from(bio),
            website_url: Some(String::from(website_url)),
            ..Default::default()
        }
    }

    pub fn all(user_id: Option<Uuid>, conn: &PgConnection) -> Result<Vec<Artist>, DatabaseError> {
        let query = match user_id {
            Some(u) => artists::table
                .left_join(
                    organization_users::table.on(artists::organization_id
                        .eq(organization_users::organization_id.nullable())
                        .and(organization_users::user_id.eq(u))),
                ).left_join(
                    organizations::table
                        .on(artists::organization_id.eq(organizations::id.nullable())),
                ).filter(
                    organization_users::user_id
                        .eq(u)
                        .or(organizations::owner_user_id.eq(u))
                        .or(artists::is_private.eq(false)),
                ).order_by(artists::name)
                .select(artists::all_columns)
                .load(conn),
            None => artists::table
                .filter(artists::is_private.eq(false))
                .order_by(artists::name)
                .select(artists::all_columns)
                .load(conn),
        };

        query.to_db_error(ErrorCode::QueryError, "Unable to load all artists")
    }

    pub fn find(id: &Uuid, conn: &PgConnection) -> Result<Artist, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading artist",
            artists::table.find(id).first::<Artist>(conn),
        )
    }

    pub fn find_for_organization(
        user_id: Option<Uuid>,
        organization_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<Artist>, DatabaseError> {
        let query = match user_id {
            Some(u) => artists::table
                .left_join(
                    organization_users::table.on(artists::organization_id
                        .eq(organization_users::organization_id.nullable())
                        .and(organization_users::user_id.eq(u))),
                ).left_join(
                    organizations::table
                        .on(artists::organization_id.eq(organizations::id.nullable())),
                ).filter(
                    organization_users::user_id
                        .eq(u)
                        .or(organizations::owner_user_id.eq(u))
                        .or(artists::is_private.eq(false)),
                ).filter(artists::organization_id.eq(organization_id))
                .order_by(artists::name)
                .select(artists::all_columns)
                .load(conn),
            None => artists::table
                .filter(artists::is_private.eq(false))
                .filter(artists::organization_id.eq(organization_id))
                .order_by(artists::name)
                .select(artists::all_columns)
                .load(conn),
        };

        query.to_db_error(ErrorCode::QueryError, "Unable to load all artists")
    }

    pub fn update(
        &self,
        attributes: &ArtistEditableAttributes,
        conn: &PgConnection,
    ) -> Result<Artist, DatabaseError> {
        let query = diesel::update(self).set((attributes, artists::updated_at.eq(dsl::now)));

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Error updating artist",
            query.get_result(conn),
        )
    }

    pub fn destroy(&self, conn: &PgConnection) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::DeleteError,
            "Failed to destroy artist record",
            diesel::delete(self).execute(conn),
        )
    }

    pub fn organization(&self, conn: &PgConnection) -> Result<Option<Organization>, DatabaseError> {
        match self.organization_id {
            Some(organization_id) => Ok(Some(Organization::find(organization_id, conn)?)),
            None => Ok(None),
        }
    }

    pub fn set_privacy(
        &self,
        is_private: bool,
        conn: &PgConnection,
    ) -> Result<Artist, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update is_private for artist",
            diesel::update(self)
                .set((
                    artists::is_private.eq(is_private),
                    artists::updated_at.eq(dsl::now),
                )).get_result(conn),
        )
    }
}

#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "artists"]
pub struct ArtistEditableAttributes {
    pub name: Option<String>,
    pub bio: Option<String>,
    #[validate(url)]
    pub image_url: Option<String>,
    #[validate(url)]
    pub thumb_image_url: Option<String>,
    #[validate(url)]
    pub website_url: Option<String>,
    #[validate(custom = "validators::validate_urls")]
    pub youtube_video_urls: Option<Vec<String>>,
    pub facebook_username: Option<String>,
    pub instagram_username: Option<String>,
    pub snapchat_username: Option<String>,
    pub soundcloud_username: Option<String>,
    pub bandcamp_username: Option<String>,
}

impl ArtistEditableAttributes {
    pub fn new() -> ArtistEditableAttributes {
        Default::default()
    }
}
