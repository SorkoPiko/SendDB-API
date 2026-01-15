pub fn not_found(what: &str) -> actix_web::Error {
    actix_web::error::ErrorNotFound(format!("{} not found", what))
}

