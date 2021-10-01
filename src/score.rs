use rocket::serde::{Deserialize, Serialize};

// Types allowed in the database:
// i32
// i64
// f32
// f64
// bool
// &'r str
// String

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GameScore {
    pub score: i32,
}
