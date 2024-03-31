use std::io::{stdout, Stdout};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, Terminal};


pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialise the terminal
pub fn init() -> std::io::Result<Tui> {
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}


/// Restore the terminal to its original state
pub fn restore() -> std::io::Result<()> {
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn create_centred_rect_by_size(size_x: u16, size_y: u16, area: Rect) -> Rect {
    let centre_rect = Layout::default().direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(size_y),
            Constraint::Percentage(100),
        ])
        .split(area)[1];

    Layout::default().direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(100),
                Constraint::Min(size_x),
                Constraint::Percentage(100),
            ])
            .split(centre_rect)[1]
}

pub fn create_centred_rect_by_percent(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    // First split vertical (i.e. splits stack on top of each other)
    // Popup will fill `percent_y` proportion of screen
    let centre_rect = Layout::default().direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage((100-percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100-percent_y) / 2), 
        ])
        .split(area);

    Layout::default().direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage((100-percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100-percent_x) / 2),
            ])
            .split(centre_rect[1])[1] // Only take middle rectangles
}