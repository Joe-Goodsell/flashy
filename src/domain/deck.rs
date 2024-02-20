use super::card::Card;

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        Deck { cards: Vec::new() }
    }

    pub fn new_from_db(name: &str) -> Self {
        // Load table from DB
        Deck { cards: todo!() }
    }

    pub fn load(name: &str) -> Result<Self, std::io::Error> {
        todo!()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter()
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }
}
