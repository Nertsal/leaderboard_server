use rocket::serde::json::Json;
use rocket::*;
use sqlx::Row;

mod score;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum RequestError {
    GameExists { game_name: String },
}

impl std::error::Error for RequestError {}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GameExists { game_name } => write!(
                f,
                "a game called {} already exists in the database",
                game_name
            ),
        }
    }
}

type RequestResult<T, E = rocket::response::Debug<RequestError>> = std::result::Result<T, E>;

type DatabasePool = sqlx::any::AnyPool;

type GameId = i32;

#[launch]
async fn rocket() -> _ {
    // Connect to a database
    dotenv::dotenv().ok();
    let database_url =
        dotenv::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");

    let database_pool = DatabasePool::connect(&database_url)
        .await
        .expect("failed to connect to a database");

    // Build the rocket
    rocket::build()
        .mount("/", routes![index, check_game, add_game, delete_game])
        .manage::<DatabasePool>(database_pool)
}

#[get("/")]
fn index() -> &'static str {
    "This is an online leaderboard server!"
}

/// Checks whether a game called `game_name` is registered in the database.
/// Returns Some(game_id) if it does, else None.
#[get("/games/<game_name>", format = "json")]
pub async fn check_game(
    game_name: &str,
    database: &State<DatabasePool>,
) -> RequestResult<Json<Option<GameId>>> {
    // Check that a game with such name does not exist
    let response = sqlx::query(&format!(
        "SELECT game_id FROM games WHERE game_name = \'{}\'",
        game_name,
    ))
    .fetch_optional(database.inner())
    .await
    .unwrap();

    Ok(Json(
        response.map(|row| row.get_unchecked::<GameId, usize>(0)),
    ))
}

/// Attempts to add a new game called `game_name` into the database.
/// Returns an error if such game is already in the database,
/// otherwise returns the new game's id.
#[post("/games", format = "json", data = "<game_name>")]
pub async fn add_game(
    game_name: Json<String>,
    database: &State<DatabasePool>,
) -> RequestResult<Json<GameId>> {
    let game_name = game_name.0;

    // Check that a game with such name does not exist
    let check = check_game(&game_name, database).await.unwrap();
    if check.is_some() {
        return Err(RequestError::GameExists { game_name }.into());
    }

    // Insert
    let response = sqlx::query(&format!(
        "INSERT INTO games (game_name) VALUES (\'{}\') RETURNING game_id",
        game_name
    ))
    .fetch_one(database.inner())
    .await
    .unwrap();

    let id = response.get_unchecked::<GameId, usize>(0);

    Ok(Json(id))
}

/// Attempts to delete a game called `game_name` from the `games` table
/// and all scores for that game from the `scores` table if such game is registered.
/// If such game was present in the database, returns the number of scores removed,
/// otherwise returns None.
#[delete("/games/<game_name>")]
pub async fn delete_game(
    game_name: &str,
    database: &State<DatabasePool>,
) -> RequestResult<Json<Option<u64>>> {
    // Delete game from the the `games` table
    let response = sqlx::query(&format!(
        "DELETE FROM games WHERE game_name = \'{}\' RETURNING game_id",
        game_name
    ))
    .fetch_optional(database.inner())
    .await
    .unwrap();

    // Check that such game existed
    let game_id = match response {
        Some(response) => response.get_unchecked::<GameId, usize>(0),
        None => {
            return Ok(Json(None));
        }
    };

    // Delete scores
    let response = sqlx::query(&format!("DELETE FROM scores WHERE game_id = {}", game_id))
        .execute(database.inner())
        .await
        .unwrap();

    let rows_affected = response.rows_affected();
    Ok(Json(Some(rows_affected)))
}
