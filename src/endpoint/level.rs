use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::endpoint::common;
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
    app_state: web::Data<AppState>,
    batch: web::Json<BatchLevelRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    if batch.level_ids.is_empty() {
        return Ok(HttpResponse::Ok().json(BatchLevelResponse { levels: vec![] }));
    } else if batch.level_ids.len() > 50 {
        return Err(common::bad_request("Too many level IDs"));
    }

    let mut levels = Vec::new();
    let mut missing_ids = Vec::new();

    for &level_id in &batch.level_ids {
        if level_id < 0 {
            continue;
        }

        if let Some(cached_level) = app_state.batch_level_cache.get(&level_id).await {
            if let Some(level) = cached_level {
                levels.push(level);
            }
        } else if let Some(cached_level) = app_state.level_cache.get(&level_id).await {
            if let Some(level) = cached_level {
                levels.push(BatchLevel::from(level));
            }
        } else {
            missing_ids.push(level_id);
        }
    }

    if !missing_ids.is_empty() {
        let db_levels = {
            let db = app_state.database.lock().await;
            db.get_levels_by_ids(&missing_ids).await
                .map_err(|e| {
                    log::error!("{:?}", e);
                    common::internal_server_error("Database error")
                })?
        };

        for level in db_levels {
            app_state.batch_level_cache.insert(level.level_id as i64, Some(level.clone())).await;
            levels.push(level);
        }

        let found_ids: std::collections::HashSet<_> = levels.iter().map(|l| l.level_id as i64).collect();
        for &missing_id in &missing_ids {
            if !found_ids.contains(&missing_id) {
                app_state.batch_level_cache.insert(missing_id, None).await;
            }
        }
    }

    Ok(HttpResponse::Ok().json(BatchLevelResponse { levels }))
}

#[utoipa::path(summary = "Get level data", responses(
    (status = OK, description = "Get level data", body = Level),
    (status = NOT_FOUND, description = "Level not found")
))]
#[get("/{level_id}")]
pub async fn get_level(
    app_state: web::Data<AppState>,
    level_id: web::Path<i64>,
) -> Result<HttpResponse, actix_web::Error> {
    if (*level_id) < 0 {
        return Err(common::not_found("Level"));
    }

    let level = app_state.level_cache
        .try_get_with(*level_id, async {
            let db = app_state.database.lock().await;
            db.get_level_by_id(*level_id).await
                .map_err(|e| {
                    log::error!("{:?}", e);
                    e
                })
        })
        .await
        .map_err(|_| common::internal_server_error("Database error"))?;

    match level {
        Some(level) => Ok(HttpResponse::Ok().json(level)),
        None => Err(common::not_found("Level")),
    }
}