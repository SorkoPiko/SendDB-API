use std::net::IpAddr;
use actix_web::body::{MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpResponse};
use actix_web::middleware::Next;

pub async fn metrics_ip_filter(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if req.path() == "/metrics" {
        let peer_addr = {
            let connection_info = req.connection_info();
            req.headers().get("X-Real-IP")
                .or_else(|| req.headers().get("X-Forwarded-For"))
                .and_then(|value| value.to_str().ok())
                .or_else(|| connection_info.peer_addr())
                .and_then(|addr| addr.parse::<IpAddr>().ok())
        };

        if let Some(ip) = peer_addr && !is_local_ip(ip) {
            let (req, _) = req.into_parts();
            return Ok(ServiceResponse::new(
                req,
                HttpResponse::NotFound().finish()
            ).map_into_boxed_body());
        }
    }

    next.call(req).await.map(ServiceResponse::map_into_boxed_body)
}

fn is_local_ip(ip: IpAddr) -> bool {
    log::debug!("Checking IP address: {}", ip);
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_private(),
        IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unique_local(),
    }
}