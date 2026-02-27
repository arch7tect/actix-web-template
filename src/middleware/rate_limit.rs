use actix_governor::{
    Governor, GovernorConfig, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError,
    governor::middleware::NoOpMiddleware,
};
use actix_web::dev::ServiceRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForwardedIpKeyExtractor;

impl KeyExtractor for ForwardedIpKeyExtractor {
    type Key = String;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // Use rightmost X-Forwarded-For IP (proxy-appended, not client-controlled)
        if let Some(forwarded) = req.headers().get("x-forwarded-for")
            && let Ok(value) = forwarded.to_str()
            && let Some(ip) = value.rsplit(',').next()
        {
            let ip = ip.trim();
            if !ip.is_empty() {
                return Ok(ip.to_string());
            }
        }

        if let Some(real_ip) = req.headers().get("x-real-ip")
            && let Ok(ip) = real_ip.to_str()
        {
            let ip = ip.trim();
            if !ip.is_empty() {
                return Ok(ip.to_string());
            }
        }

        Ok(req
            .peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string()))
    }
}

pub fn create_rate_limiter_config(
) -> anyhow::Result<GovernorConfig<ForwardedIpKeyExtractor, NoOpMiddleware>> {
    let mut builder = GovernorConfigBuilder::default().key_extractor(ForwardedIpKeyExtractor);
    builder.milliseconds_per_request(600).burst_size(100);
    builder
        .finish()
        .ok_or_else(|| anyhow::anyhow!("Failed to create rate limiter configuration"))
}

pub fn create_governor(
    config: &GovernorConfig<ForwardedIpKeyExtractor, NoOpMiddleware>,
) -> Governor<ForwardedIpKeyExtractor, NoOpMiddleware> {
    Governor::new(config)
}
