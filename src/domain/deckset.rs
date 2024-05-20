
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

        let decks: Vec<Deck> = raw.iter().map(Deck::from).collect();

        Ok(
            DeckSet {
                decks
            }
        )
    }

    pub fn get_deck_by_id(&self, id: Uuid) -> Option<Deck> {
        self.decks.iter().find(|d| d.id == id).map(|d| d.clone())
    }

    pub async fn reload(&mut self, db: &PgPool) -> Result<(), sqlx::Error> {
        let raw: Vec<RawDeck> = sqlx::query_as!(
            RawDeck,
            r#"
            SELECT * FROM decks
            "#,
        )
        .fetch_all(db)
        .await?;

        self.decks = raw.iter().map(Deck::from).collect();
        Ok(())
    }

    pub async fn delete_deck_with_cards(&mut self, db: &PgPool, deck_id: Uuid) -> Result<(), sqlx::Error> {
        // DELETE ALL CARDS IN DECK
        sqlx::query!(
            r#"
            DELETE FROM cards
            WHERE deck_id = ($1)
            "#,
            deck_id
        ).execute(db)
        .await?;

        // DELETE DECK
        sqlx::query!(
            r#"
            DELETE FROM decks
            WHERE id = ($1)
            "#,
            deck_id
        ).execute(db)
        .await?;

        self.reload(db).await?;

        Ok(())
    }
}