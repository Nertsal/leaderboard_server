use super::*;

pub struct Leaderboard<T: LeaderboardItem> {
    collection: Vec<T>,
}

impl<T: LeaderboardItem> Leaderboard<T> {
    pub fn new(collection: Vec<T>) -> Self {
        Self { collection }
    }

    pub fn add(&mut self, item: T) {
        self.collection.push(item);
    }

    pub fn to_json(&self) -> Value {
        serde::json::serde_json::to_value(self).expect("Failed to convert leaderboard to json")
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.collection.iter()
    }
}

impl<T: LeaderboardItem> Serialize for Leaderboard<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.collection.serialize(serializer)
    }
}

impl<'de, T: LeaderboardItem> Deserialize<'de> for Leaderboard<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(Vec::deserialize(deserializer)?))
    }
}

pub trait LeaderboardItem: Ord + Serialize {}

impl<T: Ord + Serialize> LeaderboardItem for T {}
