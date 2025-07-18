use sqlx::Pool;

use crate::config::SharedConfig;

pub struct Context {
    pub pool: Pool<sqlx::Sqlite>,
    #[allow(dead_code)]
    pub shared_config: SharedConfig,
}