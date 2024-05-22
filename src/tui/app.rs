use super::event_handler::{self, Event};
use super::panes::alertpopup::{AlertPopup, AlertPriority};
use super::panes::confirm::{ConfirmAction, ConfirmPopup};
use super::screens::create_card::CreateCard;
use super::screens::create_deck::CreateDeck;
use super::{
    utils,
    utils::{Searcher, Tui},
};
use crate::domain::card::Card;
use crate::domain::deck::Deck;
use crate::domain::deckset::DeckSet;
use crate::tui::panes::statusbar::StatusBar;
use color_eyre::eyre;
use crossterm::event::KeyCode;
use crossterm::event::KeyCode::Char;
use std::fmt::Display;

// UI
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Borders, List, ListState, Padding, Paragraph, Widget,
    },
};

// BACKEND
use sqlx::PgPool;

#[derive(Debug, Default)]
pub enum CurrentScreen {
    DECKS,
    CARDS,
    CreateCard,
    CreateDeck,
    CONFIRM(ConfirmPopup),
    REVIEW,
    #[default]
    WELCOME,
}

#[derive(Debug, Default, Clone)]
pub enum Mode {
    #[default]
    NORMAL,
    INSERT,
    SEARCH(Searcher),
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_rep = match self {
            Mode::NORMAL => "NORMAL",
            Mode::INSERT => "INSERT",
            Mode::SEARCH(_) => "SEARCH",
        };
        f.write_str(str_rep)
    }
}

