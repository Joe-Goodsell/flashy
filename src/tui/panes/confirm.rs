use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use uuid::Uuid;

use crate::tui::{app::CurrentScreen, utils::{create_centred_rect_by_percent, create_centred_rect_by_size}};


#[derive(Debug)]
// `ConfirmAction` contains an action type and the relevant
// info required to perform that action (e.g., a deck Id)
pub enum ConfirmAction {
    DeleteCard(Uuid),
    DeleteDeck(Uuid),
}

#[derive(Debug)]
pub struct ConfirmPopup {
    pub text: String,
    pub action: ConfirmAction,
}

impl Widget for &ConfirmPopup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
            let popup_area = create_centred_rect_by_percent(40u16, 20u16, area);

            Paragraph::new(format!("{} [y/n]", self.text.clone()))
                .block(Block::default().borders(Borders::ALL).title("Confirm Action"))
                .render(popup_area, buf);
    }
}