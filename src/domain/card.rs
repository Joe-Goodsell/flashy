use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;


#[derive(Debug)]
pub struct Card {
    pub id: Uuid,
    pub front_text: Option<String>,
    pub back_text: Option<String>,
    pub deck_id: Option<Uuid>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
}