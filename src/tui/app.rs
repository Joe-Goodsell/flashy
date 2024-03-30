use super::event_handler::{self, Event};
use super::panes::alertpopup::{AlertPopup, AlertPriority};
use super::screens::create_card::{CreateCard, CurrentlyEditing};
use super::screens::create_deck::CreateDeck;
use super::utils::Tui;
use crate::domain::card::Card;
use crate::domain::deck::Deck;
use crate::domain::deckset::DeckSet;
use crate::tui::panes::statusbar::StatusBar;
use color_eyre::eyre;
use crossterm::event::KeyCode;
use crossterm::event::KeyCode::Char;
use ratatui::layout::{Constraint, Direction, Layout};
use std::fmt::Display;


use ratatui::widgets::{List, ListState, StatefulWidget};
// UI
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Borders, Padding, Paragraph, Widget,
    },
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
    CreateDeck,
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
pub struct App<'a> {
    pub current_screen: CurrentScreen,

    // TODO: look up ratatui docs to see if this is right approach
    pub create_screen: Option<CreateCard>,
    pub create_deck: Option<CreateDeck>,
    pub alert: Option<AlertPopup<'a>>, // always appears in top-right (floating)

    pub mode: Mode,
    pub should_quit: bool,

    // I don't want to clone the Deck, but don't know how to avoid it...?
    pub deck: Option<Deck>,
    pub deckset: Option<DeckSet>,
    pub db_pool: PgPool, // TODO: should be optional?
    pub pointer: ListState,
    pub n_items: usize, // number of items, e.g. list items, currently displayed
}

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
            CurrentScreen::WELCOME => {
                let splash: String = r#"
        _________  __        _________  __________  __     __  __    __
       / ______ / / /       / _____  / / ________/ / /    / / / /   / /
     / /_____   / /       / /    / / / /_______  / /____/ / / /___/ /
   / ______/  / /       / /    / / /______   / / _____  / /_____  /
 / /        / /_____  / /    / / _______/  / / /    / / ______/ /
/_/        /_______/ /_/    /_/ /_________/ /_/    /_/ /_______/
                "#
                .to_string();
                // BUG: this is for some reason not aligned
                let lines: Vec<Line> = splash.split('\n').map(Line::from).collect();

                let instructions =
                    Title::from(Line::from(vec!["[ Press any key to get started ]".into()]));
                let block = Block::default()
                    .title(
                        instructions
                            .alignment(Alignment::Center)
                            .position(Position::Bottom),
                    )
                    .borders(Borders::ALL)
                    .border_set(border::THICK)
                    .padding(Padding::new(0, 0, 4, 0));

                Paragraph::new(lines)
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(block)
                    .centered()
                    .render(main_area, buf);
            }
            CurrentScreen::CARDS => {
                let title = Title::from(
                    format!(" CARDS IN {}", self.deck.as_ref().unwrap().name.clone()).bold(),
                );
                let instructions =
                    Title::from(Line::from(vec!["[ [n] to create new card ]".into()]));

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
                    Some(d) => {
                        // do I have a current deck?
                        match &d.cards {
                            // does the deck have cards?
                            Some(c) => c
                                .iter()
                                .map(|card| card.front_text.clone().unwrap())
                                .collect(),
                            None => Vec::new(),
                        }
                    }
                    None => Vec::new(),
                };

                let nums_col_width = 4usize;
                let nums: Vec<String> = (1..cards.len() + 1).map(|val| val.to_string()).collect();
                let nums: Vec<String> = nums
                    .iter()
                    .map(|s| format!("{}{}", s, " ".repeat(nums_col_width - s.len())))
                    .collect();
                let list_text: Vec<String> = nums
                    .iter()
                    .zip(cards.iter())
                    .map(|(a,b)| format!("{} {}", a, b))
                    .collect();

                if list_text.is_empty() {
                    self.alert = Some(AlertPopup::new(
                        std::time::Duration::new(5, 0),
                        "Warn: No cards in deck.".to_string(),
                        AlertPriority::Yellow,
                    ));
                }
                self.n_items = list_text.len();

                let list = List::new(list_text)
                    .block(block)
                    .highlight_symbol(">> ")
                    .repeat_highlight_symbol(true);

                ratatui::widgets::StatefulWidget::render(list, main_area, buf, &mut self.pointer);
            }
            CurrentScreen::DECKS => {
                let title = Title::from("DECKS".to_string());
                let instructions = Title::from(Line::from(vec![
                    "[ [n] to create deck, [b] to go back ]".into(),
                ]));

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
                    Some(d) => d.decks.iter().map(|deck| deck.name.clone()).collect(),
                    None => {
                        self.alert = Some(AlertPopup::new(
                            std::time::Duration::new(5, 0),
                            "No decks found".to_string(),
                            AlertPriority::Yellow,
                        ));
                        Vec::new()
                    }
                };

                // TODO: factor out
                let nums_col_width = 4usize;
                let nums: Vec<String> = (1..decklist.len() + 1).map(|val| val.to_string()).collect();
                let nums: Vec<String> = nums
                    .iter()
                    .map(|s| format!("{}{}", s, " ".repeat(nums_col_width - s.len())))
                    .collect();
                let list_text: Vec<String> = nums
                    .iter()
                    .zip(decklist.iter())
                    .map(|(a,b)| format!("{} {}", a, b))
                    .collect();
                self.n_items = decklist.len();

                let list = List::new(list_text)
                    .block(block)
                    // .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true);

                ratatui::widgets::StatefulWidget::render(list, main_area, buf, &mut self.pointer);
            }

            CurrentScreen::CreateDeck => {
                if let Some(create_deck) = &self.create_deck {
                    create_deck.render(main_area, buf);
                } else {
                    self.create_deck = Some(CreateDeck::default());
                }
            }

            CurrentScreen::CreateCard => {
                let mut state = CurrentlyEditing::FrontText;

                if let Some(create_screen) = &self.create_screen {
                    create_screen.render(main_area, buf, &mut state);
                } else {
                    self.create_screen = Some(CreateCard::default());
                }
            }
        }

        // Renders top-right 'alert' popup, and sets to None when times out
        if let Some(alert) = &self.alert {
            alert.render(main_area, buf);
            if !alert.is_valid() {
                self.alert = None;
            }
        };
    }
}

