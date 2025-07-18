use std::{path::PathBuf, sync::{Arc, RwLock}};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
    pub path: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub log: LogConfig,
}


#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PartialServerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PartialDatabaseConfig {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PartialLogConfig {
    pub level: Option<String>,
    pub path: Option<String>
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PartialAppConfig {
    pub server: Option<PartialServerConfig>,
    pub database: Option<PartialDatabaseConfig>,
    pub log: Option<PartialLogConfig>,
}

impl AppConfig {
    // 载入配置，支持加载 .env 和 toml 文件
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        // 计算workspace根路径：web/的上级的上级
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let base_path = manifest_dir.parent().unwrap().to_path_buf();

        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

        let default_path = base_path.join("config").join("default.toml");
        let env_path = base_path.join("config").join(format!("{}.toml", env));

        let builder = config::Config::builder()
            .add_source(config::File::from(default_path).required(true))
            .add_source(config::File::from(env_path).required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        let settings = builder.build().expect("Failed to build config");

        settings.try_deserialize().expect("Failed to deserialize config")
    }

    pub fn merge(&self, user: &PartialAppConfig) -> AppConfig {
        AppConfig {
            server: ServerConfig {
                host: user
                    .server
                    .as_ref()
                    .and_then(|s| s.host.clone())
                    .unwrap_or_else(|| self.server.host.clone()),
                port: user
                    .server
                    .as_ref()
                    .and_then(|s| s.port)
                    .unwrap_or(self.server.port),
            },
            database: DatabaseConfig {
                url: user
                    .database
                    .as_ref()
                    .and_then(|d| d.url.clone())
                    .unwrap_or_else(|| self.database.url.clone()),
            },
            log: LogConfig {
                level: user
                    .log
                    .as_ref()
                    .and_then(|l| l.level.clone())
                    .unwrap_or_else(|| self.log.level.clone()),
                path: user
                    .log
                    .as_ref()
                    .and_then(|l| l.path.clone())
                    .unwrap_or_else(|| self.log.path.clone()),
            },
        }
    }
}

pub struct SharedConfig {
    default: AppConfig,
    user: Arc<RwLock<PartialAppConfig>>,
}

impl SharedConfig {
    pub fn new(default: AppConfig) -> Self {
        Self {
            default,
            user: Arc::new(RwLock::new(PartialAppConfig::default())),
        }
    }

    pub fn get(&self) -> AppConfig {
        let user_cfg = self.user.read().unwrap();
        self.default.merge(&user_cfg)
    }
    #[allow(dead_code)]
    pub fn set_user_config(&self, partial: PartialAppConfig) {
        let mut w = self.user.write().unwrap();
        *w = partial;
    }
    #[allow(dead_code)]
    pub fn reset_user_config(&self) {
        let mut w = self.user.write().unwrap();
        *w = PartialAppConfig::default();
    }
}
