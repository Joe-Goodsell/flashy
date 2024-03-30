use sqlx::{types::chrono::{DateTime, Utc}, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Card {
    pub id: Uuid,
    pub front_text: Option<String>,
    pub back_text: Option<String>,
    pub deck_id: Option<Uuid>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
}

impl Default for Card {
    fn default() -> Self {
        Self { 
            id: Uuid::new_v4(), 
            front_text: None, 
            back_text: None, 
            deck_id: None, 
            created: None, 
            modified: None
        } 
    }
}

impl Card {
    pub fn new() -> Self {
        Card {
            id: Uuid::new_v4(),
            front_text: None,
            back_text: None,
            deck_id: None,
            created: None,
            modified: None,
        }
    }

    pub fn set_front_text(&mut self, text: String) {
        self.front_text = Some(text);
    }

    pub fn set_back_text(&mut self, text: String) {
        self.back_text = Some(text);
    }

    pub async fn save(&self, connection_pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO cards (id, front_text, back_text, deck_id, created, modified)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET front_text = EXCLUDED.front_text,
            back_text = EXCLUDED.back_text,
            modified = EXCLUDED.modified
            "#,
            self.id,
            self.front_text,
            self.back_text,
            self.deck_id,
            Utc::now(),
            Utc::now(), // `modified` will be overwritten where card exists in db
        )
        .execute(connection_pool)
        .await?;
        Ok(())
    }
}