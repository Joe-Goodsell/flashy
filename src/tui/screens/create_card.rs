use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidget, Widget},
};

use sqlx::PgPool;

use crate::{
    domain::card::Card,
    tui::{app::Mode, utils::create_centred_rect_by_percent},
};

const TEXTBOX_STYLE_EDITING: Style = Style::new().fg(Color::Yellow);
const TEXTBOX_STYLE_VIEWING: Style = Style::new().fg(Color::LightBlue);

#[derive(Debug, Clone)]
pub struct CreateCard {
    pub card: Card,
    pub mode: Mode,
    pub state: CurrentlyEditing,
}

#[derive(Default, Debug, Clone)]
pub enum CurrentlyEditing {
    #[default]
    FrontText,
    BackText,
    Saving,
}



/// TODO: TESTING PURPOSES
impl Default for CreateCard {
    fn default() -> Self {
        CreateCard {
            card: Card::new(),
            mode: Mode::default(),
            state: CurrentlyEditing::default(),
        }
    }
}

impl From<&Card> for CreateCard {
    fn from(card: &Card) -> Self {
        // WARN: this doesn't account for "reloading" the card stored in the `App`
        Self {
            card: card.clone(),
            mode: Mode::default(),
            state: CurrentlyEditing::default(),
        }
    }
}

impl Widget for &CreateCard {
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

        let (front_text, back_text) = match self.state {
            CurrentlyEditing::FrontText => (
                Span::styled(self.card.front_text.clone().unwrap_or_default(), TEXTBOX_STYLE_EDITING),
                Span::styled(self.card.back_text.clone().unwrap_or_default(), TEXTBOX_STYLE_VIEWING),
            ),
            CurrentlyEditing::BackText => (
                Span::styled(self.card.front_text.clone().unwrap_or_default(), TEXTBOX_STYLE_VIEWING),
                Span::styled(self.card.back_text.clone().unwrap_or_default(), TEXTBOX_STYLE_EDITING),
            ),
            CurrentlyEditing::Saving => (
                Span::styled(self.card.front_text.clone().unwrap_or_default(), TEXTBOX_STYLE_VIEWING),
                Span::styled(self.card.back_text.clone().unwrap_or_default(), TEXTBOX_STYLE_VIEWING),
            ),
        };

        //POPUP
        Paragraph::default().block(block).render(popup_area, buf);
        //FRONT_TEXT
        Paragraph::new(front_text)
            .block(text_field_block.clone().title("Front".to_string()))
            .render(text_fields[0], buf);
        //BACK_FRONT
        Paragraph::new(back_text)
            .block(text_field_block.title("Back".to_string()))
            .render(text_fields[1], buf);
    }
}

impl CreateCard {
    //TODO: implement saving to db

    pub async fn try_save(&self, db_pool: &PgPool) -> Result<(), sqlx::Error> {
        tracing::info!("SAVING: {:?}", self.card);
        self.card.save(db_pool).await
    }

    pub fn toggle_field(&mut self) {
        tracing::info!("TOGGLING CREATE CARD FIELD");
        self.state = match self.state {
            CurrentlyEditing::FrontText => CurrentlyEditing::BackText,
            _ => CurrentlyEditing::FrontText,
        };
    }

    pub fn push_char(&mut self, ch: char) {
        tracing::info!("{}", format!("pushing '{}' to {:?}", ch, self.state));
        match self.state {
            CurrentlyEditing::FrontText => {
                // TODO: is this best way?
                let mut text = self.card.front_text.clone().unwrap_or("".to_string());
                text.push(ch);
                self.card.set_front_text(text);
            },
            CurrentlyEditing::BackText => {
                let mut text = self.card.back_text.clone().unwrap_or("".to_string());
                text.push(ch);
                self.card.set_back_text(text);
            }
            CurrentlyEditing::Saving => {}
        }
    }

    pub fn pop_char(&mut self) {
        //TODO: rewrite to store current cursor location
        match self.state {
            CurrentlyEditing::FrontText => {
                let mut text = self.card.front_text.clone().unwrap_or("".to_string());
                text.pop();
                self.card.set_front_text(text);
            },
            CurrentlyEditing::BackText => {
                let mut text = self.card.back_text.clone().unwrap_or("".to_string());
                text.pop();
                self.card.set_back_text(text);
            }
            CurrentlyEditing::Saving => {}
        }
    }
}
