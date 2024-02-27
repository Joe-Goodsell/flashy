use std::io;
use flashy::tui::{utils::*, app::App};

#[tracing::instrument]
#[tokio::main]
async fn main() -> io::Result<()> {
    // TELEMETRY
    flashy::telemetry::initialise_subscriber();
    tracing::info!("TESTING TELEMETRY");

    // INITIALISE APP & TERMINAL
    let app = App::default();
    let term = init().expect("Failed to intialise terminal");

    // RUN
    let _result = app.run(term).await;

    // CLEANUP
    let _ = restore();
    Ok(())
}
