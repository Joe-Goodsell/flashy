
use sqlx::{types::chrono::{DateTime, Utc}, PgPool};
use uuid::Uuid;

use super::deck::Deck;



// CREATE TABLE IF NOT EXISTS decks (
//     id UUID NOT NULL,
//     PRIMARY KEY(id),
//     name TEXT UNIQUE NOT NULL,
//     created TIMESTAMPTZ,
//     modified TIMESTAMPTZ 
// );
pub struct RawDeck {
    pub id: Uuid,
    pub name: String,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
}


#[derive(Debug)]
pub struct DeckSet {
    pub decks: Vec<Deck>,
}

impl DeckSet {
    pub async fn load_from_db(db: &PgPool) -> Result<Self, sqlx::Error> {
        let raw: Vec<RawDeck> = sqlx::query_as!(
            RawDeck,
            r#"
            SELECT * FROM decks
            "#,
        )
        .fetch_all(db)
        .await?;

        let decks: Vec<Deck> = raw.iter().map(|d| Deck::from(d)).collect();

        Ok(
            DeckSet {
                decks
            }
        )
    }
}