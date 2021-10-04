use std::io::Cursor;

use rocket::{http::Status, response::Responder, Request, Response};

pub type RequestResult<T, E = RequestError> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum RequestError {
    Unathorized,
    Forbidden,
    InvalidGameName { game_name: String },
    GameAlreadyExists { game_name: String },
    NoSuchGame { game_name: String },
}

impl RequestError {
    fn status(&self) -> Status {
        match self {
            RequestError::Unathorized => Status::Unauthorized,
            RequestError::Forbidden => Status::Forbidden,
            RequestError::InvalidGameName { .. } => Status::BadRequest,
            RequestError::GameAlreadyExists { .. } => Status::Conflict,
            RequestError::NoSuchGame { .. } => Status::NotFound,
        }
    }
}

impl std::error::Error for RequestError {}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unathorized => write!(f, "unathorized request"),
            Self::Forbidden => write!(f, "unathorized request, not enough rights"),
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

impl<'r> Responder<'r, 'static> for RequestError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let response_string = format!("{}", self);
        Ok(Response::build()
            .sized_body(response_string.len(), Cursor::new(response_string))
            .status(self.status())
            .finalize())
    }
}
