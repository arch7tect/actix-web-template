use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cors: CorsConfig,
    pub api: ApiConfig,
    pub app: AppConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    pub max_request_size: usize,
    pub enable_swagger: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub env: Environment,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Pretty,
    Json,
    Compact,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();

        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3737".to_string())
                .parse()?,
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            connect_timeout: env::var("DATABASE_CONNECT_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
        };

        let cors = CorsConfig {
            allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        };

        let api = ApiConfig {
            max_request_size: env::var("MAX_REQUEST_SIZE")
                .unwrap_or_else(|_| "262144".to_string())
                .parse()?,
            enable_swagger: env::var("ENABLE_SWAGGER")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        };

        let app_env_str = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let app = AppConfig {
            env: match app_env_str.to_lowercase().as_str() {
                "production" => Environment::Production,
                "staging" => Environment::Staging,
                _ => Environment::Development,
            },
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let logging = LoggingConfig {
            level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            format: match env::var("LOG_FORMAT")
                .unwrap_or_else(|_| "pretty".to_string())
                .to_lowercase()
                .as_str()
            {
                "json" => LogFormat::Json,
                "compact" => LogFormat::Compact,
                _ => LogFormat::Pretty,
            },
        };

        tracing::info!("Configuration loaded successfully");
        tracing::debug!(?app.env, ?logging.format, "Application configuration");

        Ok(Settings {
            server,
            database,
            cors,
            api,
            app,
            logging,
        })
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.app.env == Environment::Production && self.cors.allowed_origins.contains(&"*".to_string()) {
            anyhow::bail!("CORS wildcard (*) is not allowed in production");
        }

        if self.server.port == 0 {
            anyhow::bail!("Server port must be greater than 0");
        }

        if self.database.max_connections == 0 {
            anyhow::bail!("Database max_connections must be greater than 0");
        }

        tracing::info!("Configuration validation passed");
        Ok(())
    }

    pub fn is_development(&self) -> bool {
        self.app.env == Environment::Development
    }

    pub fn is_production(&self) -> bool {
        self.app.env == Environment::Production
    }
}