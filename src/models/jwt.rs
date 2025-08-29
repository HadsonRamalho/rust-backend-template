use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub id: Uuid,
    pub public_id: i32,
    pub user_type: String,
    pub email: String,
    pub exp: usize,
}
