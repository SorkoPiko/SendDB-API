use serde::{Deserialize, Serialize};
use crate::model::database::default_accurate;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Send {
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatorLevel {
    name: String,
    level_id: i32,
    send_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LevelCreator {
    name: String,
    player_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Rate {
    difficulty: i32,
    points: i32,
    stars: i32,
    timestamp: i64,
    #[serde(default = "default_accurate")]
    accurate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LeaderboardRate {
    difficulty: i32,
    points: i32,
    stars: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Level {
    name: String,
    level_id: i32,
    creator: LevelCreator,
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

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BatchLevel {
    pub level_id: i32,
    send_count: i32,
    accurate: bool,
    platformer: bool,
    length: i32,
    rank: i32,
    trending_score: f64,
    rate: Option<Rate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Creator {
    name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LeaderboardLevel {
    name: String,
    level_id: i32,
    creator: LevelCreator,
    send_count: i32,
    rank: i32,
    platformer: bool,
    rate: Option<LeaderboardRate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrendingLeaderboardLevel {
    name: String,
    level_id: i32,
    creator: LevelCreator,
    send_count: i32,
    rank: i32,
    trending_score: f64,
    platformer: bool,
    rate: Option<LeaderboardRate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchResultCreator {
    pub name: String,
    pub player_id: i32,
}

fn default_relevance() -> f64 {
    0.0
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchLevel {
    pub level_id: i32,
    pub name: String,
    pub send_count: i32,
    pub creator: SearchResultCreator,
    pub rate: Option<LeaderboardRate>,
    pub platformer: bool,
    #[serde(skip_serializing)]
    #[serde(default = "default_relevance")]
    pub relevance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchCreator {
    pub player_id: i32,
    pub name: String,
    pub account_id: i32,
    pub send_count: i32,
    pub level_count: i32,
    pub rank: i32,
    #[serde(skip_serializing)]
    #[serde(default = "default_relevance")]
    pub relevance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchResult {
    Level(SearchLevel),
    Creator(SearchCreator),
}

impl From<Level> for BatchLevel {
    fn from(level: Level) -> Self {
        BatchLevel {
            level_id: level.level_id,
            send_count: level.sends.len() as i32,
            accurate: level.accurate,
            platformer: level.platformer,
            length: level.length,
            rank: level.rank,
            trending_score: level.trending_score,
            rate: level.rate,
        }
    }
}