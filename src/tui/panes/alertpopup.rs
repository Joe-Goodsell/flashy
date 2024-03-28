use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::Text, widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap}};
use std::time;

#[derive(Debug)]
pub enum AlertPriority {
    Green,
    Yellow,
    Red,
}


#[derive(Debug)]
pub struct AlertPopup<'a> {
    pub duration: time::Duration,
    pub start: time::Instant, 
    pub priority: AlertPriority,
    pub text: Text<'a>,
}

impl<'a> AlertPopup<'a> {
    pub fn new(duration: time::Duration, text: String, priority: AlertPriority) -> Self {
        let style: Style = match &priority {
            AlertPriority::Green => Style::default()
                .fg(Color::Green)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            AlertPriority::Yellow => Style::default()
                .fg(Color::Yellow)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            AlertPriority::Red => Style::default()
                .fg(Color::Red)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        };
        Self {
            duration,
            start: time::Instant::now(),
            text: Text::styled(text, style),
            priority,
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
            // TODO: add padding
            let mut top_right = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(100),
                    Constraint::Min(self.text.width() as u16 + 4),
                ])
                .split(area)[1];
            top_right.height = 4;
            Paragraph::new(self.text.clone())
                .style(self.text.style)
                .block(Block::default().borders(Borders::ALL))
                // .block(Block::default().borders(Borders::ALL).padding(Padding::uniform(2)))
                .wrap(Wrap { trim: true })
                .render(top_right, buf);
    }
} 