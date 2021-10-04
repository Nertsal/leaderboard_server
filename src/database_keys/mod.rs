use rocket::serde::{Deserialize, Serialize};

mod generate;
mod guard;

pub use guard::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GameKeys {
    pub read_key: StringKey,
    pub write_key: StringKey,
    pub admin_key: StringKey,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct StringKey {
    key: String,
}

impl StringKey {
    #![allow(dead_code)]
    pub fn inner(&self) -> &str {
        &self.key
    }
}

impl std::fmt::Display for StringKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}
