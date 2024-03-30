use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;
use super::deckset::RawDeck;

use super::card::Card;

#[derive(Debug, Clone)]
pub struct Deck {
    pub id: Uuid,
    pub name: String,
    pub cards: Option<Vec<Card>>,
}

impl From<&RawDeck> for Deck {
    /// Creates a Deck from sql query values
    /// Deck is created *without* cards loaded from db!
    fn from(value: &RawDeck) -> Self {
        Deck {
            id: value.id,
            name: value.name.clone(),
            cards: None,
        }
    }
}

impl Default for Deck {
    fn default() -> Self {
        let id = Uuid::new_v4();
        tracing::info!("Creating default deck with id {}", id);
        Deck {
            id,
            name: "default".to_string(),
            cards: None,
        }
    }

}

impl Deck {
    pub fn new(name: &str) -> Self {
        let id = Uuid::new_v4();
        Deck {
            id,
            name: name.to_string(),
            cards: None
        }
    }

    pub async fn load_cards(&mut self, db: &PgPool) -> Result<(), sqlx::Error> {
        let cards: Vec<Card> = sqlx::query_as!(
            Card,
            r#"
            SELECT * FROM cards
            WHERE deck_id = $1
            "#,
            self.id,
        )
        .fetch_all(db)
        .await?;

        self.cards = Some(cards);
        Ok(())
    }



    pub async fn new_from_db(name: &str, db: &PgPool) -> Result<Self, sqlx::Error> {
        // Load table from DB
        let id: Uuid = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id FROM decks
            WHERE name = ($1)
            "#,
        )
        .bind(name)
        .fetch_one(db)
        .await?;

        let mut deck = Deck {
            id,
            name: name.to_string(),
            cards: None,
        };

        deck.load_cards(db);
        Ok(deck)
    }

    pub async fn save_to_db(&self, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
        // CREATE TABLE IF NOT EXISTS decks (
        //     id UUID NOT NULL,
        //     PRIMARY KEY(id),
        //     name TEXT UNIQUE NOT NULL,
        //     created TIMESTAMPTZ,
        //     modified TIMESTAMPTZ
        // );
        sqlx::query!(
            r#"
            INSERT INTO decks (id, name, created, modified)
            VALUES ($1, $2, $3, $4)
            "#,
            self.id,
            self.name,
            Utc::now(),
            Utc::now(),
        )
        .execute(pg_pool)
        .await?;
        Ok(())
    }

    pub fn load(name: &str) -> Result<Self, std::io::Error> {
        todo!()
    }

    // TODO: fix this
    // pub fn iter(&self) -> impl Iterator<Item = &RawCard> {
    //     self.cards.iter()
    // }

    pub fn add(&mut self, card: Card) -> Result<(), std::io::Error> {
        match &mut self.cards {
            Some(c) => {
                c.push(card);
                Ok(())
            },
            // TODO: fix error variant
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "No cards available")),
        }
    }
}
