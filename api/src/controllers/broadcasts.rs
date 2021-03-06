use crate::auth::user::User;
use crate::database::Connection;
use crate::errors::ApiError;
use crate::extractors::Json;
use crate::models::{PathParameters, WebPayload};
use actix_web::{
    web::{Path, Query},
    HttpResponse,
};
use chrono::NaiveDateTime;
use db::models::enums::{BroadcastAudience, BroadcastChannel, BroadcastType};
use db::models::scopes::Scopes;
use db::models::{Broadcast, BroadcastEditableAttributes, Organization, PagingParameters};
use reqwest::StatusCode;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct NewBroadcastData {
    pub notification_type: BroadcastType,
    pub name: Option<String>,
    //None is now
    pub send_at: Option<NaiveDateTime>,
    pub message: Option<String>,
    pub channel: Option<BroadcastChannel>,
    pub audience: Option<BroadcastAudience>,
    pub subject: Option<String>,
    pub preview_email: Option<String>,
}

pub async fn create(
    (conn, path, json, user): (Connection, Path<PathParameters>, Json<NewBroadcastData>, User),
) -> Result<HttpResponse, ApiError> {
    let connection = conn.get();
    let organization = Organization::find_for_event(path.id, connection)?;
    let channel = json.channel.unwrap_or(BroadcastChannel::PushNotification);

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;

    let broadcast = Broadcast::create(
        path.id,
        json.notification_type.clone(),
        channel,
        json.name.clone().unwrap_or(json.notification_type.to_string()),
        json.message.clone(),
        json.send_at,
        None,
        json.subject.clone(),
        json.audience.clone().unwrap_or(BroadcastAudience::PeopleAtTheEvent),
        json.preview_email.clone(),
    )
    .commit(connection)?;
    Ok(HttpResponse::Created().json(json!(broadcast)))
}

pub async fn index(
    (conn, path, query, user): (Connection, Path<PathParameters>, Query<PagingParameters>, User),
) -> Result<WebPayload<Broadcast>, ApiError> {
    let connection = conn.get();
    let organization = Organization::find_for_event(path.id, connection)?;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;

    let push_notifications = Broadcast::find_by_event_id(
        path.id,
        match query.get_tag_as_str("channel") {
            Some(s) => Some(s.parse()?),
            None => None,
        },
        match query.get_tag_as_str("broadcast_type") {
            Some(s) => Some(s.parse()?),
            None => None,
        },
        query.page() as i64,
        query.limit() as i64,
        connection,
    )?;

    Ok(WebPayload::new(StatusCode::OK, push_notifications))
}

pub async fn show((conn, path, user): (Connection, Path<PathParameters>, User)) -> Result<HttpResponse, ApiError> {
    let connection = conn.get();
    let push_notification = Broadcast::find(path.id, connection)?;
    let organization = Organization::find_for_event(push_notification.event_id, connection)?;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;

    Ok(HttpResponse::Ok().json(push_notification))
}

pub async fn update(
    (conn, path, json, user): (
        Connection,
        Path<PathParameters>,
        Json<BroadcastEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, ApiError> {
    let connection = conn.get();
    let broadcast = Broadcast::find(path.id, connection)?;
    let organization = Organization::find_for_event(broadcast.event_id, connection)?;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;
    let mut broadcast_attributes = json.into_inner();
    //Never allow an API call to update the status of a broadcast, it must either be set in the model or be cancelled specifically
    broadcast_attributes.status = None;
    let broadcast = broadcast.update(broadcast_attributes, connection)?;
    Ok(HttpResponse::Ok().json(broadcast))
}

pub async fn delete((conn, path, user): (Connection, Path<PathParameters>, User)) -> Result<HttpResponse, ApiError> {
    let connection = conn.get();
    let broadcast = Broadcast::find(path.id, connection)?;
    let organization = Organization::find_for_event(broadcast.event_id, connection)?;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;

    let broadcast = broadcast.cancel(connection)?;
    Ok(HttpResponse::Ok().json(broadcast))
}

pub async fn tracking_count(
    (conn, path, _user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, ApiError> {
    let connection = conn.get();
    Broadcast::increment_open_count(path.id.clone(), connection)?;
    let broadcast = Broadcast::find(path.id, connection)?;
    Ok(HttpResponse::Ok().json(json!({"event_id": broadcast.event_id})))
}
#[derive(Serialize, Deserialize)]
pub struct BroadcastPushNotificationAction {
    pub event_id: Uuid,
}
