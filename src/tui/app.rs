use super::event_handler::{self, Event};
use super::screens::create_card::{CreateCard, CurrentlyEditing};
use super::screens::statusbar::StatusBar;
use super::utils::Tui;
use crate::domain::deck::Deck;
use crate::domain::deckset::DeckSet;
use color_eyre::eyre;
use crossterm::event::KeyCode;
use crossterm::event::KeyCode::Char;
use ratatui::layout::{Constraint, Direction, Layout};
use std::fmt::Display;
use uuid::Uuid;

use ratatui::widgets::{List, ListDirection, ListState, StatefulWidget};
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
    DECKS,
    CARDS,
    CreateCard,
    #[default]
    WELCOME,
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
    pub deckset: Option<DeckSet>,
    pub db_pool: PgPool, // TODO: should be optional?
    pub pointer: ListState,
    pub n_items: usize, // number of items, e.g. list items, currently displayed
}

impl Widget for &mut App {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Min(1)])
            .margin(1)
            .split(area);

        let (main_area, statusbar_area) = (layout[0], layout[1]);


        // seems bad
        self.n_items = 0usize;

        // Renders StatusBar
        StatusBar::new(self.mode.clone()).render(statusbar_area, buf);

        match &self.current_screen {
            CurrentScreen::CARDS => {
                let title = Title::from(format!(" CARDS IN {}", self.deck.as_ref().unwrap().name.clone()).bold());
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

                // help
                let cards: Vec<String> = match &self.deck {
                    Some(d) => { // do I have a current deck?
                        match &d.cards { // does the deck have cards?
                            Some(c) => {
                                c.iter()
                                    .map(|card| card.front_text.clone().unwrap())
                                    .collect()
                            },
                            None => vec!["NO CARDS IN DECK".to_string()],
                        }
                    },
                    None => vec!["NO DECK FOUND".to_string()],
                };

                self.n_items = cards.len();

                let list = List::new(cards)
                    .block(block)
                    // .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true);

                ratatui::widgets::StatefulWidget::render(list, area, buf, &mut self.pointer);
            }
            CurrentScreen::DECKS => {
                let title = Title::from("DECKS".to_string());
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

                // help
                let decklist: Vec<String> = match &self.deckset {
                    Some(d) => {
                        d.decks.iter().map(|deck| deck.name.clone()).collect()
                    }, 
                    None => vec!["NO DECKS FOUND".to_string()],
                };

                self.n_items = decklist.len();

                let list = List::new(decklist)
                    .block(block)
                    // .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true);

                ratatui::widgets::StatefulWidget::render(list, area, buf, &mut self.pointer);
            }
            CurrentScreen::CreateCard => {
                let mut state = CurrentlyEditing::FrontText;

                if let Some(create_screen) = &self.create_screen {
                    create_screen.render(main_area, buf, &mut state);
                };
            }
            CurrentScreen::WELCOME => {
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
    // TODO: make names clearer
    /// Fetches a `DeckSet` containing all saved decks (without loading cards)
    async fn fetch_decks(&mut self) -> Result<(), sqlx::Error> {
        match DeckSet::load_from_db(&self.db_pool).await {
            Ok(deckset) => self.deckset = Some(deckset),
            Err(e) => return Err(e),
        }
        Ok(())
    }


    /// Loads deck from name, if it doesn't exist, creates a new one named "default"
    /// And saves to DB.
    async fn fetch_deck(&mut self, name: &str) -> Result<(), sqlx::Error> {
        match Deck::new_from_db(name, &self.db_pool).await {
            Ok(deck) => self.deck = Some(deck),
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    tracing::info!("Deck not found, creating new one.");
                    let new_deck = Deck::default();
                    new_deck.save_to_db(&self.db_pool).await?;
                    self.deck = Some(new_deck);
                }
                _ => return Err(e),
            },
        }
        Ok(())
    }

    async fn update(&mut self, event: Event) -> eyre::Result<()> {
        if let Event::Key(key) = event {
            match &self.current_screen {
                // Main screen lists cards
                CurrentScreen::DECKS => match &key.code {
                    Char('q') => self.should_quit = true,
                    Char('e') => self.current_screen = CurrentScreen::CreateCard,
                    Char('j') => {
                        // how to set back to None?
                        if self.n_items != 0 {
                            let selected = match self.pointer.selected() {
                                Some(val) => {
                                    if val < self.n_items - 1 {
                                        val + 1
                                    } else {
                                        val
                                    }
                                },
                                None => 0usize,
                            };
                            self.pointer.select(Some(selected));
                        }
                    },
                    Char('k') => {
                        if let Some(val) = self.pointer.selected() {
                            self.pointer.select(Some(val.saturating_sub(1)));
                        }
                    },
                    _ => {}
                },

                // Create screen allows creation of new flashcard
                CurrentScreen::CreateCard => {
                    if let Some(create_card) = &mut self.create_screen {
                        match self.mode {
                            Mode::NORMAL => match &key.code {
                                Char('q') => self.should_quit = true,
                                Char('i') => self.mode = Mode::INSERT,
                                KeyCode::Tab => {
                                    create_card.toggle_field();
                                }
                                KeyCode::Enter => {
                                    match &self.deck {
                                        Some(deck) => {
                                            match create_card.try_save(&self.db_pool, deck.id).await
                                            {
                                                Ok(_) => {
                                                    self.deck = Some(
                                                        Deck::new_from_db(
                                                            "default",
                                                            &self.db_pool,
                                                        )
                                                        .await?,
                                                    );
                                                    self.current_screen = CurrentScreen::CARDS;
                                                    self.create_screen = None;
                                                }
                                                Err(e) => {
                                                    // TODO: implement error popup
                                                    tracing::error!("Error saving card: {}", e);
                                                }
                                            }
                                        }
                                        _ => (),
                                    };
                                }
                                KeyCode::Esc => self.current_screen = CurrentScreen::CARDS,
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
                CurrentScreen::WELCOME => self.current_screen = CurrentScreen::DECKS,
                _ => self.current_screen = CurrentScreen::DECKS,
            }
        }
        Ok(())
    }

    pub async fn run(mut self, mut term: Tui) -> eyre::Result<()> {
        // Event handler
        let mut events = event_handler::EventHandler::default();

        match self.fetch_decks().await {
            Ok(_) => {}
            Err(e) => {
                // TODO: HANDLE ERROR PROPERLY
                tracing::info!("COULD NOT FETCH DECKS WITH ERROR {}", e);
            }
        };

        while !self.should_quit {
            // Check events
            let event = events.next().await?;

            // Update application state
            self.update(event).await?;

            // Render
            // Must only call `draw()` once per pass; should render whole frame
            term.draw(|f| f.render_widget(&mut self, f.size()))?;
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
}
