use prometheus::{Encoder, Registry, TextEncoder};

#[derive(Clone)]
pub struct MetricsExporter {
    registry: Registry,
}

impl MetricsExporter {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        Ok(Self { registry })
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn collect_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();

        if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
            tracing::error!("Failed to encode metrics: {}", e);
            return String::new();
        }

        String::from_utf8(buffer).unwrap_or_default()
    }
}

impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics exporter")
    }
}
