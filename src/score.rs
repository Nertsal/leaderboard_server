use rocket::serde::{Deserialize, Serialize};

// Types allowed in the database:
// i32
// i64
// f32
// f64
// bool
// &'r str
// String

pub type GameScore = i32;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ScoreRecord {
    pub score: GameScore,
}
