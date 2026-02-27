use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Compress, web};
use actix_web_prom::PrometheusMetricsBuilder;
use actix_web_template::{
    config::Settings,
    docs::ApiDoc,
    handlers,
    middleware::{SecurityHeaders, create_governor, create_rate_limiter_config},
    observability::tracing::{init_tracing_with_otlp, shutdown_tracing},
    state::AppState,
};
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load()?;

    let otlp_endpoint = std::env::var("OTLP_ENDPOINT").ok();
    init_tracing_with_otlp("memos-api", otlp_endpoint)
        .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;

    tracing::info!(
        version = settings.app.version,
        env = ?settings.app.env,
        "Starting Actix Web Memos application"
    );

    settings.validate()?;

    tracing::info!(
        url = %settings.database.url.split('@').next_back().unwrap_or("***"),
        max_connections = settings.database.max_connections,
        min_connections = settings.database.min_connections,
        connect_timeout = settings.database.connect_timeout,
        idle_timeout = settings.database.idle_timeout,
        max_lifetime = settings.database.max_lifetime,
        "Connecting to database with tuned connection pool"
    );

    let mut opt = ConnectOptions::new(&settings.database.url);
    opt.max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)
        .connect_timeout(Duration::from_secs(settings.database.connect_timeout))
        .acquire_timeout(Duration::from_secs(settings.database.connect_timeout))
        .idle_timeout(Duration::from_secs(settings.database.idle_timeout))
        .max_lifetime(Duration::from_secs(settings.database.max_lifetime))
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;
    tracing::info!("Database connection established with optimized pool settings");

    tracing::info!("Initializing Prometheus metrics exporter");
    let prometheus = PrometheusMetricsBuilder::new("actix_web")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let state = AppState::new(settings.clone(), db);

    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);
    tracing::info!(address = %bind_address, "Starting HTTP server");

    if settings.server.trust_proxy {
        tracing::info!(
            "TRUST_PROXY=true: rate limiter will use X-Forwarded-For / X-Real-Ip headers"
        );
    }

    tracing::info!("Configuring rate limiting: 100 requests per minute per IP");
    let governor_conf = create_rate_limiter_config()?;

    let security_hsts = settings.security.hsts_enabled;
    let security_frame = settings.security.frame_options.clone();

    HttpServer::new(move || {
        let rate_limiter = create_governor(&governor_conf);
        let cors = if state.config.cors.allowed_origins.len() == 1
            && state.config.cors.allowed_origins[0] == "*"
        {
            Cors::permissive()
        } else {
            let mut cors = Cors::default();
            for origin in &state.config.cors.allowed_origins {
                cors = cors.allowed_origin(origin.as_str());
            }
            cors.allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .max_age(3600)
        };

        let openapi = ApiDoc::openapi();

        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::JsonConfig::default().limit(state.config.api.max_request_size))
            .app_data(web::PayloadConfig::default().limit(state.config.api.max_request_size))
            .wrap(prometheus.clone())
            .wrap(Compress::default())
            .wrap(SecurityHeaders::new(security_hsts, &security_frame))
            .wrap(rate_limiter)
            .wrap(cors)
            .wrap(TracingLogger::default())
            .service(actix_files::Files::new("/static", "./static"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(handlers::index)
            .service(handlers::get_memos_list)
            .service(handlers::get_new_memo_form)
            .service(handlers::create_memo_web)
            .service(handlers::get_edit_memo_form)
            .service(handlers::update_memo_web)
            .service(handlers::delete_memo_web)
            .service(handlers::toggle_memo_complete_web)
            .service(handlers::health_check)
            .service(handlers::ready)
            .service(handlers::list_memos)
            .service(handlers::get_memo)
            .service(handlers::create_memo)
            .service(handlers::update_memo)
            .service(handlers::patch_memo)
            .service(handlers::delete_memo)
            .service(handlers::toggle_complete)
            .service(handlers::list_tags)
            .service(handlers::test_not_found)
            .service(handlers::test_validation)
            .service(handlers::test_internal)
            .service(handlers::test_database)
            .service(handlers::test_create_dto)
            .service(handlers::test_repo)
            .service(handlers::test_svc)
    })
    .workers(num_cpus::get() * 2)
    .keep_alive(Duration::from_secs(75))
    .client_request_timeout(Duration::from_secs(60))
    .client_disconnect_timeout(Duration::from_secs(5))
    .shutdown_timeout(30)
    .bind(&bind_address)?
    .run()
    .await?;

    shutdown_tracing();
    tracing::info!("Application shutdown complete");
    Ok(())
}
