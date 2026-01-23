use std::time::Duration;
use anyhow::Context;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use crate::endpoint::leaderboard::{GamemodeFilter, LeaderboardQuery, LeaderboardResponse, RateFilter};
use crate::model::database::{CreatorStatItem, Database, InfoItem, LevelStatItem, RateItem, SendItem};
use crate::model::info::{BatchLevel, Creator, LeaderboardLevel, Level};

pub struct MongoDatabase {
    client: mongodb::Client,

    info: mongodb::Collection<InfoItem>,
    rates: mongodb::Collection<RateItem>,
    sends: mongodb::Collection<SendItem>,
    level_stats: mongodb::Collection<LevelStatItem>,
    creator_stats: mongodb::Collection<CreatorStatItem>,

    oldest_level: i32,
}

impl MongoDatabase {
    pub async fn new(database_url: &str, oldest_level: i32) -> anyhow::Result<Self> {
        let client = mongodb::Client::with_uri_str(database_url)
            .await
            .context("Failed to initialize MongoDB client")?;

        let db = client.database("data");

        let info = db.collection("info");
        let rates = db.collection("rates");
        let sends = db.collection("sends");
        let level_stats = db.collection("level_stats");
        let creator_stats = db.collection("creator_stats");

        Ok(Self {
            client,
            info,
            rates,
            sends,
            level_stats,
            creator_stats,
            oldest_level,
        })
    }

    fn build_level_pipeline_stages(&self) -> Vec<Document> {
        vec![
            doc! {
                "$lookup": {
                    "from": "info",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "info"
                }
            },
            doc! {
                "$lookup": {
                    "from": "sends",
                    "localField": "_id",
                    "foreignField": "levelID",
                    "as": "sends"
                }
            },
            doc! {
                "$lookup": {
                    "from": "rates",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "rate"
                }
            },
            doc! {
                "$project": {
                    "level_id": "$_id",
                    "sends": {
                        "$map": {
                            "input": "$sends",
                            "as": "send",
                            "in": { "timestamp": { "$toLong": "$$send.timestamp" } }
                        }
                    },
                    "accurate": { "$gt": ["$_id", self.oldest_level] },
                    "platformer": { "$arrayElemAt": ["$info.platformer", 0] },
                    "length": { "$arrayElemAt": ["$info.length", 0] },
                    "trending_score": "$trending_score",
                    "rank": "$rank",
                    "rate_rank": "$rate_rank",
                    "gamemode_rank": "$gamemode_rank",
                    "joined_rank": "$joined_rank",
                    "trending_rank": "$trending_rank",
                    "rate": {
                        "$cond": {
                            "if": { "$gt": [{ "$size": "$rate" }, 0] },
                            "then": {
                                "difficulty": { "$arrayElemAt": ["$rate.difficulty", 0] },
                                "points": { "$arrayElemAt": ["$rate.points", 0] },
                                "stars": { "$arrayElemAt": ["$rate.stars", 0] },
                                "timestamp": { "$toLong": { "$arrayElemAt": ["$rate.timestamp", 0] } },
                                "accurate": { "$arrayElemAt": ["$rate.accurate", 0] }
                            },
                            "else": None::<Document>
                        }
                    }
                }
            }
        ]
    }

    fn build_batch_level_pipeline_stages(&self) -> Vec<Document> {
        vec![
            doc! {
                "$lookup": {
                    "from": "info",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "info"
                }
            },
            doc! {
                "$lookup": {
                    "from": "rates",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "rate"
                }
            },
            doc! {
                "$project": {
                    "level_id": "$_id",
                    "send_count": "$send_count",
                    "accurate": { "$gt": ["$_id", self.oldest_level] },
                    "platformer": { "$arrayElemAt": ["$info.platformer", 0] },
                    "length": { "$arrayElemAt": ["$info.length", 0] },
                    "rank": "$rank",
                    "trending_score": "$trending_score",
                    "rate": {
                        "$cond": {
                            "if": { "$gt": [{ "$size": "$rate" }, 0] },
                            "then": {
                                "difficulty": { "$arrayElemAt": ["$rate.difficulty", 0] },
                                "points": { "$arrayElemAt": ["$rate.points", 0] },
                                "stars": { "$arrayElemAt": ["$rate.stars", 0] },
                                "timestamp": { "$toLong": { "$arrayElemAt": ["$rate.timestamp", 0] } },
                                "accurate": { "$arrayElemAt": ["$rate.accurate", 0] }
                            },
                            "else": None::<Document>
                        }
                    }
                }
            }
        ]
    }

    fn build_creator_pipeline_stages(&self) -> Vec<Document> {
        vec![
            doc! {
                "$lookup": {
                    "from": "info",
                    "localField": "_id",
                    "foreignField": "creator",
                    "as": "creator_levels"
                }
            },
            doc! {
                "$addFields": {
                    "level_ids": {
                        "$map": {
                            "input": "$creator_levels",
                            "as": "level",
                            "in": "$$level._id"
                        }
                    }
                }
            },
            doc! {
                "$lookup": {
                    "from": "level_stats",
                    "localField": "level_ids",
                    "foreignField": "_id",
                    "as": "level_stats_data"
                }
            },
            doc! {
                "$addFields": {
                    "levels": {
                        "$map": {
                            "input": "$level_stats_data",
                            "as": "stat",
                            "in": {
                                "level_id": "$$stat._id",
                                "send_count": "$$stat.send_count"
                            }
                        }
                    }
                }
            },
            doc! {
                "$project": {
                    "player_id": "$_id",
                    "account_id": 1,
                    "levels": 1,
                    "send_count": 1,
                    "recent_sends": 1,
                    "send_count_stddev": 1,
                    "trending_score": 1,
                    "trending_level_count": 1,
                    "latest_send": { "$toLong": "$latest_send" },
                    "rank": 1,
                    "trending_rank": 1
                }
            }
        ]
    }

