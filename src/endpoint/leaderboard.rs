use std::sync::Arc;
use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::endpoint::common;
use crate::model::database::Database;
use crate::model::info::LeaderboardLevel;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub enum RateFilter {
    Rated,
    Unrated,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub enum GamemodeFilter {
    Classic,
    Platformer,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LeaderboardQuery {
    pub limit: i32,
    pub offset: i32,
    pub rate_filter: Option<RateFilter>,
    pub gamemode_filter: Option<GamemodeFilter>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LeaderboardResponse {
    pub total: i32,
    pub levels: Vec<LeaderboardLevel>,
}

#[utoipa::path(summary = "Get leaderboard", responses(
    (status = OK, description = "Get leaderboard", body = LeaderboardResponse)
))]
#[post("")]
pub async fn leaderboard(
    database: web::Data<Arc<Mutex<dyn Database>>>,
    req: actix_web::HttpRequest,
    query: web::Json<LeaderboardQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    if query.limit <= 0 {
        return Ok(HttpResponse::Ok().json(LeaderboardResponse { total: 0, levels: vec![] }));
    } else if query.limit > 50 {
        return Err(common::bad_request("Too many levels"));
    } else if query.offset < 0 {
        return Err(common::bad_request("Invalid offset"));
    }

    let response = {
        let db = database.lock().await;
        db.get_leaderboard_levels(&query).await
            .map_err(|e| {
                log::error!("{:?}", e);
                common::internal_server_error("Database error")
            })?
    };

    Ok(HttpResponse::Ok().json(response))
}