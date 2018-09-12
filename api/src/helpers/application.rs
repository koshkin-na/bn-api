use actix_web::{http::StatusCode, HttpResponse};
use errors::*;
use validator::ValidationErrors;

pub fn unauthorized() -> Result<HttpResponse, BigNeonError> {
    unauthorized_with_message("Unauthorized")
}

pub fn unauthorized_with_message(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Unauthorized: {}", message);
    Ok(HttpResponse::Unauthorized().json(json!({"error": message.to_string()})))
}

pub fn forbidden(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Forbidden: {}", message);
    Ok(HttpResponse::Forbidden().json(json!({"error":message.to_string()})))
}

pub fn internal_server_error(message: &str) -> Result<HttpResponse, BigNeonError> {
    error!("Internal Server Error: {}", message);
    Ok(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
        .into_builder()
        .json(json!({"error": message.to_string()})))
}

pub fn validation_error_response(errors: ValidationErrors) -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::BadRequest()
        .json(json!({"error": "Validation error".to_string(), "fields": errors.inner()})))
}