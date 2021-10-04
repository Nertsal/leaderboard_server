use rocket::{
    http::{Header, Status},
    local::asynchronous::{Client, LocalResponse},
};

use crate::{
    database::{GameId, ScoreRecord},
    database_keys::GameKeys,
};

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
async fn create_game(client: &Client, game_name: &String) -> Result<(GameId, GameKeys), String> {
    let response = client.post("/games").json(game_name).dispatch().await;
    if response.status() != Status::Ok {
        return Err(response.into_string().await.unwrap());
    }

    let result = deserialize_response::<(GameId, GameKeys)>(response)
        .await
        .unwrap();
    Ok(result)
}

/// Deletes a game called `game_name` from the database
/// and returns whether it was in the database
async fn delete_game(client: &Client, game_name: &str, api_key: &str) -> Result<u64, String> {
    let uri = format!("/games/{}", game_name);
    let response = client
        .delete(&uri)
        .header(Header::new("api-key", api_key.to_owned()))
        .dispatch()
        .await;
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
    api_key: &str,
) -> Result<(), String> {
    let uri = format!("/games/{}/scores", game_name);
    let response = client
        .post(&uri)
        .header(Header::new("api-key", api_key.to_owned()))
        .json(score_record)
        .dispatch()
        .await;
    if response.status() != Status::Ok {
        return Err(response.into_string().await.unwrap());
    }

    Ok(())
}

/// Gets scores from the database under game called `game_name`.
async fn get_scores(
    client: &Client,
    game_name: &str,
    api_key: &str,
) -> Result<Vec<ScoreRecord>, String> {
    let uri = format!("/games/{}/scores", game_name);
    let response = client
        .get(&uri)
        .header(Header::new("api-key", api_key.to_owned()))
        .dispatch()
        .await;
    if response.status() != Status::Ok {
        return Err(response.into_string().await.unwrap());
    }

    let scores = deserialize_response::<Vec<ScoreRecord>>(response)
        .await
        .unwrap();
    Ok(scores)
}

const TEST_GAME_NAME: &'static str = "test_game";

/// Creates and deletes a game in the database
#[rocket::async_test]
async fn create_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let (_, game_keys) = create_game(&client, &game_name).await.unwrap();

    // Delete the game
    let response = delete_game(&client, &game_name, game_keys.admin_key.inner()).await;
    assert_eq!(response, Ok(0));
}

/// Creates a game, adds score, and deletes the game from the database
#[rocket::async_test]
async fn create_add_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let (_, game_keys) = create_game(&client, &game_name).await.unwrap();

    // Add score
    let score_record = ScoreRecord::new(10, None);
    let response = add_score(
        &client,
        &game_name,
        &score_record,
        game_keys.write_key.inner(),
    )
    .await;
    assert!(response.is_ok());

    // Delete the game
    let response = delete_game(&client, &game_name, game_keys.admin_key.inner()).await;
    assert_eq!(response, Ok(1));
}

/// Creates a game, adds score, fetches score, and deletes the game from the database
#[rocket::async_test]
async fn create_add_get_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let (_, game_keys) = create_game(&client, &game_name).await.unwrap();

    // Add scores
    let scores = vec![
        ScoreRecord::new(10, None),
        ScoreRecord::new(-3, Some("good guy".to_owned())),
        ScoreRecord::new(31, None),
    ];
    let scores_len = scores.len();
    for score_record in &scores {
        let response = add_score(
            &client,
            &game_name,
            &score_record,
            game_keys.admin_key.inner(),
        )
        .await;
        assert!(response.is_ok());
    }

    // Fetch score
    let response = get_scores(&client, &game_name, game_keys.read_key.inner()).await;
    assert_eq!(response, Ok(scores));

    // Delete the game
    let response = delete_game(&client, &game_name, game_keys.admin_key.inner()).await;
    assert_eq!(response, Ok(scores_len as u64));
}
