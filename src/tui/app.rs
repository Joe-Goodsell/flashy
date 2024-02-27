use super::event_handler::{self, Event};
use super::utils::Tui;
use crate::domain::deck::Deck;
use color_eyre::eyre;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use crossterm::event::KeyCode::Char;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::Widget;
use ratatui::{widgets::{Block, Borders, Paragraph}, Frame};


#[derive(Debug, Default)]
pub enum CurrentScreen {
    #[default]
    Main,
    SelectCard,
}

// Stores application state; for now just a simple u8 counter
#[derive(Debug, Default)]
pub struct App {
    pub current_screen: CurrentScreen,
    pub counter: u8,
    pub should_quit: bool,
    pub deck: Option<Deck>,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // WARN: where does this get the terminal from??
        let title = Title::from(" Flashy ".bold());
        let instructions = Title::from(Line::from(vec![
            "Do something".into(),
        ]));

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom)
            ).borders(Borders::ALL)
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
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
            // Must only call `draw()` once per pass; should render whole frame
            term.draw(|f| {
                self.render_frame(f)
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            crossterm::event::Event::Key(key_event) if key_event.kind== KeyEventKind::Press => {
                self.handle_key_event(key_event)
            },
            _ => {},
        };
        todo!()
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.should_quit = true,
            _ => {},
        }
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }


}

fn ui(frame: &mut Frame, app: &App) {
    frame.render_widget(
        Paragraph::new("Hello World!")
            .block(Block::default().title("Greeting").borders(Borders::ALL)),
        frame.size(),
    );
}
