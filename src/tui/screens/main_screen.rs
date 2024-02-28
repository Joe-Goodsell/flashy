use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{domain::deck::Deck, tui::app::GetScreen};


#[derive(Debug)]
pub struct MainScreen {
    deck: Option<Deck>, // TODO: MainScreen shouldn't hold `Deck`
    para: ratatui::widgets::Paragraph<'static>,
}

impl Widget for MainScreen {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
        todo!()
    }
}

impl Default for MainScreen {
    fn default() -> Self {
        Self {
            deck: None,
            para: ratatui::widgets::Paragraph::new("hello"),
        }
    }
}

impl MainScreen {
    pub fn new() -> Self {
        Self {
            deck: None,
            para: ratatui::widgets::Paragraph::new("hello"),
        }
    }
}

impl GetScreen for MainScreen {
    fn get_screen() -> ratatui::widgets::Paragraph<'static> {
        ratatui::widgets::Paragraph::new("hello")
    }
}