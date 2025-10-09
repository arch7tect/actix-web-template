use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use actix_web_template::{config::Settings, handlers, state::AppState, utils::init_tracing};
use sea_orm::Database;

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
        url = %settings.database.url.split('@').last().unwrap_or("***"),
        max_connections = settings.database.max_connections,
        "Connecting to database"
    );

    let db = Database::connect(&settings.database.url).await?;
    tracing::info!("Database connection established");

    let state = AppState::new(settings.clone(), db);

    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);
    tracing::info!(address = %bind_address, "Starting HTTP server");

    HttpServer::new(move || {
        let cors = if state.config.cors.allowed_origins.len() == 1
            && state.config.cors.allowed_origins[0] == "*" {
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

        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::JsonConfig::default().limit(state.config.api.max_request_size))
            .app_data(web::PayloadConfig::default().limit(state.config.api.max_request_size))
            .wrap(cors)
            .wrap(Logger::default())
            .service(handlers::health_check)
            .service(handlers::test_not_found)
            .service(handlers::test_validation)
            .service(handlers::test_internal)
            .service(handlers::test_database)
    })
    .bind(&bind_address)?
    .run()
    .await?;

    tracing::info!("Application shutdown complete");
    Ok(())
}
