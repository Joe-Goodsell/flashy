use serde::Deserialize;

#[derive(Deserialize, Debug, sqlx::FromRow)]
pub struct Card {
    front_text: String,
    back_test: String,
    id: u64,
}
