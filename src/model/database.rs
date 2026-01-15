use serde::{Deserialize, Serialize};
use crate::model::info::Level;

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

#[async_trait::async_trait]
pub trait Database: Send + Sync {
    async fn get_levels_by_ids(&self, level_ids: &[i64]) -> anyhow::Result<Vec<Level>>;
    async fn get_level_by_id(&self, level_id: i64) -> anyhow::Result<Option<Level>>; }