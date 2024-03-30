use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

use sqlx::PgPool;

use crate::{
    domain::deck::Deck,
    tui::{app::Mode, utils::create_centred_rect},
};

#[derive(Debug, Clone)]
pub struct CreateDeck {
    pub name: String,
    pub mode: Mode,
}

/// TODO: TESTING PURPOSES
impl Default for CreateDeck {
    fn default() -> Self {
        CreateDeck {
            name: "".to_string(),
            mode: Mode::default(),
        }
    }
}

impl Widget for &CreateDeck {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area: Rect = create_centred_rect(50u16, 50u16, area);

        let block = Block::default()
            .title(Span::styled(
                "Add a deck",
                Style::default().fg(Color::Yellow).bg(Color::Black),
            ))
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));


        //FRONT_TEXT
        Paragraph::new(Text::from(self.name.clone()))
            .block(block)
            .render(popup_area, buf);
    }
}

impl CreateDeck {
    //TODO: implement saving to db

    pub async fn try_save(&self, db_pool: &PgPool) -> Result<(), sqlx::Error> {
        let deck = Deck::new(&self.name);
        deck.save_to_db(db_pool).await
    }
  

    pub fn push_char(&mut self, ch: char) {
        self.name.push(ch);
    }

    pub fn pop_char(&mut self) {
        self.name.pop();
    }
}
