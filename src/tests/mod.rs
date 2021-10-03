use rocket::{
    http::Status,
    local::asynchronous::{Client, LocalResponse},
};

use crate::{score::GameScore, GameId};

async fn spawn_client() -> Client {
    Client::tracked(super::rocket().await)
        .await
        .expect("valid rocket instance")
}

async fn deserialize_response<'a, T: rocket::serde::DeserializeOwned>(
    response: LocalResponse<'a>,
) -> rocket::serde::json::serde_json::Result<T> {
    let string = response.into_string().await.unwrap();
    rocket::serde::json::serde_json::from_str(&string)
}

/// Checks whether a game called `game_name` exists in the database
/// and return Some(game_id) if it does.
async fn check_game(client: &Client, game_name: &str) -> Option<GameId> {
    let uri = format!("/games/{}", game_name);
    let response = client.get(&uri).dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    deserialize_response::<GameId>(response)
        .await
        .map(|id| Some(id))
        .unwrap_or_default()
}

/// Creates a new game called `game_name` in the database
/// and returns its generated id
async fn create_game(client: &Client, game_name: &String) -> Option<GameId> {
    let response = client.post("/games").json(game_name).dispatch().await;
    if response.status() != Status::Ok {
        return None;
    }

    deserialize_response::<GameId>(response)
        .await
        .map(|id| Some(id))
        .unwrap_or_default()
}

/// Deletes a game called `game_name` from the database
/// and returns whether it was in the database
async fn delete_game(client: &Client, game_name: &str) -> Option<u64> {
    let uri = format!("/games/{}", game_name);
    let response = client.delete(&uri).dispatch().await;
    if response.status() != Status::Ok {
        return None;
    }

    deserialize_response::<u64>(response)
        .await
        .map(|rows| Some(rows))
        .unwrap_or_default()
}

/// Add score to the database under game called `game_name`.
/// Returns false if such game does not exist.
async fn add_score(client: &Client, game_name: &str, score: &GameScore) -> bool {
    let uri = format!("/games/{}", game_name);
    let response = client.post(&uri).json(score).dispatch().await;
    if response.status() != Status::Ok {
        return false;
    }

    true
}

const TEST_GAME_NAME: &'static str = "test_game";

/// Checks that a game called `TEST_GAME_NAME` does not exist.
/// It it does exist, then all other tests must fail,
/// as this name is used for testing.
#[rocket::async_test]
async fn prepare_test() {
    let client = spawn_client().await;

    let response = check_game(&client, TEST_GAME_NAME).await;
    assert_eq!(
        response, None,
        "Please delete a game called \'{}\' from the database for testing purposes",
        TEST_GAME_NAME
    );
}

/// Deletes an unexisting game
#[rocket::async_test]
async fn delete_unexistent() {
    let client = spawn_client().await;

    let response = delete_game(&client, TEST_GAME_NAME).await;
    assert_eq!(response, None);
}

/// Creates and deletes a game in the database
#[rocket::async_test]
async fn create_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let game_id = create_game(&client, &game_name).await.unwrap();

    // Check game
    assert_eq!(check_game(&client, &game_name).await, Some(game_id));

    // Delete the game
    let response = delete_game(&client, &game_name).await;
    assert_eq!(response, Some(0));
}

/// Creates and deletes a game in the database
#[rocket::async_test]
async fn create_add_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let game_id = create_game(&client, &game_name).await.unwrap();

    // Check game
    assert_eq!(check_game(&client, &game_name).await, Some(game_id));

    // Add score
    let score = GameScore { score: 10 };
    let response = add_score(&client, &game_name, &score).await;
    assert!(response);

    // Delete the game
    let response = delete_game(&client, &game_name).await;
    assert_eq!(response, Some(1));
}
