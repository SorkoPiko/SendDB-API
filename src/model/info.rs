use serde::{Deserialize, Serialize};
use crate::model::database::default_accurate;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Send {
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Rate {
    pub difficulty: i32,
    pub points: i32,
    pub stars: i32,
    pub timestamp: i64,
    #[serde(default = "default_accurate")]
    pub accurate: bool,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Level {
    pub level_id: i32,
    pub sends: Vec<Send>,
    pub accurate: bool,
    pub platformer: bool,
    pub length: i32,
    pub rank: i32,
    pub rate_rank: i32,
    pub gamemode_rank: i32,
    pub joined_rank: i32,
    pub trending_score: f64,
    pub rate: Option<Rate>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BatchLevel {
    pub level_id: i32,
    pub send_count: i32,
    pub accurate: bool,
    pub platformer: bool,
    pub length: i32,
    pub rank: i32,
    pub trending_score: f64,
    pub rate: Option<Rate>,
}