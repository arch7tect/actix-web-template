use actix_web::{web, App, HttpServer};
use actix_web_template::{config::Settings, handlers, state::AppState, utils::init_tracing};

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

    let state = AppState::new(settings.clone());

    let bind_address = format!("{}:{}", settings.server.host, settings.server.port);
    tracing::info!(address = %bind_address, "Starting HTTP server");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(handlers::health_check)
    })
    .bind(&bind_address)?
    .run()
    .await?;

    tracing::info!("Application shutdown complete");
    Ok(())
}
