use rocket::{
    http::Status,
    local::asynchronous::{Client, LocalResponse},
};

use crate::{score::ScoreRecord, GameId};

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
async fn check_game(client: &Client, game_name: &str) -> Result<GameId, String> {
    let uri = format!("/games/{}", game_name);
    let response = client.get(&uri).dispatch().await;
    if response.status() != Status::Ok {
        return Err(response.into_string().await.unwrap());
    }

    let game_id = deserialize_response::<GameId>(response).await.unwrap();
    Ok(game_id)
}

/// Creates a new game called `game_name` in the database
/// and returns its generated id
async fn create_game(client: &Client, game_name: &String) -> Result<GameId, String> {
    let response = client.post("/games").json(game_name).dispatch().await;
    if response.status() != Status::Ok {
        return Err(response.into_string().await.unwrap());
    }

    let game_id = deserialize_response::<GameId>(response).await.unwrap();
    Ok(game_id)
}

/// Deletes a game called `game_name` from the database
/// and returns whether it was in the database
async fn delete_game(client: &Client, game_name: &str) -> Result<u64, String> {
    let uri = format!("/games/{}", game_name);
    let response = client.delete(&uri).dispatch().await;
    if response.status() != Status::Ok {
        return Err(response.into_string().await.unwrap());
    }

    let score_count = deserialize_response::<u64>(response).await.unwrap();
    Ok(score_count)
}

/// Add score to the database under game called `game_name`.
/// Returns false if such game does not exist.
async fn add_score(
    client: &Client,
    game_name: &str,
    score_record: &ScoreRecord,
) -> Result<(), String> {
    let uri = format!("/games/{}/scores", game_name);
    let response = client.post(&uri).json(score_record).dispatch().await;
    if response.status() != Status::Ok {
        return Err(response.into_string().await.unwrap());
    }

    Ok(())
}

async fn get_scores(client: &Client, game_name: &str) -> Result<Vec<ScoreRecord>, String> {
    let uri = format!("/games/{}/scores", game_name);
    let response = client.get(&uri).dispatch().await;
    if response.status() != Status::Ok {
        return Err(response.into_string().await.unwrap());
    }

    let scores = deserialize_response::<Vec<ScoreRecord>>(response)
        .await
        .unwrap();
    Ok(scores)
}

const TEST_GAME_NAME: &'static str = "test_game";

/// Checks that a game called `TEST_GAME_NAME` does not exist.
/// It it does exist, then all other tests must fail,
/// as this name is used for testing.
#[rocket::async_test]
async fn prepare_test() {
    let client = spawn_client().await;

    let response = check_game(&client, TEST_GAME_NAME).await;
    assert!(
        response.is_err(),
        "Please delete a game called \'{}\' from the database for testing purposes",
        TEST_GAME_NAME
    );
}

/// Deletes an unexisting game
#[rocket::async_test]
async fn delete_unexistent() {
    let client = spawn_client().await;

    let response = delete_game(&client, TEST_GAME_NAME).await;
    assert!(response.is_err());
}

/// Creates and deletes a game in the database
#[rocket::async_test]
async fn create_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let game_id = create_game(&client, &game_name).await.unwrap();

    // Check game
    let response = check_game(&client, &game_name).await;
    assert_eq!(response, Ok(game_id));

    // Delete the game
    let response = delete_game(&client, &game_name).await;
    assert_eq!(response, Ok(0));

    // Check that the game was deleted
    let response = check_game(&client, &game_name).await;
    assert!(response.is_err());
}

/// Creates a game, adds score, and deletes the game from the database
#[rocket::async_test]
async fn create_add_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let game_id = create_game(&client, &game_name).await.unwrap();

    // Check game
    let response = check_game(&client, &game_name).await;
    assert_eq!(response, Ok(game_id));

    // Add score
    let score_record = ScoreRecord::new(10, None);
    let response = add_score(&client, &game_name, &score_record).await;
    assert!(response.is_ok());

    // Delete the game
    let response = delete_game(&client, &game_name).await;
    assert_eq!(response, Ok(1));
}

/// Creates a game, adds score, fetches score, and deletes the game from the database
#[rocket::async_test]
async fn create_add_get_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let game_id = create_game(&client, &game_name).await.unwrap();

    // Check game
    let response = check_game(&client, &game_name).await;
    assert_eq!(response, Ok(game_id));

    // Add scores
    let scores = vec![
        ScoreRecord::new(10, None),
        ScoreRecord::new(-3, Some("good guy".to_owned())),
        ScoreRecord::new(31, None),
    ];
    let scores_len = scores.len();
    for score_record in &scores {
        let response = add_score(&client, &game_name, &score_record).await;
        assert!(response.is_ok());
    }

    // Fetch score
    let response = get_scores(&client, &game_name).await;
    assert_eq!(response, Ok(scores));

    // Delete the game
    let response = delete_game(&client, &game_name).await;
    assert_eq!(response, Ok(scores_len as u64));
}