impl<'a> App<'a> {
    // TODO: make names clearer
    /// Fetches a `DeckSet` containing all saved decks (without loading cards)
    async fn fetch_decks(&mut self) -> Result<(), sqlx::Error> {
        match DeckSet::load_from_db(&self.db_pool).await {
            Ok(deckset) => self.deckset = Some(deckset),
            Err(e) => return Err(e),
        }
        self.alert = Some(AlertPopup::new(
            std::time::Duration::new(5, 0),
            "Decks loaded successfully".to_string(),
            AlertPriority::Green,
        ));
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
                                }
                                None => 0usize,
                            };
                            self.pointer.select(Some(selected));
                        }
                    }
                    Char('k') => {
                        if let Some(val) = self.pointer.selected() {
                            self.pointer.select(Some(val.saturating_sub(1)));
                        }
                    }
                    Char('n') => {
                        // Create new deck
                        self.current_screen = CurrentScreen::CreateDeck;
                    }
                    KeyCode::Enter => {
                        // TODO: go to `cards` screen for current deck
                        match &self.deckset {
                            // Check we have a deckset
                            Some(deckset) => {
                                // Check we have a valid "pointer" to selected deck
                                if let Some(curr_deck) =
                                    deckset.decks.get(self.pointer.selected().unwrap_or(0usize))
                                {
                                    let mut deck = curr_deck.clone();
                                    match deck.load_cards(&self.db_pool).await {
                                        // TODO: refactor alert generator
                                        Ok(_) => {}
                                        Err(e) => {
                                            tracing::error!("failed to load cards {}", e);
                                            self.alert = Some(AlertPopup::new(
                                                std::time::Duration::new(5, 0),
                                                "Error: Failed to load cards in deck.".to_string(),
                                                AlertPriority::Red,
                                            ));
                                        }
                                    };
                                    self.deck = Some(deck);
                                    // Set ListState to default
                                    self.pointer = ListState::default(); 
                                    self.current_screen = CurrentScreen::CARDS;
                                };
                            }
                            None => {}
                        }
                    }
                    _ => {}
                },

                // DISPLAY CARDS
                CurrentScreen::CARDS => match &key.code {
                    Char('q') => self.should_quit = true,
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
                                }
                                None => 0usize,
                            };
                            self.pointer.select(Some(selected));
                        }
                    }
                    Char('k') => {
                        if let Some(val) = self.pointer.selected() {
                            self.pointer.select(Some(val.saturating_sub(1)));
                        }
                    }
                    Char('b') => {
                        self.current_screen = CurrentScreen::DECKS;
                    }
                    Char('n') => {
                        // Create new deck
                        self.current_screen = CurrentScreen::CreateCard;
                    }
                    KeyCode::Enter => {
                        if let Some(deck) = &self.deck {
                            // TODO: review this .clone()
                            if let Some(card) = deck.cards.clone().unwrap_or(Vec::<Card>::new()).get(self.pointer.selected().unwrap_or(0usize)) {
                                self.current_screen = CurrentScreen::CreateCard;
                                self.pointer = ListState::default();
                                self.create_screen = Some(CreateCard::from(card));
                            } else {
                                self.alert = Some(
                                    AlertPopup::new(
                                        std::time::Duration::new(5, 0),
                                        "Warn: No card selected".to_string(), 
                                        AlertPriority::Yellow
                                    )
                                );
                            }
                        }
                    }
                    _ => {}
                },
                // CREATE NEW DECK
                CurrentScreen::CreateDeck => {
                    if let Some(create_deck) = &mut self.create_deck {
                        match self.mode {
                            Mode::NORMAL => match &key.code {
                                KeyCode::Enter => match create_deck.try_save(&self.db_pool).await {
                                    Ok(_) => {
                                        self.current_screen = CurrentScreen::DECKS;
                                        self.create_deck = None;
                                        self.fetch_decks().await.unwrap();
                                        self.alert = Some(AlertPopup::new(
                                            std::time::Duration::new(5, 0),
                                            "Saved deck".to_string(),
                                            AlertPriority::Green,
                                        ));
                                    }
                                    Err(e) => {
                                        self.alert = Some(AlertPopup::new(
                                            std::time::Duration::new(5, 0),
                                            "Failed to save deck!".to_string(),
                                            AlertPriority::Red,
                                        ));
                                        tracing::error!("Error saving deck: {}", e);
                                    }
                                },
                                Char('i') => self.mode = Mode::INSERT,
                                Char('q') => self.should_quit = true,
                                Char('d') => todo!(), // TODO: impl delete deck
                                KeyCode::Esc => self.current_screen = CurrentScreen::DECKS,
                                _ => {}
                            },
                            Mode::INSERT => match &key.code {
                                KeyCode::Backspace => create_deck.pop_char(),
                                Char(ch) => create_deck.push_char(*ch),
                                KeyCode::Esc => self.mode = Mode::NORMAL,
                                _ => {}
                            },
                        }
                    }
                }

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
                                    if let Some(deck) = &mut self.deck {
                                        match create_card.try_save(&self.db_pool).await {
                                            Ok(_) => {
                                                self.current_screen = CurrentScreen::CARDS;
                                                self.create_screen = None;
                                                match deck.load_cards(&self.db_pool).await {
                                                    Ok(_) => tracing::info!("Deck reloaded"),
                                                    Err(e) => tracing::error!(
                                                        "Failed to reload deck! {}",
                                                        e
                                                    ),
                                                };
                                            }
                                            Err(e) => {
                                                // TODO: implement error popup
                                                tracing::error!("Error saving card: {}", e);
                                            }
                                        }
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
                    }
                }
                CurrentScreen::WELCOME => self.current_screen = CurrentScreen::DECKS,
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
