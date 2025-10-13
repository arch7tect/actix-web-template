use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
    App, HttpServer,
    middleware::{Compress, Logger},
    web,
};
use actix_web_template::{
    config::Settings, docs::ApiDoc, handlers, middleware::SecurityHeaders, state::AppState,
    utils::init_tracing,
};
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load()?;

    init_tracing(&settings.logging)?;

    tracing::info!(
        version = settings.app.version,
        env = ?settings.app.env,
        "Starting Actix Web Memos application"
    );

    settings.validate()?;

    tracing::info!(
        url = %settings.database.url.split('@').next_back().unwrap_or("***"),
        max_connections = settings.database.max_connections,
        connect_timeout = settings.database.connect_timeout,
        "Connecting to database with tuned connection pool"
    );

    let mut opt = ConnectOptions::new(&settings.database.url);
    opt.max_connections(settings.database.max_connections)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(settings.database.connect_timeout))
        .acquire_timeout(Duration::from_secs(settings.database.connect_timeout))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;
    tracing::info!("Database connection established with optimized pool settings");

    let state = AppState::new(settings.clone(), db);

    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);
    tracing::info!(address = %bind_address, "Starting HTTP server");

    tracing::info!("Configuring rate limiting: 100 requests per minute per IP");
    let governor_conf = GovernorConfigBuilder::default()
        .milliseconds_per_request(600)
        .burst_size(100)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        let rate_limiter = Governor::new(&governor_conf);
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
            .wrap(Compress::default())
            .wrap(SecurityHeaders)
            .wrap(rate_limiter)
            .wrap(cors)
            .wrap(Logger::default())
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
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
    .bind(&bind_address)?
    .run()
    .await?;

    tracing::info!("Application shutdown complete");
    Ok(())
}
