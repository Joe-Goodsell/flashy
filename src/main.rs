use std::io::{self, stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use flashy::tui;
use ratatui::{prelude::*, widgets::*};

#[tracing::instrument]
#[tokio::main]
async fn main() -> io::Result<()> {
    flashy::telemetry::initialise_subscriber();
    tracing::info!("TESTING TELEMETRY");
    enable_raw_mode()?;
    let app = tui::tui_run::App::default();
    let _result = app.run().await;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
