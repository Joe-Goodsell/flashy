use crate::configuration::DatabaseSettings;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

pub fn acquire_pg_pool(db_settings: &DatabaseSettings) -> PgPool {
    let pg_options: PgConnectOptions = db_settings.get_connect_options();
    PgPoolOptions::new().connect_lazy_with(pg_options)
}

pub fn run_app() -> Result<(), std::io::Error> {
    // TODO: how should I start the app?
    unimplemented!()
}
