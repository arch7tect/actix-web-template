use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, runtime, trace as sdktrace};
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

            let tracer =
                opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(
                        opentelemetry_otlp::new_exporter()
                            .tonic()
                            .with_endpoint(endpoint),
                    )
                    .with_trace_config(sdktrace::Config::default().with_resource(Resource::new(
                        vec![KeyValue::new("service.name", service_name.to_string())],
                    )))
                    .install_batch(runtime::Tokio)?;

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
    opentelemetry::global::shutdown_tracer_provider();
}
