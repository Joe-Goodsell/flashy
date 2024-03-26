use flashy::{
    configuration::{self, Settings},
    startup,
    tui::{
        app::{App, CurrentScreen, Mode},
        utils::*,
    },
};
use ratatui::widgets::ListState;
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

    // // Stores application state
    // #[derive(Debug)]
    // pub struct App {
    //     pub current_screen: CurrentScreen,
    //     pub create_screen: Option<CreateCard>,
    //     pub mode: Mode,
    //     pub should_quit: bool,
    //     pub deck: Option<Deck>,
    //     pub db_pool: PgPool, // TODO: should be optional?

    // INITIALISE APP & TERMINAL
    let term = init().expect("Failed to intialise terminal");
    let app = App {
        current_screen: CurrentScreen::default(),
        create_screen: None,
        mode: Mode::default(),
        should_quit: false,
        deck: None,
        deckset: None,
        db_pool: pg_pool,
        pointer: ListState::default(),
        n_items: 0,
    };

    // RUN
    let _result = app.run(term).await;

    // CLEANUP
    let _ = restore();
    Ok(())
}
