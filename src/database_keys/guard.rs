use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};

pub struct ApiKey<'r>(pub &'r str);

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
}

impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyError::Missing => write!(f, "the key is missing"),
            ApiKeyError::Invalid => write!(f, "the key is invalid"),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_valid(_key: &str) -> bool {
            true
        }

        match request.headers().get_one("api-key") {
            None => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}
