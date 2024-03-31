use flashy::{
    configuration::{self, Settings},
    startup,
    tui::{
        app::App,
        utils::*,
    },
};
use sqlx::PgPool;
use std::io;

#[tracing::instrument]
#[tokio::main]
async fn main() -> io::Result<()> {
    // TELEMETRY
    flashy::telemetry::initialise_subscriber();
    tracing::info!("TESTING TELEMETRY");

    // INTIALISE CONFIGURATION
    let config: Settings = configuration::get_config().expect("Failed to get configuration");

    // DATABASE
    let pg_pool: PgPool = startup::acquire_pg_pool(&config.database);

    // INITIALISE APP & TERMINAL
    let term = init().expect("Failed to intialise terminal");
    let app = App::new(pg_pool);

    // RUN
    let _result = app.run(term).await;

    // CLEANUP
    let _ = restore();
    Ok(())
}
