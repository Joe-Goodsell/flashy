use uuid::Uuid;

#[derive(serde::Deserialize, Debug, sqlx::FromRow)]
pub struct Card {
    front_text: String,
    back_test: String,
    id: Uuid,
}
