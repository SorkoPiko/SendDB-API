use crate::endpoint::jwt::TokenClaims;
use actix_governor::governor::clock::{Clock, DefaultClock, QuantaInstant};
use actix_governor::governor::NotUntil;
use actix_governor::{KeyExtractor, SimpleKeyExtractionError};
use actix_web::{HttpMessage, HttpResponse, HttpResponseBuilder};
use std::net::IpAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitKey {
    Ip(IpAddr),
    Exempt
}

impl std::fmt::Display for RateLimitKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitKey::Ip(ip) => write!(f, "ip:{}", ip),
            RateLimitKey::Exempt => write!(f, "exempt"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IpKeyExtractor;

impl IpKeyExtractor {
    fn extract_token(&self, req: &actix_web::dev::ServiceRequest) -> Option<TokenClaims> {
        req.extensions().get::<TokenClaims>().cloned()
    }
}

impl KeyExtractor for IpKeyExtractor {
    type Key = RateLimitKey;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &actix_web::dev::ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        if let Some(_) = self.extract_token(req) {
            return Ok(RateLimitKey::Exempt);
        }

        if let Some(real_ip) = req.headers().get("X-Real-IP") {
            if let Ok(ip_str) = real_ip.to_str() {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    return Ok(RateLimitKey::Ip(ip));
                }
            }
        }

        if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
            if let Ok(forwarded_str) = forwarded.to_str() {
                if let Some(first_ip) = forwarded_str.split(',').next() {
                    if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                        return Ok(RateLimitKey::Ip(ip));
                    }
                }
            }
        }

        req.peer_addr()
            .map(|socket| RateLimitKey::Ip(socket.ip()))
            .ok_or_else(|| SimpleKeyExtractionError::new("Could not extract IP"))
    }

    fn exceed_rate_limit_response(
        &self,
        negative: &NotUntil<QuantaInstant>,
        mut response: HttpResponseBuilder,
    ) -> HttpResponse {
        let wait_time = negative
            .wait_time_from(DefaultClock::default().now())
            .as_secs();

        response
            .json(serde_json::json!({
                "error": true,
                "message": format!("You have made too many requests. Please try again in {} seconds.", wait_time),
                "retry_after": wait_time
            }))
    }

    fn whitelisted_keys(&self) -> Vec<Self::Key> {
        vec![RateLimitKey::Exempt]
    }
}