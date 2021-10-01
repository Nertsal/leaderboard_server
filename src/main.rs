use rocket::serde::json::Json;
use rocket::*;
use score::GameScore;

mod score;
#[cfg(test)]
mod tests;

type SqlResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

type DatabasePool = sqlx::any::AnyPool;

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
        .mount(
            "/",
            routes![
                index,
                new_game,
                delete_game,
                get_game_leaderboard,
                add_game_score
            ],
        )
        .manage::<DatabasePool>(database_pool)
}

#[get("/")]
fn index() -> &'static str {
    "This is an online leaderboard server!"
}

#[post("/games", data = "<game_name>")]
pub async fn new_game(game_name: String, database: &State<DatabasePool>) -> SqlResult<()> {
    // TODO: ensure the action is legalised (game_developer key)

    // Create a new game table
    sqlx::query(&format!(
        "CREATE TABLE {}_leaderboard (score int)",
        game_name
    ))
    .execute(database.inner())
    .await?;

    // TODO: generate access keys (write and admin)

    Ok(())
}

#[delete("/games/<game_name>")]
pub async fn delete_game(game_name: String, database: &State<DatabasePool>) -> SqlResult<()> {
    // TODO: ensure the action is legalised (admin key)

    // Delete game table
    sqlx::query(&format!("DROP TABLE {}_leaderboard", game_name))
        .execute(database.inner())
        .await?;

    // TODO: remove access keys

    Ok(())
}

#[get("/games/<game_name>/leaderboard", format = "json")]
pub async fn get_game_leaderboard(
    game_name: String,
    database: &State<DatabasePool>,
) -> SqlResult<Json<Vec<i32>>> {
    // Get info from the database
    let result = sqlx::query(&format!("SELECT score FROM {}_leaderboard", game_name))
        .fetch_all(database.inner())
        .await?;

    // Translate info
    use sqlx::Row;
    let result = result
        .into_iter()
        .map(|row| {
            row.try_get_unchecked::<i32, usize>(0)
                .expect("failed to decode value")
        })
        .collect();
    Ok(Json(result))
}

#[post("/games/<game_name>/leaderboard", format = "json", data = "<score>")]
pub async fn add_game_score(
    game_name: String,
    score: Json<GameScore>,
    database: &State<DatabasePool>,
) -> SqlResult<()> {
    // TODO: ensure the action is legalised (write key)

    // Insert score
    let score = score.0;
    sqlx::query(&format!(
        "INSERT INTO {}_leaderboard (score) VALUES ({})",
        game_name, score.score
    ))
    .execute(database.inner())
    .await?;

    Ok(())
}
