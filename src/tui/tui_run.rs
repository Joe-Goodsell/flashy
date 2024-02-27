use super::event_handler::{self, Event};
use super::utils::Tui;
use color_eyre::eyre;
use crossterm::event::KeyCode::Char;
use ratatui::{widgets::{Block, Borders, Paragraph}, Frame};

// Stores application state; for now just a simple u8 counter
#[derive(Debug, Default)]
pub struct App {
    pub counter: u8,
    pub should_quit: bool,
}


impl App {
    fn update(&mut self, event: Event) -> eyre::Result<()> {
        if let Event::Key(key) = event {

            match key.code {
                Char('j') => self.counter += 1,
                Char('k') => self.counter -= 1,
                Char('q') => self.should_quit = true,
                _ => {},
            }
        }
        Ok(())
    }

    pub async fn run(mut self, mut term: Tui) -> eyre::Result<()> {
        // Event handler
        let mut events = event_handler::EventHandler::default();

        while !self.should_quit {
            // Check events
            let event = events.next().await?;

            // Update application state
            self.update(event)?;

            // Render
            term.draw(|f| {
                ui(f, &self);
            })?;
        }
        Ok(())
    }


}

fn ui(frame: &mut Frame, app: &App) {
    frame.render_widget(
        Paragraph::new("Hello World!")
            .block(Block::default().title("Greeting").borders(Borders::ALL)),
        frame.size(),
    );
}
