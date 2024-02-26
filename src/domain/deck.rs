use sqlx::{postgres::PgRow, types::chrono::{DateTime, Local, NaiveDateTime, Utc}, FromRow, PgPool};
use uuid::Uuid;

use super::card::Card;


pub struct CardRow {
    pub id: Uuid,
    pub front_text: Option<String>,
    pub back_text: Option<String>,
    pub deck_id: Option<Uuid>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug)]
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
        let id: Uuid = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id FROM decks
            WHERE name = ($1)
            "#
        ).bind(name)
        .fetch_one(db)
        .await?;

        let rows: Vec<CardRow> = sqlx::query_as!(
            CardRow,
            r#"
            SELECT * FROM cards
            WHERE deck_id = $1
            "#,
            id,
        )
        .fetch_all(db)
        .await?;

        // let cards = Vec::<Card>::new();
        // for row in &rows {
        //     if let Ok(card) = Card::from_row(row) {
        //         cards.push(card);
        //     }
        // }
        Ok(Deck { id, cards: Vec::<Card>::new()})
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
