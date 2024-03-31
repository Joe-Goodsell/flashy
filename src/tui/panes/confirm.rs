use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::{app::CurrentScreen, utils::{create_centred_rect_by_percent, create_centred_rect_by_size}};

#[derive(Debug)]
pub struct ConfirmPopup {
    pub text: String,
    pub from: CurrentScreen,
    pub to: CurrentScreen,
}

impl Widget for &ConfirmPopup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
            let popup_area = create_centred_rect_by_size(12u16, 6u16, area);

            Paragraph::new(format!("{} [y/n]", self.text.clone()))
                .block(Block::default().borders(Borders::ALL).title("Confirm Action"))
                .render(popup_area, buf);
    }
}