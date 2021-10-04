use rocket::serde::json::Json;
use rocket::*;
use sqlx::Row;

mod request_error;
pub mod requests;
mod score;

pub use request_error::*;
pub use score::{GameScore, ScoreRecord};

pub type DatabasePool = sqlx::any::AnyPool;
pub type GameId = i32;
