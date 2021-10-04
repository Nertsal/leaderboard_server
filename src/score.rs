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
    pub extra_info: Option<String>,
}

impl ScoreRecord {
    pub fn new(score: GameScore, extra_info: Option<String>) -> Self {
        Self { score, extra_info }
    }
}
