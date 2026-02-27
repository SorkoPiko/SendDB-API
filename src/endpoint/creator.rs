use actix_web::{get, web, HttpResponse};
use crate::AppState;
use crate::endpoint::common;
use crate::model::info::Creator;

#[utoipa::path(summary = "Get creator data", responses(
    (status = OK, description = "Get creator data", body = Creator),
    (status = NOT_FOUND, description = "Creator not found")
))]
#[get("/{player_id}")]
pub async fn get_creator(
    app_state: web::Data<AppState>,
    player_id: web::Path<i64>,
) -> Result<HttpResponse, actix_web::Error> {
    if (*player_id) < 0 {
        return Err(common::not_found("Creator"));
    }

    let creator = app_state.creator_cache
        .try_get_with(*player_id, async {
            let db = app_state.database.lock().await;
            db.get_creator_by_id(*player_id).await
                .map_err(|e| {
                    log::error!("{:?}", e);
                    e
                })
        })
        .await
        .map_err(|_| common::internal_server_error("Database error"))?;

    match creator {
        Some(creator) => Ok(HttpResponse::Ok().json(creator)),
        None => Err(common::not_found("Creator")),
    }
}