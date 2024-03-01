use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidget, Widget},
};

use sqlx::PgPool;

use crate::{
    domain::new_card::NewCard,
    tui::{app::Mode, utils::create_centred_rect},
};

const TEXTBOX_STYLE_EDITING: Style = Style::new().bg(Color::Yellow);
const TEXTBOX_STYLE_VIEWING: Style = Style::new().bg(Color::LightBlue);

#[derive(Debug, Clone)]
pub struct CreateCard {
    pub front_field: String,
    pub back_field: String,
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
            front_field: "FRONT".to_string(),
            back_field: "BACK".to_string(),
            mode: Mode::default(),
            state: CurrentlyEditing::default(),
        }
    }
}

impl StatefulWidget for &CreateCard {
    type State = CurrentlyEditing;
    /// Intended to render a popup window with fields `Front Text`, `Back Text`
    /// And a select menu for an existing Deck (TODO:)
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let popup_area: Rect = create_centred_rect(50u16, 50u16, area);

        let block = Block::default()
            .title(Span::styled(
                "Add a card",
                Style::default().fg(Color::Yellow).bg(Color::Black),
            ))
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let text_fields = Layout::default()
            .direction(Direction::Horizontal)
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
                Span::styled(self.front_field.clone(), TEXTBOX_STYLE_EDITING),
                Span::styled(self.back_field.clone(), TEXTBOX_STYLE_VIEWING),
            ),
            CurrentlyEditing::BackText => (
                Span::styled(self.front_field.clone(), TEXTBOX_STYLE_VIEWING),
                Span::styled(self.back_field.clone(), TEXTBOX_STYLE_EDITING),
            ),
            CurrentlyEditing::Saving => (
                Span::styled(self.front_field.clone(), TEXTBOX_STYLE_VIEWING),
                Span::styled(self.back_field.clone(), TEXTBOX_STYLE_VIEWING),
            ),
        };

        //POPUP
        Paragraph::default().block(block).render(popup_area, buf);
        //FRONT_TEXT
        Paragraph::new(front_text)
            .block(text_field_block.clone())
            .render(text_fields[0], buf);
        //BACK_FRONT
        Paragraph::new(back_text)
            .block(text_field_block)
            .render(text_fields[1], buf);
    }
}

impl CreateCard {
    //TODO: implement saving to db

    pub async fn try_save(&self, db_pool: &PgPool) -> Result<(), sqlx::Error> {
        let card = NewCard::try_from(&self).unwrap();
        tracing::info!("SAVING: {:?}", card);
        card.save(db_pool).await
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
            CurrentlyEditing::FrontText => self.front_field.push(ch),
            CurrentlyEditing::BackText => self.back_field.push(ch),
            CurrentlyEditing::Saving => {}
        }
    }

    pub fn pop_char(&mut self) {
        //TODO: rewrite to store current cursor location
        match self.state {
            CurrentlyEditing::FrontText => _ = self.front_field.pop(),
            CurrentlyEditing::BackText => _ = self.back_field.pop(),
            _ => {}
        }
    }
}
