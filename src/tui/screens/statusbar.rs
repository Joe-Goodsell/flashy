use ratatui::{
    layout::{Constraint, Direction, Layout}, 
    style::{Color, Style}, 
    text::Span, 
    widgets::{Block, Paragraph, Widget}
};

use crate::tui::app::Mode;



// TODO: Am I really instantatiating a whole new instance of a struct for *each frame*?
#[derive(Default, Debug)]
pub struct StatusBar {
    mode: Mode,
}

impl StatusBar {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode
        }
    }
}

impl Widget for StatusBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let layout = Layout::default().direction(Direction::Horizontal).constraints(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(area);

        let mode_disp  = format!("MODE: {}", self.mode);
        let color: Color = match self.mode {
            Mode::NORMAL => Color::Green,
            Mode::INSERT => Color::Red,
        };

        Paragraph::new(Span::styled(mode_disp, Style::new().fg(color)))
            .block(Block::default())
            .render(layout[0], buf);

        Paragraph::new(Span::styled("LINE 0X/XX | filename.rs  ", Style::new().bg(Color::LightBlue)))
            .block(Block::default())
            .render(layout[1], buf);
    }
}