use super::event_handler::{self, Event};
use super::screens::create_card::{self, CreateCard, CurrentlyEditing};
use super::screens::main_screen::MainScreen;
use super::screens::statusbar::StatusBar;
use super::utils::Tui;
use crate::domain::deck::Deck;
use color_eyre::eyre;
use crossterm::event::KeyCode::Char;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use std::fmt::Display;

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
        Block, Borders, Padding, Paragraph, Widget,
    },
    Frame,
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

#[derive(Debug, Default, Clone)]
pub enum Mode {
    #[default]
    NORMAL,
    INSERT,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_rep = match self {
            Mode::NORMAL => "NORMAL",
            Mode::INSERT => "INSERT",
        };
        f.write_str(str_rep)
    }
}

// Stores application state
#[derive(Debug)]
pub struct App {
    pub current_screen: CurrentScreen,
    pub create_screen: Option<CreateCard>,
    pub mode: Mode,
    pub should_quit: bool,
    pub deck: Option<Deck>,
    pub db_pool: PgPool, // TODO: should be optional?
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Min(1)])
            .margin(1)
            .split(area);

        let (main_area, statusbar_area) = (layout[0], layout[1]);

        // Renders StatusBar
        StatusBar::new(self.mode.clone()).render(statusbar_area, buf);

        match &self.current_screen {
            CurrentScreen::Main => {
                let title = Title::from(" Flashy ".bold());
                let instructions = Title::from(Line::from(vec!["Do something".into()]));

                let block = Block::default()
                    .title(title.alignment(Alignment::Center))
                    .title(
                        instructions
                            .alignment(Alignment::Center)
                            .position(Position::Bottom),
                    )
                    .borders(Borders::ALL)
                    .border_set(border::THICK);

                let cards: Vec<Line> = match &self.deck {
                    Some(d) => d
                        .iter()
                        .map(|c| Line::from(c.front_text.clone().unwrap_or("".to_string())))
                        .collect(),
                    _ => vec![Line::from("DECK IS EMPTY")],
                };

                let counter_text = Text::from(cards);

                Paragraph::new(counter_text)
                    .block(block)
                    .centered()
                    .render(main_area, buf);
            }
            CurrentScreen::Create => {
                let mut state = CurrentlyEditing::FrontText;

                if let Some(create_screen) = &self.create_screen {
                    create_screen.render(main_area, buf, &mut state);
                };
            }
            CurrentScreen::Welcome => {
                let title = Title::from(" Flashy ".bold());
                let instructions =
                    Title::from(Line::from(vec!["Press any key to get started".into()]));
                let block = Block::default()
                    .title(title.alignment(Alignment::Center))
                    .title(
                        instructions
                            .alignment(Alignment::Center)
                            .position(Position::Bottom),
                    )
                    .borders(Borders::ALL)
                    .border_set(border::THICK)
                    .padding(Padding::new(0, 0, 4, 0));

                let body_text = Text::from(vec![Line::from(vec![Span::styled(
                    "WELCOME TO FLASHY!",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::ITALIC)
                        .add_modifier(Modifier::RAPID_BLINK),
                )])])
                .alignment(Alignment::Center);

                Paragraph::new(body_text)
                    .block(block)
                    .centered()
                    .render(main_area, buf);
            }
            _ => {}
        }
    }
}

impl App {
    async fn fetch_deck(&mut self, db: &PgPool) -> Result<(), sqlx::Error> {
        let deck = Deck::load_from_db("default", db).await?;
        self.deck = Some(deck);
        Ok(())
    }

    async fn update(&mut self, event: Event) -> eyre::Result<()> {
        if let Event::Key(key) = event {
            match &self.current_screen {
                CurrentScreen::Main => match &key.code {
                    Char('q') => self.should_quit = true,
                    Char('e') => self.current_screen = CurrentScreen::Create,
                    KeyCode::Esc => self.current_screen = CurrentScreen::Main,
                    _ => {}
                },
                CurrentScreen::Create => {
                    if let Some(create_card) = &mut self.create_screen {
                        match self.mode {
                            Mode::NORMAL => match &key.code {
                                Char('q') => self.should_quit = true,
                                Char('i') => self.mode = Mode::INSERT,
                                KeyCode::Tab => {
                                    create_card.toggle_field();
                                }
                                KeyCode::Enter => {
                                    create_card.try_save(&self.db_pool).await;
                                }
                                KeyCode::Esc => self.current_screen = CurrentScreen::Main,
                                _ => {}
                            },
                            Mode::INSERT => {
                                #[allow(clippy::single_match)]
                                match &key.code {
                                    KeyCode::Esc => self.mode = Mode::NORMAL,
                                    KeyCode::Backspace => create_card.pop_char(),
                                    Char(ch) => create_card.push_char(*ch),
                                    _ => {} //TODO:
                                }
                            }
                        }
                    } else {
                        self.create_screen = Some(CreateCard::default()); // WARN: skippin a frame here for no reason
                    }
                }
                CurrentScreen::Welcome => self.current_screen = CurrentScreen::Main,
                _ => self.current_screen = CurrentScreen::Main,
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
            self.update(event).await?;

            // Render
            // Must only call `draw()` once per pass; should render whole frame
            term.draw(|f| self.render_frame(f))?;
            // self.handle_events()?;
        }
        Ok(())
    }

    // fn handle_events(&mut self) -> std::io::Result<()> {
    //     match event::read()? {
    //         crossterm::event::Event::Key(key_event) if key_event.kind== KeyEventKind::Press => {
    //             self.handle_key_event(key_event)
    //         },
    //         _ => {},
    //     };
    //     //TODO:
    //     Ok(())
    // }

    // fn handle_key_event(&mut self, key_event: KeyEvent) {
    //     unimplemented!()
    // }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }
}
