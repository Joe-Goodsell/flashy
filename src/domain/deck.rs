use sqlx::PgPool;
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

    pub async fn load_from_db(name: &str, db: &PgPool) -> Result<Self, sqlx::Error> {
        // Load table from DB
        let id: Uuid = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id FROM decks
            WHERE name = ($1)
            "#
        ).bind(name)
        .fetch_one(db)
        .await?;

        let cards: Vec<Card> = sqlx::query_as!(
            Card,
            r#"
            SELECT * FROM cards
            WHERE deck_id = $1
            "#,
            id,
        )
        .fetch_all(db)
        .await?;

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
