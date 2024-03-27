use ratatui::{layout::Rect, text::Text, widgets::{Block, Borders, Paragraph, Widget, Wrap}};
use std::time;


#[derive(Debug)]
pub struct AlertPopup<'a> {
    pub duration: time::Duration,
    pub start: time::Instant, 
    pub text: Text<'a>,
}

impl<'a> AlertPopup<'a> {
    pub fn new(duration: time::Duration, text: Text<'a>) -> Self {
        Self {
            duration,
            start: time::Instant::now(),
            text,
        }
    }

    /// Checks if the popup should still be displayed. If not, must be disabled by caller.
    pub fn is_valid(&self) -> bool {
        (time::Instant::now() - self.start) < self.duration
    }
}

impl<'a> Widget for &AlertPopup<'a> {
    /// Renders a fixed-size alert popup in top-right corner
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
            let x = area.x / 3;
            let width = area.width - x;
            let rect = Rect::new(x, 0, width, 6);
            Paragraph::new(self.text.clone())
                .block(Block::default().title("Alert").borders(Borders::ALL))
                .wrap(Wrap { trim: true })
                .render(rect, buf);
    }
} 