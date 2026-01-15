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
    pub rate: Option<Rate>,
}