// Stores application state
#[derive(Debug)]
pub struct App<'a> {
    current_screen: CurrentScreen,

    // Persistent UI elements
    create_screen: Option<CreateCard<'a>>,
    create_deck: Option<CreateDeck>,
    statusbar: Option<StatusBar>,
    alert: Option<AlertPopup<'a>>, // always appears in top-right (floating)

    mode: Mode,
    should_quit: bool,

    cursor: Option<(usize, usize)>,

    // I don't want to clone the Deck, but don't know how to avoid it...?
    deck: Option<Deck>,
    deckset: Option<DeckSet>,
    db_pool: PgPool, // TODO: should be optional?
    current_list: Vec<String>,
    pointer: ListState,
    n_items: usize, // number of items, e.g. list items, currently displayed
}

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate area for main pane and status bar
        // And render statusbar if it exists
        let main_area = match &mut self.statusbar {
            Some(statusbar) => {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(100), Constraint::Min(1)])
                    .margin(1)
                    .split(area);

                let (main_area, statusbar_area) = (layout[0], layout[1]);
                statusbar.mode = self.mode.clone();
                statusbar.render(statusbar_area, buf);
                main_area
            }
            // If there's no statusbar to render, the "main area" is just "area"
            None => area,
        };

        self.n_items = 0usize;

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
                    format!("[ CARDS IN {} ]", self.deck.as_ref().unwrap().name.clone()).bold(),
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
                                .map(|card| {
                                    if let Some(text) = card.front_text.clone() {
                                        text
                                    } else {
                                        tracing::warn!("invalid card, no text");
                                        "".to_string()
                                    }
                                })
                                .collect(),
                            None => Vec::new(),
                        }
                    }
                    None => Vec::new(),
                };

                self.current_list = cards.clone();
                self.n_items = self.current_list.len();
                let list_text = utils::add_nums_to_text(cards);
                if list_text.is_empty() {
                    self.alert = Some(AlertPopup::new(
                        std::time::Duration::new(5, 0),
                        "Warn: No cards in deck.".to_string(),
                        AlertPriority::Yellow,
                    ));
                }
                let list = utils::styled_list(list_text, block);

                ratatui::widgets::StatefulWidget::render(list, main_area, buf, &mut self.pointer);
            }
            CurrentScreen::DECKS => {
                let text = match &self.mode {
                    Mode::SEARCH(searcher) => {
                        /*
                        TODO:‼️
                        There is no way to select results!
                         */
                        searcher.get_text()
                    }
                    _ => match &self.deckset {
                        Some(d) => d.decks.iter().map(|deck| deck.name.clone()).collect(),
                        None => {
                            self.alert = Some(AlertPopup::new(
                                std::time::Duration::new(5, 0),
                                "No decks found".to_string(),
                                AlertPriority::Yellow,
                            ));
                            Vec::new()
                        }
                    },
                };

                let main_area = match &self.mode {
                    Mode::NORMAL | Mode::INSERT => main_area,
                    Mode::SEARCH(searcher) => {
                        let layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints(vec![Constraint::Percentage(100), Constraint::Min(3)])
                            .split(main_area);
                        let (main_area, search_bar_area) = (layout[0], layout[1]);

                        // Render search bar
                        Paragraph::new(searcher.get_search_string())
                            .block(Block::default().borders(Borders::ALL))
                            .render(search_bar_area, buf);
                        main_area
                    }
                };

                let title = Title::from("DECKS".to_string());
                let instructions = Title::from(Line::from(vec![
                    "[ [n] to create deck, [/] to search ]".into(),
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

                let list_text = utils::add_nums_to_text(text.to_vec());
                self.n_items = list_text.len();

                let list = utils::styled_list(list_text, block);

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
                if let Some(create_screen) = &mut self.create_screen {
                    create_screen.render(area, buf);
                    tracing::info!("getting current text field for create card");
                    if let Some(current_text_field) = create_screen.current_text_field() {
                        tracing::info!("textfield exists");
                        let tf_cursor = current_text_field.get_offset_cursor();
                        if let Some(tf_coords) = current_text_field.coords() {
                            let tf_pos = tf_coords.as_position();
                            self.cursor = Some((tf_cursor.0 + tf_pos.x as usize, tf_cursor.1 + tf_pos.y as usize ));
                        }
                    }
                    // if let Some(current_text_field) = create_screen.current_text_field() {
                    //     tracing::info!("textfield exists");
                    //     let textfield = current_text_field.borrow();
                    //     let cursor_in_text_field = textfield.cursor();
                    //     if let Some(area_of_text_field) = textfield.coords() {
                    //         let position = area_of_text_field.as_position();
                    //         self.cursor = Some((cursor_in_text_field.0 + position.x as usize, cursor_in_text_field.1 + position.y as usize));
                    //         tracing::info!("cursor is: {}, {}", self.cursor.unwrap().0, self.cursor.unwrap().1);
                    //     } else {
                    //         tracing::info!("no coords exist");
                    //     }
                    //     create_screen.render(main_area, buf);
                    // } else {
                    //     tracing::info!("no textfield exists");
                    // }
                } else {
                    self.create_screen = Some(CreateCard::default());
                }
            }
            CurrentScreen::REVIEW => todo!(),
            CurrentScreen::CONFIRM(popup) => {
                popup.render(main_area, buf);
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
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            current_screen: CurrentScreen::default(),
            create_screen: None,
            create_deck: None,
            statusbar: None,
            alert: None,
            mode: Mode::default(),
            should_quit: false,
            deck: None,
            deckset: None,
            current_list: Vec::new(),
            db_pool,
            pointer: ListState::default(),
            n_items: 0usize,
            cursor: None,
        }
    }

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

    async fn update(&mut self, event: Event) -> eyre::Result<()> {
        if let Event::Key(key) = event {
            match &self.current_screen {
                // Main screen lists cards
                CurrentScreen::DECKS => match &mut self.mode {
                    Mode::NORMAL | Mode::INSERT => match &key.code {
                        Char('q') => self.should_quit = true,
                        Char('j') => {
                            tracing::info!("scrolling down cards list");
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
                        Char('d') => match &self.deckset {
                            Some(deckset) => {
                                if let Some(curr_deck) =
                                    deckset.decks.get(self.pointer.selected().unwrap_or(0usize))
                                {
                                    tracing::info!("Deleting deck: {}", curr_deck.name);
                                    let popup = ConfirmPopup {
                                            text: format!("Are you sure you want to delete deck '{}'?\n All of its cards will be also be deleted!", curr_deck.name),
                                            action: ConfirmAction::DeleteDeck(curr_deck.id),
                                        };
                                    self.current_screen = CurrentScreen::CONFIRM(popup);
                                } else {
                                    self.alert = Some(AlertPopup::new(
                                        std::time::Duration::new(5, 0),
                                        "No deck selected".to_string(),
                                        AlertPriority::Yellow,
                                    ));
                                }
                            }
                            None => {
                                self.alert = Some(AlertPopup::new(
                                    std::time::Duration::new(5, 0),
                                    "No deck selected".to_string(),
                                    AlertPriority::Yellow,
                                ));
                            }
                        },
                        KeyCode::Enter => {
                            // TODO: rewrite to use stored Uuid for deck retrieval, rather than assumign that n decks *displayed* is same as n decks (this is to achieve compatibility with selecting items when searching)
                            match &self.deckset {
                                // Check we have a deckset
                                Some(deckset) => {
                                    // Check we have a valid "pointer" to selected deck
                                    if let Some(curr_deck) =
                                        deckset.decks.get(self.pointer.selected().unwrap_or(0usize))
                                    {
                                        let mut deck = curr_deck.clone();
                                        match deck.load_cards(&self.db_pool).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                tracing::error!("failed to load cards {}", e);
                                                self.alert = Some(AlertPopup::new(
                                                    std::time::Duration::new(5, 0),
                                                    "Error: Failed to load cards in deck."
                                                        .to_string(),
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
                        KeyCode::Char('/') => {
                            tracing::info!("searching in decks");
                            self.mode = Mode::SEARCH(Searcher::new(
                                self.deckset
                                    .as_ref()
                                    .unwrap()
                                    .decks
                                    .iter()
                                    .map(|d| d.name.as_str())
                                    .collect(),
                            ));
                        }
                        _ => {}
                    },
                    Mode::SEARCH(ref mut searcher) => match &key.code {
                        KeyCode::Esc => self.mode = Mode::NORMAL,
                        KeyCode::Char(ch) => {
                            searcher.push_and_search(*ch);
                            searcher.build_index();
                            searcher.search();
                        }
                        KeyCode::Backspace => {
                            searcher.pop_and_search();
                            searcher.build_index();
                            searcher.search();
                        }
                        _ => {}
                    },
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
                        // Create new card
                        self.create_screen = Some(CreateCard::from(&Card::new_with_deck(
                            self.deck.as_ref().unwrap().id,
                        )));
                        self.current_screen = CurrentScreen::CreateCard;
                    }
                    Char('d') => {
                        if let Some(deck) = &self.deck {
                            if let Some(card) = deck
                                .cards
                                .clone()
                                .unwrap_or(Vec::<Card>::new())
                                .get(self.pointer.selected().unwrap_or(0usize))
                            {
                                self.current_screen = CurrentScreen::CONFIRM(ConfirmPopup {
                                    text: "Are you sure you want to delete this card?".to_string(),
                                    action: ConfirmAction::DeleteCard(card.id),
                                });
                                self.pointer = ListState::default();
                            } else {
                                self.alert = Some(AlertPopup::new(
                                    std::time::Duration::new(5, 0),
                                    "Warning: No card selected".to_string(),
                                    AlertPriority::Yellow,
                                ));
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(deck) = &self.deck {
                            // TODO: review this .clone()
                            if let Some(card) = deck
                                .cards
                                .clone()
                                .unwrap_or(Vec::<Card>::new())
                                .get(self.pointer.selected().unwrap_or(0usize))
                            {
                                self.current_screen = CurrentScreen::CreateCard;
                                self.pointer = ListState::default();
                                self.create_screen = Some(CreateCard::from(card));
                            } else {
                                self.alert = Some(AlertPopup::new(
                                    std::time::Duration::new(5, 0),
                                    "Warning: No card selected".to_string(),
                                    AlertPriority::Yellow,
                                ));
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
                                KeyCode::Esc => self.current_screen = CurrentScreen::DECKS,
                                _ => {}
                            },
                            Mode::INSERT => match &key.code {
                                KeyCode::Backspace => create_deck.pop_char(),
                                Char(ch) => create_deck.push_char(*ch),
                                KeyCode::Esc => self.mode = Mode::NORMAL,
                                _ => {}
                            },
                            Mode::SEARCH(_) => todo!(),
                        }
                    }
                }

                // Create screen allows creation of new flashcard
                CurrentScreen::CreateCard => {
                    if let Some(create_card) = &mut self.create_screen {
                        // but then will not be able to make popup :(
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
                                                self.alert = Some(AlertPopup::new(
                                                    std::time::Duration::new(5, 0),
                                                    "Card saved".to_string(),
                                                    AlertPriority::Green,
                                                ));
                                                match deck.load_cards(&self.db_pool).await {
                                                    Ok(_) => tracing::info!("Deck reloaded"),
                                                    Err(e) => tracing::error!(
                                                        "Failed to reload deck! {}",
                                                        e
                                                    ),
                                                };
                                            }
                                            Err(e) => {
                                                self.alert = Some(AlertPopup::new(
                                                    std::time::Duration::new(5, 0),
                                                    "Error: Failed to save card!".to_string(),
                                                    AlertPriority::Red,
                                                ));
                                                tracing::error!("failed to save card {}", e);
                                            }
                                        }
                                    };
                                }
                                KeyCode::Esc => self.current_screen = CurrentScreen::CARDS,
                                _ => {}
                            },
                            Mode::INSERT => {
                                if let Some(textfield) = create_card.current_text_field() {
                                    match &key.code {
                                        KeyCode::Esc => self.mode = Mode::NORMAL,
                                        KeyCode::Backspace => textfield.backspace(),
                                        KeyCode::Enter => textfield.insert_newline(),
                                        Char(ch) => textfield.insert_char(*ch),
                                        _ => {} //TODO:
                                    }
                                }
                            }
                            Mode::SEARCH(_) => todo!(),
                        }
                    }
                }
                CurrentScreen::WELCOME => {
                    // Create statusbar once we're past the splash screen
                    self.statusbar = Some(StatusBar::default());
                    self.current_screen = CurrentScreen::DECKS;
                }
                CurrentScreen::REVIEW => {
                    todo!()
                }
                CurrentScreen::CONFIRM(popup) => match &key.code {
                    KeyCode::Char('y') => match popup.action {
                        ConfirmAction::DeleteCard(card_id) => {
                            Card::delete_from_db(&self.db_pool, card_id)
                                .await
                                .expect("failed to delete card from db");
                            if let Some(deck) = &mut self.deck {
                                deck.load_cards(&self.db_pool)
                                    .await
                                    .expect("failed to reload deck");
                            };
                            self.alert = Some(AlertPopup::new(
                                std::time::Duration::new(5, 0),
                                "Deleted card from database".to_string(),
                                AlertPriority::Green,
                            ));
                            self.current_screen = CurrentScreen::CARDS;
                        }
                        ConfirmAction::DeleteDeck(deck_id) => {
                            if let Some(deckset) = &mut self.deckset {
                                deckset
                                    .delete_deck_with_cards(&self.db_pool, deck_id)
                                    .await
                                    .expect("Failed to delete deck from db");
                                self.alert = Some(AlertPopup::new(
                                    std::time::Duration::new(5, 0),
                                    "Deleted deck from database".to_string(),
                                    AlertPriority::Green,
                                ));
                            }
                            self.current_screen = CurrentScreen::DECKS;
                        }
                    },
                    KeyCode::Char('n') | KeyCode::Esc => {
                        self.current_screen = match popup.action {
                            ConfirmAction::DeleteCard(_) => CurrentScreen::CARDS,
                            ConfirmAction::DeleteDeck(_) => CurrentScreen::DECKS,
                        };
                    }
                    _ => {}
                },
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
                tracing::error!("COULD NOT FETCH DECKS WITH ERROR {}", e);
            }
        };

        while !self.should_quit {
            // Poll events
            let event = events.next().await?;

            // Update application state
            self.update(event).await?;

            // Render
            // Must only call `draw()` once per pass; should render whole frame
            term.draw(|f| {
                tracing::info!("setting cursor...");
                if let Some(cursor) = self.cursor {
                    tracing::info!("cursor exists");
                    // TODO: cursor coordinates are flipped
                    f.set_cursor(cursor.0 as u16, cursor.1 as u16);
                } else {
                    tracing::info!("no cursor found!");
                };
                f.render_widget(&mut self, f.size());
            })?;
        }
        Ok(())
    }
}
