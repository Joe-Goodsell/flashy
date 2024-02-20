use sqlx::{
    types::{chrono::Utc, Uuid},
    PgPool,
};

#[derive(Debug)]
pub struct NewCard {
    id: Uuid,
    front_text: String,
    back_text: String,
}

impl NewCard {
    pub fn new() -> Self {
        NewCard {
            id: Uuid::new_v4(),
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

    pub async fn save(&self, connection_pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO cards (id, front_text, back_text, created_at, last_modified)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            self.id,
            self.front_text,
            self.back_text,
            Utc::now(),
            Utc::now(),
        )
        .execute(connection_pool)
        .await?;
        Ok(())
    }
}
