use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace as sdktrace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing_with_otlp(
    service_name: &str,
    otlp_endpoint: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tracer =
        if let Some(endpoint) = otlp_endpoint {
            tracing::info!(
                "Initializing OpenTelemetry with OTLP endpoint: {}",
                endpoint
            );

            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()?;

            let provider = sdktrace::SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .with_resource(Resource::builder()
                    .with_service_name(service_name.to_string())
                    .build()
                )
                .build();

            let tracer = provider.tracer(service_name.to_string());

            Some(tracer)
        } else {
            tracing::info!("OTLP endpoint not configured, skipping OpenTelemetry setup");
            None
        };

    let telemetry_layer = tracer.map(|t| tracing_opentelemetry::layer().with_tracer(t));

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer());

    if let Some(layer) = telemetry_layer {
        subscriber.with(layer).init();
    } else {
        subscriber.init();
    }

    Ok(())
}

pub fn shutdown_tracing() {
    // The shutdown_tracer_provider function was removed in opentelemetry 0.31
    // Tracer providers are now automatically shut down when dropped
    tracing::info!("Shutting down tracing");
}
