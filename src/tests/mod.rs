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

/// Creates and deletes a game in the database
#[rocket::async_test]
async fn create_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = "test_game".to_owned();
    let id = create_game(&client, &game_name).await.unwrap();
    println!("New game's id is {}", id);

    // Check game
    let uri = format!("/games/{}", game_name);
    let response = client.get(&uri).dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(deserialize_response::<GameId>(response).await.unwrap(), id);

    // Delete a game
    let response = delete_game(&client, &game_name).await;
    assert_eq!(response, Some(0));
}

/// Deletes an unexisting game
#[rocket::async_test]
async fn delete_unexistent() {
    let client = spawn_client().await;

    let response = delete_game(&client, "no_such_game_exists").await;
    assert_eq!(response, None);
}
