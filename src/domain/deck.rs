use sqlx::{postgres::PgRow, FromRow, PgPool};

use super::card::Card;

#[derive(serde::Deserialize, Debug)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        Deck { cards: Vec::new() }
    }

    pub async fn new_from_db(name: &str, db: &PgPool) -> Result<Self, sqlx::Error> {
        // Load table from DB
        let rows: Vec<PgRow> = sqlx::query(
            r#"
            SELECT * FROM cards
            "#,
        )
        .fetch_all(db)
        .await?;

        let cards = Vec::<Card>::new();
        for row in &rows {
            if let Ok(card) = Card::from_row(row) {
                cards.push(card);
            }
        }

        Ok(Deck { cards })
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
