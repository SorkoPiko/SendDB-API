use std::time::Duration;
use anyhow::Context;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use crate::endpoint::leaderboard::{CreatorLeaderboardQuery, CreatorLeaderboardResponse, GamemodeFilter, LeaderboardQuery, LeaderboardResponse, RateFilter, TrendingLeaderboardQuery, TrendingLeaderboardResponse};
use crate::endpoint::search::{SearchQuery, SearchResponse};
use crate::model::database::{CreatorStatItem, Database, InfoItem, LevelStatItem, RateItem, SendItem};
use crate::model::info::{BatchLevel, Creator, LeaderboardCreator, LeaderboardLevel, Level, SearchCreator, SearchLevel, SearchResult, TrendingLeaderboardLevel};

pub struct MongoDatabase {
    client: mongodb::Client,

    info: mongodb::Collection<InfoItem>,
    rates: mongodb::Collection<RateItem>,
    sends: mongodb::Collection<SendItem>,
    level_stats: mongodb::Collection<LevelStatItem>,
    creator_stats: mongodb::Collection<CreatorStatItem>,
    creators: mongodb::Collection<Document>,

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
        let creators = db.collection("creators");

        Ok(Self {
            client,
            info,
            rates,
            sends,
            level_stats,
            creator_stats,
            creators,
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
                "$lookup": {
                    "from": "creators",
                    "localField": "info.creator",
                    "foreignField": "_id",
                    "as": "creator_info"
                }
            },
            doc! {
                "$project": {
                    "level_id": "$_id",
                    "name": { "$arrayElemAt": ["$info.name", 0] },
                    "creator": {
                        "name": { "$arrayElemAt": ["$creator_info.name", 0] },
                        "player_id": { "$arrayElemAt": ["$info.creator", 0] }
                    },
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
                                "send_count": "$$stat.send_count",
                                "name": {
                                    "$reduce": {
                                        "input": "$creator_levels",
                                        "initialValue": null,
                                        "in": {
                                            "$cond": {
                                                "if": { "$eq": ["$$this._id", "$$stat._id"] },
                                                "then": "$$this.name",
                                                "else": "$$value"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            doc! {
                "$lookup": {
                    "from": "creators",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "creator_info"
                }
            },
            doc! {
                "$project": {
                    "name": { "$arrayElemAt": ["$creator_info.name", 0] },
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
        let rank_field = if query.rate_filter.is_some() && query.gamemode_filter.is_some() {
            "joined_rank"
        } else if query.rate_filter.is_some() {
            "rate_rank"
        } else if query.gamemode_filter.is_some() {
            "gamemode_rank"
        } else {
            "rank"
        };

        let needs_info = query.gamemode_filter.is_some() || query.search.as_ref().map_or(false, |s| !s.trim().is_empty());
        let needs_rate = query.rate_filter.is_some();

        let mut pre_facet_stages: Vec<Document> = vec![];

        if needs_info {
            pre_facet_stages.push(doc! {
                "$lookup": {
                    "from": "info",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "info"
                }
            });
            pre_facet_stages.push(doc! {
                "$unwind": {
                    "path": "$info",
                    "preserveNullAndEmptyArrays": true
                }
            });
        }

        if let Some(search) = &query.search && needs_info {
            if let Ok(level_id) = search.parse::<i32>() && level_id > 100000 {
                pre_facet_stages.push(doc! {
                    "$match": { "_id": level_id }
                });
            } else {
                pre_facet_stages.push(doc! {
                    "$match": {
                        "info.name": {
                            "$regex": search,
                            "$options": "i"
                        }
                    }
                });
            }
        }

        if needs_rate {
            pre_facet_stages.push(doc! {
                "$lookup": {
                    "from": "rates",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "rate"
                }
            });
        }

        let mut match_conditions: Vec<Document> = vec![];

        if let Some(gamemode_filter) = &query.gamemode_filter {
            match gamemode_filter {
                GamemodeFilter::Classic => {
                    match_conditions.push(doc! { "info.platformer": false });
                }
                GamemodeFilter::Platformer => {
                    match_conditions.push(doc! { "info.platformer": true });
                }
            }
        }

        if let Some(rate_filter) = &query.rate_filter {
            match rate_filter {
                RateFilter::Rated => {
                    match_conditions.push(doc! { "rate": { "$ne": [] } });
                }
                RateFilter::Unrated => {
                    match_conditions.push(doc! { "rate": { "$eq": [] } });
                }
            }
        }

        if !match_conditions.is_empty() {
            pre_facet_stages.push(doc! {
                "$match": { "$and": match_conditions }
            });
        }

        let mut info_lookup_in_facet = vec![];

        if !needs_info {
            info_lookup_in_facet.push(doc! {
                "$lookup": {
                    "from": "info",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "info"
                }
            });
            info_lookup_in_facet.push(doc! {
                "$unwind": {
                    "path": "$info",
                    "preserveNullAndEmptyArrays": true
                }
            });
        }

        if !needs_rate {
            info_lookup_in_facet.push(doc! {
                "$lookup": {
                    "from": "rates",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "rate"
                }
            });
        }

        info_lookup_in_facet.push(doc! {
            "$lookup": {
                "from": "creators",
                "localField": "info.creator",
                "foreignField": "_id",
                "as": "creator_info"
            }
        });

        info_lookup_in_facet.push(doc! {
            "$project": {
                "name": "$info.name",
                "level_id": "$_id",
                "send_count": "$send_count",
                "rank": format!("${}", rank_field),
                "creator": {
                    "name": { "$arrayElemAt": ["$creator_info.name", 0] },
                    "player_id": "$info.creator"
                },
                "platformer": "$info.platformer",
                "rate": {
                    "$cond": {
                        "if": { "$gt": [{ "$size": "$rate" }, 0] },
                        "then": {
                            "difficulty": { "$arrayElemAt": ["$rate.difficulty", 0] },
                            "points": { "$arrayElemAt": ["$rate.points", 0] },
                            "stars": { "$arrayElemAt": ["$rate.stars", 0] }
                        },
                        "else": None::<Document>
                    }
                }
            }
        });

        let mut facet_data_stages: Vec<mongodb::bson::Bson> = vec![
            doc! { "$sort": { rank_field: 1, "_id": 1 } }.into(),
            doc! { "$skip": query.offset }.into(),
            doc! { "$limit": query.limit }.into(),
        ];
        for stage in info_lookup_in_facet {
            facet_data_stages.push(stage.into());
        }

        let mut stages = pre_facet_stages;
        stages.push(doc! {
            "$facet": {
                "metadata": [
                    { "$count": "total" }
                ],
                "data": facet_data_stages
            }
        });

        stages
    }

    fn build_trending_pipeline_stages(&self, query: &TrendingLeaderboardQuery) -> Vec<Document> {
        let needs_info = query.search.as_ref().map_or(false, |s| !s.trim().is_empty());

        let mut stages: Vec<Document> = vec![
            doc! {
                "$match": {
                    "trending_rank": { "$gt": 0 }
                }
            },
        ];

        if needs_info {
            stages.push(doc! {
                "$lookup": {
                    "from": "info",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "info"
                }
            });
            stages.push(doc! {
                "$unwind": {
                    "path": "$info",
                    "preserveNullAndEmptyArrays": true
                }
            });

            if let Some(search) = &query.search && needs_info {
                if let Ok(level_id) = search.parse::<i32>() && level_id > 100000 {
                    stages.push(doc! { "$match": { "_id": level_id } });
                } else {
                    stages.push(doc! {
                        "$match": {
                            "info.name": {
                                "$regex": search,
                                "$options": "i"
                            }
                        }
                    });
                }
            }
        }

        let mut data_facet_stages: Vec<Document> = vec![
            doc! { "$sort": { "trending_rank": 1, "_id": 1 } },
            doc! { "$skip": query.offset },
            doc! { "$limit": query.limit }
        ];

        if !needs_info  {
            data_facet_stages.push(doc! {
                "$lookup": {
                    "from": "info",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "info"
                }
            });
            data_facet_stages.push(doc! {
                "$unwind": {
                    "path": "$info",
                    "preserveNullAndEmptyArrays": true
                }
            });
        };

        data_facet_stages.push(doc! {
            "$lookup": {
                "from": "creators",
                "localField": "info.creator",
                "foreignField": "_id",
                "as": "creator_info"
            }
        });
        data_facet_stages.push(doc! {
            "$lookup": {
                "from": "rates",
                "localField": "_id",
                "foreignField": "_id",
                "as": "rate"
            }
        });
        data_facet_stages.push(doc! {
            "$project": {
                "name": "$info.name",
                "level_id": "$_id",
                "send_count": "$send_count",
                "creator": {
                    "name": { "$arrayElemAt": ["$creator_info.name", 0] },
                    "player_id": "$info.creator"
                },
                "rank": "$trending_rank",
                "trending_score": "$trending_score",
                "platformer": "$info.platformer",
                "rate": {
                    "$cond": {
                        "if": { "$gt": [{ "$size": "$rate" }, 0] },
                        "then": {
                            "difficulty": { "$arrayElemAt": ["$rate.difficulty", 0] },
                            "points": { "$arrayElemAt": ["$rate.points", 0] },
                            "stars": { "$arrayElemAt": ["$rate.stars", 0] }
                        },
                        "else": None::<Document>
                    }
                }
            }
        });

        stages.push(doc! {
            "$facet": {
                "metadata": [
                    { "$count": "total" }
                ],
                "data": data_facet_stages
            }
        });

        stages
    }

    fn build_creator_leaderboard_pipeline_stages(&self, query: &CreatorLeaderboardQuery) -> Vec<Document> {
        let needs_name = query.search.as_ref().map_or(false, |s| !s.trim().is_empty());

        let mut stages: Vec<Document> = vec![];

        if needs_name {
            stages.push(doc! {
                "$lookup": {
                    "from": "creators",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "creator_info"
                }
            });
            stages.push(doc! {
                "$unwind": {
                    "path": "$creator_info",
                    "preserveNullAndEmptyArrays": true
                }
            });

            if let Some(search) = &query.search && needs_name {
                if let Ok(player_id) = search.parse::<i32>() && player_id > 100000 {
                    stages.push(doc! { "$match": { "_id": player_id } });
                } else {
                    stages.push(doc! {
                        "$match": {
                            "creator_info.name": {
                                "$regex": search,
                                "$options": "i"
                            }
                        }
                    });
                }
            }
        }

        let mut data_facet_stages: Vec<Document> = vec![
            doc! { "$sort": { "rank": 1, "_id": 1 } },
            doc! { "$skip": query.offset },
            doc! { "$limit": query.limit }
        ];

        if needs_name {
            data_facet_stages.push(doc! {
                "$project": {
                    "name": "$creator_info.name",
                    "player_id": "$_id",
                    "account_id": 1,
                    "level_count": 1,
                    "send_count": 1,
                    "trending_score": 1,
                    "rank": 1,
                    "trending_rank": 1
                }
            });
        } else {
            data_facet_stages.push(doc! {
                "$lookup": {
                    "from": "creators",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "creator_info"
                }
            });
            data_facet_stages.push(doc! {
                "$project": {
                    "name": { "$arrayElemAt": ["$creator_info.name", 0] },
                    "player_id": "$_id",
                    "account_id": 1,
                    "level_count": 1,
                    "send_count": 1,
                    "trending_score": 1,
                    "rank": 1,
                    "trending_rank": 1
                }
            });
        };

        stages.push(doc! {
            "$facet": {
                "metadata": [
                    { "$count": "total" }
                ],
                "data": data_facet_stages
            }
        });

        stages
    }

    fn build_search_relevance_stages(search: &str, name_field: &str, id_field: &str) -> Vec<Document> {
        vec![
            doc! {
                "$addFields": {
                    "relevance": {
                        "$switch": {
                            "branches": [
                                {
                                    "case": { "$eq": [{ "$toLower": format!("${}", name_field) }, search.to_lowercase()] },
                                    "then": 3
                                },
                                {
                                    "case": {
                                        "$regexMatch": {
                                            "input": { "$toLower": format!("${}", name_field) },
                                            "regex": format!("^{}", regex::escape(search).to_lowercase())
                                        }
                                    },
                                    "then": 2
                                },
                                {
                                    "case": {
                                        "$regexMatch": {
                                            "input": { "$toLower": format!("${}", name_field) },
                                            "regex": regex::escape(search).to_lowercase()
                                        }
                                    },
                                    "then": 1
                                }
                            ],
                            "default": 0
                        }
                    }
                }
            },
            doc! { "$sort": { "relevance": -1, id_field: 1 } }
        ]
    }

    fn build_level_search_pipeline(&self, search: &str, limit: i64) -> Vec<Document> {
        let is_id = search.parse::<i32>().ok().filter(|&id| id > 100000);

        let match_stage = if let Some(id) = is_id {
            doc! { "$match": { "_id": id } }
        } else {
            doc! {
                "$match": {
                    "name": { "$regex": search, "$options": "i" }
                }
            }
        };

        let mut pipeline = vec![match_stage];

        if is_id.is_none() {
            pipeline.extend(Self::build_search_relevance_stages(search, "name", "_id"));
        }

        pipeline.push(doc! { "$limit": limit });

        pipeline.push(doc! {
            "$lookup": {
                "from": "level_stats",
                "localField": "_id",
                "foreignField": "_id",
                "as": "stats"
            }
        });
        pipeline.push(doc! {
            "$lookup": {
                "from": "rates",
                "localField": "_id",
                "foreignField": "_id",
                "as": "rate"
            }
        });
        pipeline.push(doc! {
            "$lookup": {
                "from": "creators",
                "localField": "creator",
                "foreignField": "_id",
                "as": "creator_info"
            }
        });
        pipeline.push(doc! {
            "$project": {
                "level_id": "$_id",
                "name": 1,
                "platformer": 1,
                "relevance": { "$ifNull": ["$relevance", 3] },
                "creator": {
                    "name": { "$arrayElemAt": ["$creator_info.name", 0] },
                    "player_id": "$creator"
                },
                "rate": {
                    "$cond": {
                        "if": { "$gt": [{ "$size": "$rate" }, 0] },
                        "then": {
                            "difficulty": { "$arrayElemAt": ["$rate.difficulty", 0] },
                            "stars": { "$arrayElemAt": ["$rate.stars", 0] }
                        },
                        "else": None::<Document>
                    }
                }
            }
        });

        pipeline
    }

    fn build_creator_search_pipeline(&self, search: &str, limit: i64) -> Vec<Document> {
        let is_id = search.parse::<i32>().ok().filter(|&id| id > 100000);

        let match_stage = if let Some(id) = is_id {
            doc! { "$match": { "_id": id } }
        } else {
            doc! {
                "$match": {
                    "name": { "$regex": search, "$options": "i" }
                }
            }
        };

        let mut pipeline = vec![match_stage];

        if is_id.is_none() {
            pipeline.extend(Self::build_search_relevance_stages(search, "name", "_id"));
        }

        pipeline.push(doc! { "$limit": limit });

        pipeline.push(doc! {
            "$lookup": {
                "from": "creator_stats",
                "localField": "_id",
                "foreignField": "_id",
                "as": "stats"
            }
        });
        pipeline.push(doc! {
            "$project": {
                "player_id": "$_id",
                "name": 1,
                "account_id": { "$arrayElemAt": ["$stats.account_id", 0] },
                "send_count": { "$arrayElemAt": ["$stats.send_count", 0] },
                "rank": { "$arrayElemAt": ["$stats.rank", 0] },
                "relevance": { "$ifNull": ["$relevance", 3] }
            }
        });

        pipeline
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
            log::info!("Raw creator document: {:?}", result);
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

    async fn get_trending_levels(&self, query: &TrendingLeaderboardQuery) -> anyhow::Result<TrendingLeaderboardResponse> {
        let pipeline = self.build_trending_pipeline_stages(query);

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
        let levels: Vec<TrendingLeaderboardLevel> = data
            .iter()
            .filter_map(|bson| bson.as_document())
            .filter_map(|doc| mongodb::bson::from_document(doc.clone()).ok())
            .collect();

        Ok(TrendingLeaderboardResponse { total, levels })
    }

    async fn get_creators(&self, query: &CreatorLeaderboardQuery) -> anyhow::Result<CreatorLeaderboardResponse> {
        let pipeline = self.build_creator_leaderboard_pipeline_stages(query);

        let result = self.creator_stats
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
        let creators: Vec<LeaderboardCreator> = data
            .iter()
            .filter_map(|bson| bson.as_document())
            .filter_map(|doc| mongodb::bson::from_document(doc.clone()).ok())
            .collect();

        Ok(CreatorLeaderboardResponse { total, creators })
    }

    async fn search(&self, query: &SearchQuery) -> anyhow::Result<SearchResponse> {
        let search = query.search.trim();
        let limit = query.limit.min(100);

        let level_pipeline = self.build_level_search_pipeline(search, limit);
        let creator_pipeline = self.build_creator_search_pipeline(search, limit);

        let (level_cursor, creator_cursor) = tokio::try_join!(
            self.info.aggregate(level_pipeline).max_time(Duration::from_secs(5)),
            self.creators.aggregate(creator_pipeline).max_time(Duration::from_secs(5))
        )?;

        let (level_docs, creator_docs): (Vec<_>, Vec<_>) = tokio::try_join!(
            level_cursor.try_collect(),
            creator_cursor.try_collect()
        )?;

        let mut results: Vec<SearchResult> = Vec::new();

        for doc in level_docs {
            if let Ok(level) = mongodb::bson::from_document::<SearchLevel>(doc) {
                results.push(SearchResult::Level(level));
            }
        }

        for doc in creator_docs {
            if let Ok(creator) = mongodb::bson::from_document::<SearchCreator>(doc) {
                results.push(SearchResult::Creator(creator));
            }
        }

        results.sort_by(|a, b| {
            let rel_a = match a { SearchResult::Level(l) => l.relevance, SearchResult::Creator(c) => c.relevance };
            let rel_b = match b { SearchResult::Level(l) => l.relevance, SearchResult::Creator(c) => c.relevance };
            rel_b.partial_cmp(&rel_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(limit as usize);
        let total = results.len() as i32;

        Ok(SearchResponse { total, results })
    }
}