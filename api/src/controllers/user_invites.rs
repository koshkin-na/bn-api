use crate::auth::user::User as AuthUser;
use crate::communications::mailers;
use crate::database::Connection;
use crate::errors::*;
use crate::helpers::application;
use crate::server::AppState;
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use db::models::concerns::users::password_resetable::PasswordResetable;
use db::models::User;
use db::utils::errors::Optional;
use std::str;

#[derive(Deserialize)]
pub struct UserInviteRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
}

pub async fn create(
    (state, connection, parameters, auth_user): (Data<AppState>, Connection, Json<UserInviteRequest>, AuthUser),
) -> Result<HttpResponse, ApiError> {
    let connection = connection.get();
    let email = parameters.email.trim().to_lowercase();
    // User already exists, so no need to invite them
    if let Some(_) = User::find_by_email(&email, true, connection).optional()? {
        return application::created(json!({}));
    }

    let new_user = User::new_stub(
        parameters.first_name.clone(),
        parameters.last_name.clone(),
        Some(parameters.email.clone()),
        None,
    );

    let user = new_user.commit(Some(auth_user.id()), connection)?;
    let user = user.create_password_reset_token(connection)?;

    mailers::user::invite_user_email(&state.config, &user, connection)?;

    application::created(json!({}))
}
