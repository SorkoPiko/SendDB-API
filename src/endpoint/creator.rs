use std::sync::Arc;
use actix_web::{get, web, HttpResponse};
use tokio::sync::Mutex;
use crate::endpoint::common;
use crate::model::database::Database;
use crate::model::info::Creator;

#[utoipa::path(summary = "Get creator data", responses(
    (status = OK, description = "Get creator data", body = Creator),
    (status = NOT_FOUND, description = "Creator not found")
))]
#[get("/{player_id}")]
pub async fn get_creator(
    database: web::Data<Arc<Mutex<dyn Database>>>,
    req: actix_web::HttpRequest,
    player_id: web::Path<i64>,
) -> Result<HttpResponse, actix_web::Error> {
    if (*player_id) < 0 {
        return Err(common::not_found("Creator"));
    }

    let creator = {
        let db = database.lock().await;
        db.get_creator_by_id(*player_id).await
            .map_err(|e| {
                log::error!("{:?}", e);
                common::internal_server_error("Database error")
            })?
    };

    match creator {
        Some(creator) => Ok(HttpResponse::Ok().json(creator)),
        None => Err(common::not_found("Creator")),
    }
}