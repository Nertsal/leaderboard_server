use crate::database_keys::{self, ApiKey, GameKeys};

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum AuthorityLevel {
    Unathorizied,
    Read,
    Write,
    Admin,
}

/// Checks whether a game called `game_name` is registered in the database.
/// Returns game id and authority level associated with the key if the game does exist.
pub async fn check_game(
    game_name: &str,
    api_key: Option<ApiKey<'_>>,
    database: &State<DatabasePool>,
) -> RequestResult<(GameId, AuthorityLevel)> {
    // Check that a game with such name does not exist
    let response = sqlx::query(&format!(
        "SELECT game_id, read_key, write_key, admin_key FROM games WHERE game_name = \'{}\'",
        game_name,
    ))
    .fetch_optional(database.inner())
    .await
    .unwrap();

    match response {
        Some(response) => {
            let game_id = response.get_unchecked::<GameId, usize>(0);
            let read_key = response.get_unchecked::<String, usize>(1);
            let write_key = response.get_unchecked::<String, usize>(2);
            let admin_key = response.get_unchecked::<String, usize>(3);

            let authority_level = match api_key {
                None => AuthorityLevel::Unathorizied,
                Some(key) => {
                    if key.0 == &read_key {
                        AuthorityLevel::Read
                    } else if key.0 == &write_key {
                        AuthorityLevel::Write
                    } else if key.0 == &admin_key {
                        AuthorityLevel::Admin
                    } else {
                        AuthorityLevel::Unathorizied
                    }
                }
            };

            Ok((game_id, authority_level))
        }
        None => Err(RequestError::NoSuchGame {
            game_name: game_name.to_owned(),
        }
        .into()),
    }
}

/// Attempts to add a new game called `game_name` into the database.
/// Returns an error if such game is already in the database,
/// otherwise returns the new game's id.
#[post("/games", data = "<game_name>", format = "json")]
pub async fn create_game(
    game_name: Json<String>,
    database: &State<DatabasePool>,
) -> RequestResult<Json<(GameId, GameKeys)>> {
    let game_name = game_name.0;

    if game_name.is_empty() {
        return Err(RequestError::InvalidGameName {
            game_name: game_name.to_owned(),
        }
        .into());
    }

    // Check that a game with such name does not exist
    let check = check_game(&game_name, None, database).await;
    if check.is_ok() {
        return Err(RequestError::GameAlreadyExists {
            game_name: game_name.to_owned(),
        }
        .into());
    }

    // Generate keys
    let game_keys = database_keys::GameKeys::generate();

    // Insert
    let response = sqlx::query(&format!(
        "INSERT INTO games (game_name, read_key, write_key, admin_key) VALUES (\'{}\', \'{}\', \'{}\', \'{}\') RETURNING game_id",
        game_name, game_keys.read_key, game_keys.write_key, game_keys.admin_key,
    ))
    .fetch_one(database.inner())
    .await
    .unwrap();

    let id = response.get_unchecked::<GameId, usize>(0);
    Ok(Json((id, game_keys)))
}

/// Attempts to delete a game called `game_name` from the `games` table
/// and all scores for that game from the `scores` table if such game is registered.
/// If such game was present in the database, returns the number of scores removed,
/// otherwise returns an error.
#[delete("/games/<game_name>")]
pub async fn delete_game(
    game_name: &str,
    api_key: ApiKey<'_>,
    database: &State<DatabasePool>,
) -> RequestResult<Json<u64>> {
    // Check that the api_key is admin level
    let (game_id, authority_level) = check_game(game_name, Some(api_key), database).await?;
    if authority_level < AuthorityLevel::Admin {
        return Err(RequestError::Unathorized.into());
    }

    // Delete game from the the `games` table
    sqlx::query(&format!("DELETE FROM games WHERE game_id = {}", game_id))
        .execute(database.inner())
        .await
        .unwrap();

    // Delete scores
    let response = sqlx::query(&format!("DELETE FROM scores WHERE game_id = {}", game_id))
        .execute(database.inner())
        .await
        .unwrap();

    let rows_affected = response.rows_affected();
    Ok(Json(rows_affected))
}

/// Adds score to the database under game called `game_name`ю
/// Returns an error if a game with such name already existed.
#[post("/games/<game_name>/scores", format = "json", data = "<score_record>")]
pub async fn add_score(
    game_name: &str,
    score_record: Json<ScoreRecord>,
    api_key: ApiKey<'_>,
    database: &State<DatabasePool>,
) -> RequestResult<()> {
    let score_record = score_record.0;

    // Check game
    let (game_id, authority_level) = check_game(game_name, Some(api_key), database).await?;

    // Check authority
    if authority_level < AuthorityLevel::Write {
        return Err(RequestError::Unathorized.into());
    }

    // Insert score
    let extra_info = match score_record.extra_info {
        Some(extra_info) => format!("\'{}\'", extra_info),
        None => format!("null"),
    };
    sqlx::query(&format!(
        "INSERT INTO scores (game_id, score, extra_info) VALUES ({}, {}, {})",
        game_id, score_record.score, extra_info
    ))
    .execute(database.inner())
    .await
    .unwrap();

    Ok(())
}

/// Fetches scores for a specific game from the `scores` database.
/// Returns an error if such game is not registered,
/// otherwise returns a vector of scores.
#[get("/games/<game_name>/scores", format = "json")]
pub async fn get_scores(
    game_name: &str,
    api_key: ApiKey<'_>,
    database: &State<DatabasePool>,
) -> RequestResult<Json<Vec<ScoreRecord>>> {
    // Check game
    let (game_id, authority_level) = check_game(game_name, Some(api_key), database).await?;

    // Check authority_level
    if authority_level < AuthorityLevel::Read {
        return Err(RequestError::Unathorized.into());
    }

    // Fetch scores
    let response = sqlx::query(&format!(
        "SELECT score, extra_info FROM scores WHERE game_id = {}",
        game_id
    ))
    .fetch_all(database.inner())
    .await
    .unwrap();

    let scores = response
        .into_iter()
        .map(|row| {
            let score = row.get_unchecked::<GameScore, usize>(0);
            let extra_info = row.get_unchecked::<Option<String>, usize>(1);
            ScoreRecord::new(score, extra_info)
        })
        .collect();

    Ok(Json(scores))
}
