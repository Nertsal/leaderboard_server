use rocket::*;
use rocket::{
    serde::{
        json::{json, Json, Value},
        Deserialize, Serialize,
    },
    tokio::sync::Mutex,
};

mod leaderboard;
#[cfg(test)]
mod tests;

type Leaderboard<T> = Mutex<leaderboard::Leaderboard<T>>;
type Leaderboards<'r> = &'r State<Leaderboard<u32>>;

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![index, get_leaderboard, add_score])
        .manage(Leaderboard::new(leaderboard::Leaderboard::<u32>::new(
            vec![],
        )))
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/leaderboard", format = "json")]
pub async fn get_leaderboard(leaderboards: Leaderboards<'_>) -> Json<Value> {
    let leaderboards = leaderboards.lock().await;
    Json(leaderboards.to_json())
}

#[post("/leaderboard", format = "json", data = "<score>")]
pub async fn add_score(score: Json<u32>, leaderboards: Leaderboards<'_>) -> Json<Value> {
    let score = score.0;
    let mut leaderboards = leaderboards.lock().await;
    leaderboards.add(score);

    Json(json!({
        "status": "ok"
    }))
}
