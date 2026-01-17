mod model;
mod database;
mod endpoint;

use std::sync::Arc;
use tokio::sync::Mutex;
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use env_logger::{Env};
use fern::Dispatch;
use log::LevelFilter;
use utoipa::openapi::{ContactBuilder, InfoBuilder};
use utoipa_actix_web::{scope, AppExt};
use utoipa_swagger_ui::SwaggerUi;
use crate::database::mongo::MongoDatabase;
use crate::endpoint::level;
use crate::model::config::AppConfig;
use crate::model::database::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    setup_logger().map_err(|e| {
        eprintln!("Failed to initialize logger: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Logger initialization error")
    })?;

    let config = AppConfig::from_env();
    let database: Arc<Mutex<dyn Database>> = Arc::new(Mutex::new(MongoDatabase::new(config.database_url.as_str(), config.oldest_level).await
        .map_err(|e| {
            log::error!("Failed to create database connection: {:?}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "Database connection error")
        })?));

    let config_clone = config.clone();

    let governor_conf = GovernorConfigBuilder::default()
        .requests_per_minute(60)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        let (app, _) = App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .wrap(Governor::new(&governor_conf))
            .into_utoipa_app()
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(scope::scope("/api/v1")
                .service(scope::scope("/level")
                    .service(level::batch_level)
                    .service(level::get_level)
                )
            )
            .openapi_service(|mut api| {
                api.info = InfoBuilder::new()
                    .title("SendDB API".to_owned())
                    .version(env!("CARGO_PKG_VERSION").to_string())
                    .description(Some("SendDB API Documentation".to_owned()))
                    .contact(Some(ContactBuilder::new()
                        .name(Some("SorkoPiko"))
                        .url(Some("https://sorkopiko.com"))
                        .email(Some("me@sorko.dev"))
                        .build()))
                    .build();

                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .split_for_parts();
        app
    })
        .bind((config_clone.server_address, config_clone.server_port))?
        .run()
        .await
}

fn setup_logger() -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
