use serde::{Deserialize, Serialize};
use crate::model::database::default_accurate;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Send {
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatorLevel {
    level_id: i32,
    send_count: i32,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Rate {
    difficulty: i32,
    points: i32,
    stars: i32,
    timestamp: i64,
    #[serde(default = "default_accurate")]
    accurate: bool,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Level {
    level_id: i32,
    sends: Vec<Send>,
    accurate: bool,
    platformer: bool,
    length: i32,
    rank: i32,
    trending_score: f64,
    rate_rank: i32,
    gamemode_rank: i32,
    joined_rank: i32,
    trending_rank: i32,
    rate: Option<Rate>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BatchLevel {
    level_id: i32,
    send_count: i32,
    accurate: bool,
    platformer: bool,
    length: i32,
    rank: i32,
    trending_score: f64,
    rate: Option<Rate>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Creator {
    player_id: i32,
    account_id: i32,
    levels: Vec<CreatorLevel>,
    send_count: i32,
    recent_sends: i32,
    send_count_stddev: f64,
    trending_score: f64,
    trending_level_count: i32,
    latest_send: i64,
    rank: i32,
    trending_rank: i32,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LeaderboardLevel {
    level_id: i32,
    send_count: i32,
    rank: i32,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrendingLeaderboardLevel {
    level_id: i32,
    send_count: i32,
    rank: i32,
    trending_score: f64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LeaderboardCreator {
    name: String,
    player_id: i32,
    account_id: i32,
    level_count: i32,
    send_count: i32,
    trending_score: f64,
    rank: i32,
    trending_rank: i32,
}