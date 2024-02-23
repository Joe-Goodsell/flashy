use sqlx::{
    types::{chrono::Utc, Uuid},
    PgPool,
};

#[derive(Debug)]
pub struct NewCard {
    id: Uuid,
    deck_id: Uuid,
    front_text: String,
    back_text: String,
}

impl NewCard {
    pub fn new(deck_id: Uuid) -> Self {
        NewCard {
            id: Uuid::new_v4(),
            deck_id,
            front_text: String::new(),
            back_text: String::new(),
        }
    }

    pub fn set_front_text(&mut self, text: String) {
        self.front_text = text;
    }

    pub fn set_back_text(&mut self, text: String) {
        self.back_text = text;
    }

    // CREATE TABLE WHERE NOT EXISTS cards (
    //     id INTEGER PRIMARY KEY,
    //     front_text TEXT,
    //     back_text TEXT,
    //     deck_id INTEGER,
    //     FOREIGN KEY (deck_id) REFERENCES decks(id),
    //     created DATETIME,
    //     modified DATETIME,
    // )

    pub async fn save(&self, connection_pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO cards (id, front_text, back_text, deck_id, created, modified)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            self.id,
            self.front_text,
            self.back_text,
            self.deck_id,
            Utc::now(),
            Utc::now(),
        )
        .execute(connection_pool)
        .await?;
        Ok(())
    }
}
