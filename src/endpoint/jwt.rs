use crate::endpoint::common::unauthorized;
use crate::model::config::AppConfig;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{Error, HttpMessage};
use anyhow::Context;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn create_token(sub: String, secret: &str, duration_secs: i64) -> anyhow::Result<String> {
    let now = Utc::now().timestamp();
    let claims = TokenClaims {
        sub,
        exp: now + duration_secs,
        iat: now,
    };

    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    let token = encode(&Header::default(), &claims, &encoding_key).context("Failed to encode token")?;

    Ok(token)
}

pub fn verify_token(token: &str, secret: &str) -> anyhow::Result<TokenClaims> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    let data = decode::<TokenClaims>(token, &decoding_key, &validation).context("Failed to decode token")?;
    let current_time = Utc::now().timestamp();
    if data.claims.exp < current_time {
        anyhow::bail!("Token expired");
    }

    Ok(data.claims)
}

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => return Err(unauthorized("Invalid Authorization header")),
        };

        let parts: Vec<&str> = auth_str.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(unauthorized("Invalid Authorization header format"));
        }

        let token = parts[1];

        let app_config = match req.app_data::<actix_web::web::Data<AppConfig>>() {
            Some(config) => config.clone(),
            None => return next.call(req).await,
        };

        let secret = match &app_config.jwt_secret {
            Some(s) => s,
            None => return next.call(req).await,
        };

        match verify_token(token, secret) {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
                next.call(req).await
            }
            Err(_) => Err(unauthorized("Invalid or expired token")),
        }
    } else {
        next.call(req).await
    }
}