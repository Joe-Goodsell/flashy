use std::{borrow::BorrowMut, cell::{RefCell, RefMut}, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Padding, Paragraph, Widget},
};

use sqlx::PgPool;

use crate::{
    domain::card::Card,
    tui::{app::Mode, panes::text_field::TextField, utils::create_centred_rect_by_percent},
};

const TEXTBOX_STYLE_EDITING: Style = Style::new().fg(Color::Yellow);
const TEXTBOX_STYLE_VIEWING: Style = Style::new().fg(Color::LightBlue);

#[derive(Debug, Clone)]
pub struct CreateCard<'a> {
    pub card: Card,
    pub mode: Mode,
    pub state: CurrentlyEditing,
    pub front_text: TextField<'a>,
    pub back_text: TextField<'a>,
    // pub front_text: Rc<RefCell<TextField<'a>>>,
    // pub back_text: Rc<RefCell<TextField<'a>>>,
    pub cursor: (u16, u16),
    pub db_pool: Option<&'a PgPool>,
}

#[derive(Default, Debug, Clone)]
pub enum CurrentlyEditing {
    #[default]
    FrontText,
    BackText,
    Saving,
}


impl<'a> Default for CreateCard<'a> {
    fn default() -> Self {
        CreateCard {
            card: Card::new(),
            mode: Mode::default(),
            // front_text: Rc::new(RefCell::new(front_text)),
            // back_text: Rc::new(RefCell::new(TextField::default())),
            front_text: TextField::default(),
            back_text: TextField::default(),
            state: CurrentlyEditing::default(),
            cursor: (0u16, 0u16),
            db_pool: None,
        }
    }
}


impl<'a> From<&Card> for CreateCard<'a> {
    fn from(card: &Card) -> Self {
        // WARN: this doesn't account for "reloading" the card stored in the `App`
        Self {
            card: card.clone(),
            mode: Mode::default(),
            state: CurrentlyEditing::default(),
            front_text: TextField::from(card.front_text.clone().unwrap_or("".to_string()).as_str()),
            back_text: TextField::from(card.back_text.clone().unwrap_or("".to_string()).as_str()),
            // front_text: Rc::new(RefCell::new(TextField::from(card.front_text.clone().unwrap_or("".to_string()).as_str()))),
            // back_text: Rc::new(RefCell::new(TextField::from(card.back_text.clone().unwrap_or("".to_string()).as_str()))),
            cursor: (0u16, card.front_text.clone().unwrap_or("".to_string()).len() as u16),
            db_pool: None,
        }
    }
}

impl<'a> Widget for &mut CreateCard<'a> {
    /// Intended to render a popup window with fields `Front Text`, `Back Text`
    /// And a select menu for an existing Deck (TODO:)
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area: Rect = create_centred_rect_by_percent(50u16, 50u16, area);

        let block = Block::default()
            .title(Span::styled(
                "Add a card",
                Style::default().fg(Color::Yellow).bg(Color::Black),
            ))
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        let text_fields = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(50u16),
                Constraint::Percentage(50u16),
            ])
            .margin(2)
            .split(popup_area);
        let text_field_block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::uniform(1));

        //POPUP
        Paragraph::default().block(block).render(popup_area, buf);
        self.front_text.render(text_fields[0], buf);
        self.back_text.render(text_fields[1], buf);
        
        // let tmp_front_text = Rc::clone(&self.front_text);
        // let tmp_back_text = Rc::clone(&self.back_text);
        // {
        //     (*tmp_front_text).borrow_mut().update_coords(Some(&area));
        //     (*tmp_back_text).borrow_mut().update_coords(Some(&area));
        // }
        // let tmp_front_text = Rc::clone(&self.front_text);
        // let tmp_back_text = Rc::clone(&self.back_text);
        // {
        //     // ERROR: cloning textfield means I don't set coords!!
        //     let textfield = (*tmp_front_text).borrow().to_owned();
        //     textfield.render(text_fields[0], buf);
        //     let textfield = (*tmp_back_text).borrow();
        //     textfield.to_owned().render(text_fields[1], buf);
        // }
    }
}

impl<'a> CreateCard<'a> {
    // pub fn current_text_field(&self) -> Option<Rc<RefCell<TextField<'a>>>> {
    //     match self.state {
    //         //WARN: this is a bit wrong
    //         CurrentlyEditing::FrontText => Some(Rc::clone(&self.front_text)),
    //         CurrentlyEditing::BackText => Some(Rc::clone(&self.back_text)),
    //         _ => None,
    //     }
    // }

    pub fn current_text_field(&mut self) -> Option<&mut TextField<'a>> {
        match self.state {
            CurrentlyEditing::FrontText => Some(&mut self.front_text),
            CurrentlyEditing::BackText => Some(&mut self.back_text),
            _ => None,
        }
    }

    //TODO: implement saving to db

    pub fn set_db_pool(&mut self, db_pool: &'a PgPool) {
        self.db_pool = Some(db_pool);
    }

    pub async fn try_save(&mut self, db_pool: &PgPool) -> Result<(), sqlx::Error> {
        // WARN: text will not be updated here
        tracing::info!("SAVING: {:?}", self.card);
        // let front_text = Rc::clone(&self.front_text).borrow().to_string();
        // let back_text = Rc::clone(&self.back_text).borrow().to_string();
        let ft = self.front_text.to_string();
        let bt = self.front_text.to_string();
        self.card.front_text = Some(ft);
        self.card.back_text = Some(bt);
        self.card.save(db_pool).await
    }

    pub fn toggle_field(&mut self) {
        tracing::info!("TOGGLING CREATE CARD FIELD");
        self.state = match self.state {
            CurrentlyEditing::FrontText => CurrentlyEditing::BackText,
            _ => CurrentlyEditing::FrontText,
        };
    }

    // pub fn input(&mut self, &mut app_mode: Mode, keycode: &KeyCode) {
    //     // TODO:
    //     // current thinking: `CreateCard` mutates global app start as parameter
    //     // can pass &mut Mode to relevant text field (which also mutates)
    //     match app_mode {
    //         Mode::NORMAL => match keycode {},
    //         Mode::INSERT => match keycode {},
    //         _ => {}
    //     }
    //     match keycode {
    //         KeyCode::Tab => self.toggle_field(),
    //         _ => match self.state {
    //             CurrentlyEditing::FrontText => {
    //                 // TODO: Handle error
    //                 let _ = self.front_text.insert(keycode);
    //             }
    //             CurrentlyEditing::BackText => {
    //                 let _ = self.back_text.insert(keycode);
    //             }
    //             CurrentlyEditing::Saving => {
    //                 todo!()
    //             }
    //         },
    //     }
    // }
}
