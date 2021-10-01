use rocket::{http::Status, local::asynchronous::Client};

use crate::score::GameScore;

macro_rules! into_json {
    ( $expr: expr ) => {
        rocket::serde::json::from_str(&$expr.into_string().await.unwrap()).unwrap()
    };
}

async fn spawn_client() -> Client {
    Client::tracked(super::rocket().await)
        .await
        .expect("valid rocket instance")
}

/// Creates a table and returns a uri to its leaderboard
async fn create_table(client: &Client, game_name: &str) -> String {
    let response = client.post("/games").body(game_name).dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    format!("/games/{}/leaderboard", game_name)
}

async fn insert_and_check(client: &Client, uri: &str, score: GameScore) {
    // Insert scores
    let response = client.post(uri).json(&score).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    // Check scores
    let response = client.get(uri).dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let result: Vec<i32> = into_json!(response);
    assert_eq!(result, vec![score.score]);
}

async fn delete_game(client: &Client, game_name: &str) {
    let response = client
        .delete(format!("/games/{}", game_name))
        .dispatch()
        .await;
    assert_eq!(response.status(), Status::Ok);
}

#[rocket::async_test]
async fn get_empty() {
    let client = spawn_client().await;

    // Create a table
    let game_name = "test_game";
    let uri = create_table(&client, game_name).await;

    // Get empty table
    let response = client.get(&uri).dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let result: Vec<i32> = into_json!(response);
    assert_eq!(result, Vec::<i32>::new());

    // Delete the table
    delete_game(&client, game_name).await;
}

#[rocket::async_test]
async fn add_score() {
    let client = spawn_client().await;

    // Create a table
    let game_name = "test_game";
    let uri = create_table(&client, game_name).await;

    // Insert and check values
    insert_and_check(&client, &uri, GameScore { score: 10 }).await;

    // Delete the table
    delete_game(&client, game_name).await;
}
