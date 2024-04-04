use std::io::{stdout, Stdout};
use crossterm::terminal::{
    disable_raw_mode, 
    enable_raw_mode, 
    EnterAlternateScreen, 
    LeaveAlternateScreen
};
use ratatui::{
    backend::CrosstermBackend, 
    layout::{
        Constraint, 
        Direction, 
        Layout, 
        Rect}, 
    Terminal
};

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
    // BUG: this doesn't work properly
    let centre_rect = Layout::default().direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(size_y),
            Constraint::Percentage(100),
        ])
        .split(area);

    Layout::default().direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(100),
                Constraint::Min(size_x),
                Constraint::Percentage(100),
            ])
            .split(centre_rect[1])[1]
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


/// Implementation of KMP string search algorithm
#[derive(Debug, Clone)]
pub struct Searcher {
    search_string: String,
    text: Vec<String>,
    table: Vec<usize>,
    valid_strings: Vec<bool>,
}

impl Searcher {
    pub fn new(text: Vec<&str>) -> Self {
        // TODO: rewrite
        let mut searcher = Self {
            search_string: String::new(),
            //TODO: need to clarify &str or String
            text: text.iter().map(|s| s.to_string()).collect(),
            table: Vec::new(),
            valid_strings: Vec::new(),
        };
        searcher.build();
        searcher
    }

    pub fn get_search_string(&self) -> String {
        self.search_string.clone()
    }


    /// Set search string to value directly
    pub fn set_search_string(&mut self, search_string: &str) {
        self.search_string = search_string.to_string();
        self.build_index();
    }

    /// Push character to search string (and rebuilds index table)
    pub fn push_and_search(&mut self, character: char) {
        self.search_string.push(character);
        self.build_index();

    }

    /// Pop character from search string (and rebuilds index table)
    /// TODO: cache previous results for speed
    pub fn pop_and_search(&mut self) {
        self.search_string.pop();
        self.build_index();
        // Consider cacheing here for speed when deleting chars
        for b in self.valid_strings.iter_mut() {
            *b = true;
        }
        self.search();
    }

    pub fn build_index(&mut self) {
        tracing::info!("Building index table on {}", self.search_string);

        self.table = vec![0usize; self.search_string.len() + 1];
        if self.search_string.len() <= 1 { return; }

        let (mut pref_ind, mut suff_ind) = (0usize, 1usize);
        let ss_bytes: Vec<u8> = self.search_string.bytes().collect();
        while suff_ind < ss_bytes.len() {
            tracing::info!("suff_ind is {}", suff_ind);
            if ss_bytes[pref_ind] == ss_bytes[suff_ind] {
                tracing::info!("match on {} and {}", ss_bytes[pref_ind], ss_bytes[suff_ind]);
                self.table[suff_ind+1] = self.table[suff_ind] + 1;
                pref_ind += 1; 
                suff_ind += 1;
            } else {
                tracing::info!("no match");
                while suff_ind < ss_bytes.len() && ss_bytes[pref_ind] != ss_bytes[suff_ind] {
                    suff_ind += 1;
                    tracing::info!("suf_ind incr to {}", suff_ind);
                }
            }
        }
        tracing::info!("Index table built: {:?}", self.table);
    }


    /// Build the index table etc.
    pub fn build(&mut self) {
        self.build_index();
        self.valid_strings = vec![true; self.text.len()];
        assert!(self.valid_strings.len() == self.text.len());
    }

    pub fn get_text(&self) -> Vec<String> {
        self.text
            .iter()
            .zip(self.valid_strings.iter())
            .filter(|(_,b)| **b)
            .map(|(t,_)| t.clone())
            .collect()
    }

    pub fn get_index_table(&self) -> Vec<usize> {
        self.table.clone()
    }

    /// Search
    pub fn search(&mut self) {
        tracing::info!("SEARCHING");

        for (b, t) in self.valid_strings.iter_mut().zip(self.text.iter()) {
            if b == &mut true {
                tracing::info!("doing KMP for ss {} in {}", &self.search_string, &t);
                *b = Self::kmp(&self.search_string, t, &self.table);
            }
        }

        let _ = self.valid_strings
            .iter_mut()
            .zip(self.text.iter())
            .map(|(is_in_play, text)| {
                match is_in_play {
                    true => {
                        tracing::info!("doing KMP for ss {} in {}", &self.search_string, &text);
                        *is_in_play = Self::kmp(&self.search_string, text, &self.table);
                    },
                    false => {},
                }
            });
    }

    fn kmp(search_string: &str, text: &str, index_table: &[usize]) -> bool {
        tracing::info!("performing kmp search, searching for `{}` in `{}`...", search_string, text);
        let (mut ss_ptr, mut txt_ptr) = (0usize, 0usize);
        let search_bytes = search_string.as_bytes();
        let text_bytes = text.as_bytes();

        if search_bytes.is_empty() {
            return true;
        }

        while txt_ptr < text_bytes.len() && ss_ptr < search_bytes.len() {
            if search_bytes[ss_ptr] == text_bytes[txt_ptr] {
                ss_ptr += 1; txt_ptr += 1;
                if ss_ptr >= search_bytes.len() { 
                    tracing::info!("found string {} in {}", search_string, text);
                    return true; 
                }
            } else if ss_ptr == 0 {
                txt_ptr += 1;
            } else {
                ss_ptr = index_table[ss_ptr];
            }
        }
        tracing::info!("result: not found");
        false
    }
}

// TODO: testing for search algo
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_table_correct() {
        let mut searcher = Searcher {
            search_string: "abcdab".to_string(),
            text: Vec::new(),
            table: Vec::new(),
            valid_strings: Vec::new(),
        };
        searcher.build();
        assert_eq!(vec![0, 0, 0, 0, 0, 1, 2], searcher.get_index_table());
    }
}
