#[derive(Debug)]
pub enum RequestError {
    Unathorized,
    InvalidGameName { game_name: String },
    GameAlreadyExists { game_name: String },
    NoSuchGame { game_name: String },
}

impl std::error::Error for RequestError {}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unathorized => write!(f, "unathorized request"),
            Self::InvalidGameName { game_name } => write!(f, "invalid game name: {}", game_name),
            Self::GameAlreadyExists { game_name } => write!(
                f,
                "a game called {} already exists in the database",
                game_name
            ),
            RequestError::NoSuchGame { game_name } => write!(
                f,
                "no game with the name {} exists in the database",
                game_name
            ),
        }
    }
}

pub type RequestResult<T, E = rocket::response::Debug<RequestError>> = std::result::Result<T, E>;
