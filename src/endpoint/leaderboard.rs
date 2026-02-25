use std::sync::Arc;
use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::endpoint::common;
use crate::model::database::Database;
use crate::model::info::{LeaderboardCreator, LeaderboardLevel, TrendingLeaderboardLevel};

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
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TrendingLeaderboardQuery {
    pub limit: i32,
    pub offset: i32,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatorLeaderboardQuery {
    pub limit: i32,
    pub offset: i32,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LeaderboardResponse {
    pub total: i32,
    pub levels: Vec<LeaderboardLevel>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TrendingLeaderboardResponse {
    pub total: i32,
    pub levels: Vec<TrendingLeaderboardLevel>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreatorLeaderboardResponse {
    pub total: i32,
    pub creators: Vec<LeaderboardCreator>,
}

#[utoipa::path(summary = "Get leaderboard", responses(
    (status = OK, description = "Get leaderboard", body = LeaderboardResponse)
))]
#[post("")]
pub async fn leaderboard(
    database: web::Data<Arc<Mutex<dyn Database>>>,
    query: web::Json<LeaderboardQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    if query.limit <= 0 {
        return Ok(HttpResponse::Ok().json(LeaderboardResponse { total: 0, levels: vec![] }));
    } else if query.limit > 50 {
        return Err(common::bad_request("Too many levels requested"));
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

#[utoipa::path(summary = "Get trending leaderboard", responses(
    (status = OK, description = "Get trending leaderboard", body = TrendingLeaderboardResponse)
))]
#[post("/trending")]
pub async fn trending_leaderboard(
    database: web::Data<Arc<Mutex<dyn Database>>>,
    query: web::Json<TrendingLeaderboardQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    if query.limit <= 0 {
        return Ok(HttpResponse::Ok().json(TrendingLeaderboardResponse { total: 0, levels: vec![] }));
    } else if query.limit > 50 {
        return Err(common::bad_request("Too many levels requested"));
    } else if query.offset < 0 {
        return Err(common::bad_request("Invalid offset"));
    }

    let response = {
        let db = database.lock().await;
        db.get_trending_levels(&query).await
            .map_err(|e| {
                log::error!("{:?}", e);
                common::internal_server_error("Database error")
            })?
    };

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(summary = "Get creator leaderboard", responses(
    (status = OK, description = "Get creator leaderboard", body = CreatorLeaderboardResponse)
))]
#[post("/creators")]
pub async fn creator_leaderboard(
    database: web::Data<Arc<Mutex<dyn Database>>>,
    query: web::Json<CreatorLeaderboardQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    if query.limit <= 0 {
        return Ok(HttpResponse::Ok().json(CreatorLeaderboardResponse { total: 0, creators: vec![] }));
    } else if query.limit > 50 {
        return Err(common::bad_request("Too many creators requested"));
    } else if query.offset < 0 {
        return Err(common::bad_request("Invalid offset"));
    }

    let response = {
        let db = database.lock().await;
        db.get_creators(&query).await
            .map_err(|e| {
                log::error!("{:?}", e);
                common::internal_server_error("Database error")
            })?
    };

    Ok(HttpResponse::Ok().json(response))
}