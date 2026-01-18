use std::sync::Arc;
use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::endpoint::common;
use crate::model::database::Database;
use crate::model::info::{BatchLevel, Level};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
struct BatchLevelRequest {
    pub level_ids: Vec<i64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
struct BatchLevelResponse {
    pub levels: Vec<BatchLevel>,
}

#[utoipa::path(summary = "Get batch level data", responses(
    (status = OK, description = "Get batch level data", body = BatchLevelResponse)
))]
#[post("/batch")]
pub async fn batch_level(
    database: web::Data<Arc<Mutex<dyn Database>>>,
    req: actix_web::HttpRequest,
    batch: web::Json<BatchLevelRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    if batch.level_ids.is_empty() {
        return Ok(HttpResponse::Ok().json(BatchLevelResponse { levels: vec![] }));
    } else if batch.level_ids.len() > 50 {
        return Err(common::bad_request("Too many level IDs"));
    }

    let levels = {
        let db = database.lock().await;
        db.get_levels_by_ids(&batch.level_ids).await
            .map_err(|e| {
                log::error!("{:?}", e);
                common::internal_server_error("Database error")
            })?
    };

    Ok(HttpResponse::Ok().json(BatchLevelResponse { levels }))
}

#[utoipa::path(summary = "Get level data", responses(
    (status = OK, description = "Get level data", body = Level),
    (status = NOT_FOUND, description = "Level not found")
))]
#[get("/{level_id}")]
pub async fn get_level(
    database: web::Data<Arc<Mutex<dyn Database>>>,
    req: actix_web::HttpRequest,
    level_id: web::Path<i64>,
) -> Result<HttpResponse, actix_web::Error> {
    if (*level_id) < 0 {
        return Err(common::not_found("Level"));
    }

    let level = {
        let db = database.lock().await;
        db.get_level_by_id(*level_id).await
            .map_err(|e| {
                log::error!("{:?}", e);
                common::internal_server_error("Database error")
            })?
    };

    match level {
        Some(level) => Ok(HttpResponse::Ok().json(level)),
        None => Err(common::not_found("Level")),
    }
}