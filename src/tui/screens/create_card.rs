use ratatui::{buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, text::Span, widgets::{Block, Borders, Paragraph, StatefulWidget, Widget}};

use crate::tui::utils::create_centred_rect;

#[derive(Debug, Default)]
pub struct CreateCard {
    front_field: String,
    back_field: String,
    pub state: CurrentlyEditing,
}

#[derive(Default, Debug)]
pub enum CurrentlyEditing {
    #[default]
    FrontText,
    BackText,
    Saving,
}

impl StatefulWidget for CreateCard {
    type State = CurrentlyEditing;
    /// Intended to render a popup window with fields `Front Text`, `Back Text`
    /// And a select menu for an existing Deck (TODO:)
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let popup_area: Rect = create_centred_rect(50u16, 50u16, area);

        let block = Block::default()
            .title(Span::styled("Add a card", Style::default().fg(Color::Yellow).bg(Color::Black)))
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));
        
        let body_text = Span::styled("TESTING ADD CARD", Style::default().bg(Color::Blue).fg(Color::Black).add_modifier(Modifier::ITALIC));

        Paragraph::new(body_text).block(block).render(popup_area, buf);
    }


    //TODO: implement saving to db
}