use serde_json::json;

pub fn not_found(what: &str) -> actix_web::Error {
    actix_web::error::ErrorNotFound(generic_error(format!("{} not found", what)))
}

pub fn bad_request(message: &str) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(generic_error(message.to_owned()))
}

pub fn internal_server_error(message: &str) -> actix_web::Error {
    actix_web::error::ErrorInternalServerError(generic_error(message.to_owned()))
}

pub fn unauthorized(message: &str) -> actix_web::Error {
    actix_web::error::ErrorUnauthorized(generic_error(message.to_owned()))
}

pub fn generic_error(message: String) -> serde_json::Value {
    json!({
        "error": true,
        "message": message
    })
}