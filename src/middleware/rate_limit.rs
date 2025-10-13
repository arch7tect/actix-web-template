use actix_governor::{Governor, GovernorConfigBuilder, PeerIpKeyExtractor};

pub fn create_rate_limiter() -> Governor {
    tracing::info!("Configuring rate limiting: 100 requests per minute per IP");

    let governor_conf = GovernorConfigBuilder::default()
        .key_extractor(PeerIpKeyExtractor)
        .milliseconds_per_request(600)
        .burst_size(100)
        .use_headers()
        .finish()
        .expect("Failed to create rate limiter configuration");

    Governor::new(&governor_conf)
}