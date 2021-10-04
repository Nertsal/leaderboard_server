use rocket::*;

mod database;
mod database_keys;

#[cfg(test)]
mod tests;

#[launch]
async fn rocket() -> _ {
    use database::DatabasePool;

    // Connect to a database
    dotenv::dotenv().ok();
    let database_url =
        dotenv::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");

    let database_pool = DatabasePool::connect(&database_url)
        .await
        .expect("failed to connect to a database");

    // Build the rocket
    use database::requests;
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                requests::create_game,
                requests::delete_game,
                requests::add_score,
                requests::get_scores
            ],
        )
        .manage::<DatabasePool>(database_pool)
}

#[get("/")]
fn index() -> &'static str {
    "This is an online leaderboard server!"
}
