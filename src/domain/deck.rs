use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;

use super::card::RawCard;

#[derive(Debug)]
pub struct Deck {
    pub id: Uuid,
    pub name: String,
    pub cards: Vec<RawCard>,
}

impl Default for Deck {
    fn default() -> Self {
        let id = Uuid::new_v4();
        tracing::info!("Creating default deck with id {}", id);
        Deck {
            id,
            name: "default".to_string(),
            cards: Vec::new(),
        }
    }
}

impl Deck {
    pub async fn load_from_db(name: &str, db: &PgPool) -> Result<Self, sqlx::Error> {
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

        let cards: Vec<RawCard> = sqlx::query_as!(
            RawCard,
            r#"
            SELECT * FROM cards
            WHERE deck_id = $1
            "#,
            id,
        )
        .fetch_all(db)
        .await?;

        Ok(Deck {
            id,
            cards,
            name: name.to_string(),
        })
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

    pub fn iter(&self) -> impl Iterator<Item = &RawCard> {
        self.cards.iter()
    }

    pub fn add(&mut self, card: RawCard) {
        self.cards.push(card);
    }
}
