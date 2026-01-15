use anyhow::Context;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use crate::model::database::{Database, InfoItem, RateItem, SendItem};
use crate::model::info::Level;

pub struct MongoDatabase {
    client: mongodb::Client,

    info: mongodb::Collection<InfoItem>,
    rates: mongodb::Collection<RateItem>,
    sends: mongodb::Collection<SendItem>,

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

        Ok(Self {
            client,
            info,
            rates,
            sends,
            oldest_level,
        })
    }

    fn build_level_pipeline_stages(&self) -> Vec<Document> {
        vec![
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
                    "platformer": 1,
                    "length": 1,
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
}

#[async_trait::async_trait]
impl Database for MongoDatabase {
    async fn get_levels_by_ids(&self, level_ids: &[i64]) -> anyhow::Result<Vec<Level>> {
        let level_ids_i32: Vec<i32> = level_ids.iter().map(|&id| id as i32).collect();

        let mut pipeline = vec![
            doc! { "$match": { "_id": { "$in": level_ids_i32 } } }
        ];
        pipeline.extend(self.build_level_pipeline_stages());

        let mut cursor = self.info.aggregate(pipeline).await?;
        let mut levels = Vec::new();

        while let Some(result) = cursor.try_next().await? {
            let level: Level = mongodb::bson::from_document(result)?;
            levels.push(level);
        }

        Ok(levels)
    }

    async fn get_level_by_id(&self, level_id: i64) -> anyhow::Result<Option<Level>> {
        let mut pipeline = vec![
            doc! { "$match": { "_id": level_id as i32 } }
        ];
        pipeline.extend(self.build_level_pipeline_stages());

        let mut cursor = self.info.aggregate(pipeline).await?;

        if let Some(result) = cursor.try_next().await? {
            let level: Level = mongodb::bson::from_document(result)?;
            Ok(Some(level))
        } else {
            Ok(None)
        }
    }
}