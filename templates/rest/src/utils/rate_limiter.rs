use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use actix_web::{web::Data, HttpRequest, HttpResponse};
use log::{debug, info, warn};

use super::response::ApiError;

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, (usize, Instant)>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        RateLimiter {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }
    pub fn is_allowed(&self, ip: &str) -> bool {
        debug!("Checking if ip: '{}' is allowed", ip);
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        let entry = requests.entry(ip.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) > self.window {
            *entry = (1, now);
            true
        } else {
            if entry.0 < self.max_requests {
                entry.0 += 1;
                info!("Ip: '{}' allowed", ip);
                true
            } else {
                warn!("Ip: '{}' not allowed", ip);
                false
            }
        }
    }
    pub fn check_rate_limit(&self, req: &HttpRequest) -> Result<(), ApiError> {
        if let Some(peer_addr) = req.peer_addr() {
            let ip = peer_addr.ip().to_string();
            if !self.is_allowed(&ip) {
                return Err(ApiError::TooManyRequests);
            }
        }
        Ok(())
    }
}

pub async fn index(req: HttpRequest, data: Data<RateLimiter>) -> Result<HttpResponse, ApiError> {
    data.check_rate_limit(&req).map_err(ApiError::from)?;
    Ok(HttpResponse::Ok().body("Hello, World!"))
}

use std::time::Duration;

use crate::models::rate_limiter::RateLimiter;

pub fn rate_limiter() -> RateLimiter {
    RateLimiter::new(100, Duration::from_secs(60))
}
