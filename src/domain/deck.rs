use sqlx::{postgres::PgRow, FromRow, PgPool};
use uuid::Uuid;

use super::card::Card;

#[derive(Debug)]
pub struct Deck {
    id: Uuid,
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        Deck {
            id: Uuid::new_v4(),
            cards: Vec::new(),
        }
    }

    pub async fn fetch_from_db(name: &str, db: &PgPool) -> Result<Self, sqlx::Error> {
        // Load table from DB
        let id = sqlx::query!(
            r#"
            SELECT id FROM decks
            WHERE name = ($1)
            "#,
            name,
        )
        .execute(db)
        .await?;

        let rows: Vec<PgRow> = sqlx::query!(
            r#"
            SELECT * FROM cards
            WHERE deck_id = ($1)
            "#,
            id,
        )
        .fetch_all(db)
        .await?;

        let cards = Vec::<Card>::new();
        for row in &rows {
            if let Ok(card) = Card::from_row(row) {
                cards.push(card);
            }
        }

        Ok(Deck { id, cards })
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
