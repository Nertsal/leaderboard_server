use super::*;

/// Checks whether a game called `game_name` is registered in the database.
/// Returns Ok(game_id) if it does, otherwise returns an error.
#[get("/games/<game_name>", format = "json")]
pub async fn check_game(
    game_name: &str,
    database: &State<DatabasePool>,
) -> RequestResult<Json<GameId>> {
    // Check that a game with such name does not exist
    let response = sqlx::query(&format!(
        "SELECT game_id FROM games WHERE game_name = \'{}\'",
        game_name,
    ))
    .fetch_optional(database.inner())
    .await
    .unwrap();

    match response {
        Some(response) => {
            let game_id = response.get_unchecked::<GameId, usize>(0);
            Ok(Json(game_id))
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
#[post("/games", format = "json", data = "<game_name>")]
pub async fn add_game(
    game_name: Json<String>,
    database: &State<DatabasePool>,
) -> RequestResult<Json<GameId>> {
    let game_name = game_name.0;

    // Check that a game with such name does not exist
    let check = check_game(&game_name, database).await;
    if check.is_ok() {
        return Err(RequestError::GameAlreadyExists { game_name }.into());
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
/// otherwise returns an error.
#[delete("/games/<game_name>")]
pub async fn delete_game(
    game_name: &str,
    database: &State<DatabasePool>,
) -> RequestResult<Json<u64>> {
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
            return Err(RequestError::NoSuchGame {
                game_name: game_name.to_owned(),
            }
            .into());
        }
    };

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
    database: &State<DatabasePool>,
) -> RequestResult<()> {
    let score_record = score_record.0;

    // Get game's id
    let response = check_game(game_name, database).await?;
    let game_id = response.0;

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
    database: &State<DatabasePool>,
) -> RequestResult<Json<Vec<ScoreRecord>>> {
    // Check that the game exists
    let response = check_game(game_name, database).await?;
    let game_id = response.0;

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
