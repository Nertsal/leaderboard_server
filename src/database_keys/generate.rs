use super::*;

use rand::{distributions::Alphanumeric, thread_rng, Rng};

impl StringKey {
    pub fn generate(length: usize) -> Self {
        let rng = thread_rng();

        let key: String = rng
            .sample_iter(Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();

        Self { key }
    }
}

impl GameKeys {
    pub fn generate() -> Self {
        Self {
            read_key: StringKey::generate(10),
            write_key: StringKey::generate(10),
            admin_key: StringKey::generate(20),
        }
    }
}