    fn build_leaderboard_pipeline_stages(&self, query: &LeaderboardQuery) -> Vec<Document> {
        let mut stages = vec![
            doc! {
                "$lookup": {
                    "from": "info",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "info"
                }
            },
            doc! {
                "$unwind": {
                    "path": "$info",
                    "preserveNullAndEmptyArrays": true
                }
            }
        ];

        let mut match_conditions = vec![];

        if let Some(gamemode_filter) = &query.gamemode_filter {
            match gamemode_filter {
                GamemodeFilter::Classic => {
                    match_conditions.push(doc! {
                        "info.platformer": false
                    });
                },
                GamemodeFilter::Platformer => {
                    match_conditions.push(doc! {
                        "info.platformer": true
                    });
                }
            }
        }

        if let Some(rate_filter) = &query.rate_filter {
            stages.push(doc! {
                "$lookup": {
                    "from": "rates",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "rate"
                }
            });
            match rate_filter {
                RateFilter::Rated => {
                    match_conditions.push(doc! {
                        "rate": { "$ne": [] }
                    });
                },
                RateFilter::Unrated => {
                    match_conditions.push(doc! {
                        "rate": { "$eq": [] }
                    });
                }
            }
        }

        if !match_conditions.is_empty() {
            stages.push(doc! {
                "$match": {
                    "$and": match_conditions
                }
            });
        }

        let rank_field = if query.rate_filter.is_some() && query.gamemode_filter.is_some() {
            "joined_rank"
        } else if query.rate_filter.is_some() {
            "rate_rank"
        } else if query.gamemode_filter.is_some() {
            "gamemode_rank"
        } else {
            "rank"
        };

        stages.push(doc! {
            "$facet": {
                "metadata": [
                    { "$count": "total" }
                ],
                "data": [
                    { "$sort": { rank_field: 1, "_id": 1 } },
                    { "$skip": query.offset },
                    { "$limit": query.limit },
                    {
                        "$project": {
                            "level_id": "$_id",
                            "send_count": "$send_count",
                            "rank": format!("${}", rank_field),
                        }
                    }
                ]
            }
        });

        stages
    }
}

#[async_trait::async_trait]
impl Database for MongoDatabase {
    async fn get_levels_by_ids(&self, level_ids: &[i64]) -> anyhow::Result<Vec<BatchLevel>> {
        let level_ids_i32: Vec<i32> = level_ids.iter().map(|&id| id as i32).collect();

        let mut pipeline = vec![
            doc! { "$match": { "_id": { "$in": level_ids_i32 } } }
        ];
        pipeline.extend(self.build_batch_level_pipeline_stages());

        let mut cursor = self.level_stats.aggregate(pipeline)
            .max_time(Duration::from_secs(5))
            .await?;
        let mut levels = Vec::new();

        while let Some(result) = cursor.try_next().await? {
            let level: BatchLevel = mongodb::bson::from_document(result)?;
            levels.push(level);
        }

        Ok(levels)
    }

    async fn get_level_by_id(&self, level_id: i64) -> anyhow::Result<Option<Level>> {
        let mut pipeline = vec![
            doc! { "$match": { "_id": level_id as i32 } },
            doc! { "$limit": 1 }
        ];
        pipeline.extend(self.build_level_pipeline_stages());

        let mut cursor = self.level_stats.aggregate(pipeline)
            .max_time(Duration::from_secs(5))
            .await?;

        if let Some(result) = cursor.try_next().await? {
            let level: Level = mongodb::bson::from_document(result)?;
            Ok(Some(level))
        } else {
            Ok(None)
        }
    }

    async fn get_creator_by_id(&self, player_id: i64) -> anyhow::Result<Option<Creator>> {
        let mut pipeline = vec![
            doc! { "$match": { "_id": player_id as i32 } },
            doc! { "$limit": 1 }
        ];
        pipeline.extend(self.build_creator_pipeline_stages());

        let mut cursor = self.creator_stats.aggregate(pipeline)
            .max_time(Duration::from_secs(5))
            .await?;

        if let Some(result) = cursor.try_next().await? {
            let creator: Creator = mongodb::bson::from_document(result)?;
            Ok(Some(creator))
        } else {
            Ok(None)
        }
    }

    async fn get_leaderboard_levels(&self, query: &LeaderboardQuery) -> anyhow::Result<LeaderboardResponse> {
        let pipeline = self.build_leaderboard_pipeline_stages(query);

        let result = self.level_stats
            .aggregate(pipeline)
            .max_time(Duration::from_secs(5))
            .await?
            .try_next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("No results from aggregation"))?;

        let metadata = result.get_array("metadata")?;
        let total = metadata
            .first()
            .and_then(|doc| doc.as_document())
            .and_then(|doc| doc.get_i32("total").ok())
            .unwrap_or(0);

        let data = result.get_array("data")?;
        let levels: Vec<LeaderboardLevel> = data
            .iter()
            .filter_map(|bson| bson.as_document())
            .filter_map(|doc| mongodb::bson::from_document(doc.clone()).ok())
            .collect();

        Ok(LeaderboardResponse { total, levels })
    }
}