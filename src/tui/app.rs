use super::event_handler::{self, Event};
use super::screens::create_card::{CreateCard, CurrentlyEditing};
use super::utils::Tui;
use crate::domain::deck::Deck;
use super::screens::main_screen::MainScreen;
use color_eyre::eyre;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use crossterm::event::KeyCode::Char;

use ratatui::widgets::StatefulWidget;
// UI
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Padding, Widget, Block, Borders, Paragraph,
    },
    Frame
};

// BACKEND
use sqlx::PgPool;


pub trait GetScreen {
    fn get_screen() -> Paragraph<'static>; //TODO: hmmmm
}

#[derive(Debug, Default)]
pub enum CurrentScreen {
    Main,
    Create,
    #[default]
    Welcome,
}


// Stores application state
#[derive(Debug, Default)]
pub struct App{
    pub current_screen: CurrentScreen,
    pub should_quit: bool,
    pub deck: Option<Deck>,
}

// impl Default for App {
//     fn default() -> Self {
//         Self { current_screen: CurrentScreen::default(), should_quit: false, deck: None }
//     }
// }

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        //screen.render(area, buf);

        match self.current_screen {
            CurrentScreen::Main => {
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

                let cards: Vec<Line> = match &self.deck {
                    Some(d) => d.iter().map(|c| Line::from(
                        c.front_text.clone().unwrap_or("".to_string())))
                        .collect(),
                    _ => vec![Line::from("DECK IS EMPTY")],
                };

                let counter_text = Text::from(cards);

                Paragraph::new(counter_text)
                    .block(block)
                    .centered()
                    .render(area, buf);
            },
            CurrentScreen::Create => {
                let mut create_card = CreateCard::default();
                let mut state = CurrentlyEditing::FrontText;
                create_card.render(area, buf, &mut state)
            },
            CurrentScreen::Welcome => {
                let title = Title::from(" Flashy ".bold());
                let instructions = Title::from(Line::from(vec![
                    "Press any key to get started".into(),
                ]));                
                let block = Block::default()
                    .title(title.alignment(Alignment::Center))
                    .title(
                        instructions
                            .alignment(Alignment::Center)
                            .position(Position::Bottom)
                    ).borders(Borders::ALL)
                    .border_set(border::THICK).
                    padding(Padding::new(0,0,4,0)); 
                
                let body_text = Text::from(vec![Line::from(vec![
                    Span::styled("WELCOME TO FLASHY!", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD).add_modifier(Modifier::ITALIC).add_modifier(Modifier::RAPID_BLINK))])
                ]).alignment(Alignment::Center);

                Paragraph::new(body_text).block(block).centered().render(area, buf);
            },
            _ => {},
        }
    }
}


impl App {
    async fn fetch_deck(&mut self, db: &PgPool) -> Result<(), sqlx::Error> {
        let deck = Deck::load_from_db("default", db).await?;
        self.deck = Some(deck);
        Ok(())
    }


    fn update(&mut self, event: Event) -> eyre::Result<()> {
        if let Event::Key(key) = event {
            match key.code {
                Char('q') => self.should_quit = true,
                Char('e') => self.current_screen = CurrentScreen::Create,
                KeyCode::Esc => self.current_screen = CurrentScreen::Main,
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
        //TODO:
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            Char('q') => self.should_quit = true,
            Char('e') => self.current_screen = CurrentScreen::Create,
            KeyCode::Esc => self.current_screen = CurrentScreen::Main,
            _ => {},
        }
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }


}

