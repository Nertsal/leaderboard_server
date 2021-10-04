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
async fn create_game<'a>(
    client: &'a Client,
    game_name: &String,
) -> Result<(GameId, GameKeys), LocalResponse<'a>> {
    let response = client
        .post("/games/create")
        .body(game_name)
        .dispatch()
        .await;
    if response.status() != Status::Ok {
        return Err(response);
    }

    let result = deserialize_response::<(GameId, GameKeys)>(response)
        .await
        .unwrap();
    Ok(result)
}

/// Deletes a game called `game_name` from the database
/// and returns whether it was in the database
async fn delete_game<'a>(
    client: &'a Client,
    uri: &'a str,
    api_key: &str,
) -> Result<u64, LocalResponse<'a>> {
    let response = client
        .delete(uri)
        .header(Header::new("api-key", api_key.to_owned()))
        .dispatch()
        .await;
    if response.status() != Status::Ok {
        return Err(response);
    }

    let score_count = deserialize_response::<u64>(response).await.unwrap();
    Ok(score_count)
}

/// Add score to the database under game called `game_name`.
/// Returns false if such game does not exist.
async fn add_score<'a>(
    client: &'a Client,
    uri: &'a str,
    score_record: &ScoreRecord,
    api_key: &str,
) -> Result<(), LocalResponse<'a>> {
    let response = client
        .post(uri)
        .header(Header::new("api-key", api_key.to_owned()))
        .json(score_record)
        .dispatch()
        .await;
    if response.status() != Status::Ok {
        return Err(response);
    }

    Ok(())
}

/// Gets scores from the database under game called `game_name`.
async fn get_scores<'a>(
    client: &'a Client,
    uri: &'a str,
    api_key: &str,
) -> Result<Vec<ScoreRecord>, LocalResponse<'a>> {
    let response = client
        .get(uri)
        .header(Header::new("api-key", api_key.to_owned()))
        .dispatch()
        .await;
    if response.status() != Status::Ok {
        return Err(response);
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
    let game_uri = format!("/games/{}", game_name);
    let (_, game_keys) = create_game(&client, &game_name).await.unwrap();

    // Fail to delete the game
    let response = delete_game(&client, &game_uri, "thatisarandomkey").await;
    let response = response.unwrap_err();
    assert_eq!(response.status(), Status::Unauthorized);

    // Delete the game
    let response = delete_game(&client, &game_uri, game_keys.admin_key.inner()).await;
    assert_eq!(response.unwrap(), 0);
}

/// Creates a game, adds score, and deletes the game from the database
#[rocket::async_test]
async fn create_add_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let game_uri = format!("/games/{}", game_name);
    let (_, game_keys) = create_game(&client, &game_name).await.unwrap();

    let score_record = ScoreRecord::new(10, None);
    let score_uri = format!("{}/leaderboard", game_uri);

    // Fail to add score
    let response = add_score(&client, &score_uri, &score_record, "thatisarandomkey").await;
    let response = response.unwrap_err();
    assert_eq!(response.status(), Status::Unauthorized);

    // Add score
    let response = add_score(
        &client,
        &score_uri,
        &score_record,
        game_keys.write_key.inner(),
    )
    .await;
    assert!(response.is_ok());

    // Delete the game
    let response = delete_game(&client, &game_uri, game_keys.admin_key.inner()).await;
    assert_eq!(response.unwrap(), 1);
}

/// Creates a game, adds score, fetches score, and deletes the game from the database
#[rocket::async_test]
async fn create_add_get_delete_game() {
    let client = spawn_client().await;

    // Create a game
    let game_name = TEST_GAME_NAME.to_owned();
    let game_uri = format!("/games/{}", game_name);
    let (_, game_keys) = create_game(&client, &game_name).await.unwrap();

    let scores = vec![
        ScoreRecord::new(10, None),
        ScoreRecord::new(-3, Some("good guy".to_owned())),
        ScoreRecord::new(31, None),
    ];
    let score_uri = format!("{}/leaderboard", game_uri);

    // Fail to add scores
    let response = add_score(&client, &score_uri, &scores[0], game_keys.read_key.inner()).await;
    let response = response.unwrap_err();
    assert_eq!(response.status(), Status::Forbidden);

    // Add scores
    let scores_len = scores.len();
    for score_record in &scores {
        let response = add_score(
            &client,
            &score_uri,
            &score_record,
            game_keys.admin_key.inner(),
        )
        .await;
        assert!(response.is_ok());
    }

    // Fetch score
    let response = get_scores(&client, &score_uri, game_keys.read_key.inner()).await;
    assert_eq!(response.unwrap(), scores);

    // Delete the game
    let response = delete_game(&client, &game_uri, game_keys.admin_key.inner()).await;
    assert_eq!(response.unwrap(), scores_len as u64);
}
