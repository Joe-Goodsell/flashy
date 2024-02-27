use super::event_handler::{self, Event};
use color_eyre::eyre;
use crossterm::event::KeyCode::Char;
use ratatui::{backend::CrosstermBackend, widgets::{Block, Borders, Paragraph}, Frame, Terminal};

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

    pub async fn run(mut self) -> eyre::Result<()> {
        // Event handler
        let mut events = event_handler::EventHandler::default();

        // ratatui terminal
        let mut t = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

        loop {
            let event = events.next().await?;

            self.update(event)?;

            t.draw(|f| {
                ui(f, &self);
            })?;

            if self.should_quit {
                break;
            }
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
