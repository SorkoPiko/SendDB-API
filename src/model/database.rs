use serde::{Deserialize, Serialize};
use crate::model::info::{BatchLevel, Level};

pub fn default_accurate() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoItem {
    #[serde(rename = "_id")]
    level_id: i32,
    creator: i32,
    name: String,
    length: i32,
    platformer: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RateItem {
    #[serde(rename = "_id")]
    level_id: i32,
    difficulty: i32,
    points: i32,
    stars: i32,
    #[serde(with = "bson::serde_helpers::datetime::FromI64")]
    timestamp: i64,
    #[serde(default = "default_accurate")]
    accurate: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendItem {
    #[serde(rename = "levelID")]
    level_id: i32,
    #[serde(with = "bson::serde_helpers::datetime::FromI64")]
    timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LevelStatItem {
    #[serde(rename = "_id")]
    level_id: i32,
    send_count: i32,
    #[serde(with = "bson::serde_helpers::datetime::FromI64")]
    latest_send: i64,
    trending_score: f64,
    recent_sends: i32,
    last_updated: f64,
    rank: i32,
    rate_rank: i32,
    gamemode_rank: i32,
    joined_rank: i32,
}

#[async_trait::async_trait]
pub trait Database: Send + Sync {
    async fn get_levels_by_ids(&self, level_ids: &[i64]) -> anyhow::Result<Vec<BatchLevel>>;
    async fn get_level_by_id(&self, level_id: i64) -> anyhow::Result<Option<Level>>;
}