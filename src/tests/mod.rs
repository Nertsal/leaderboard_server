use rocket::{http::Status, local::blocking::Client};

#[test]
fn hello_world() {
    let client = Client::tracked(super::rocket()).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Hello, world!".to_owned()));
}

#[test]
fn add_score() {
    let client = Client::tracked(super::rocket()).expect("valid rocket instance");
    let response = client.post("/leaderboard").json(&(10u32)).dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn get_leaderboard() {
    let client = Client::tracked(super::rocket()).expect("valid rocket instance");

    let response = client.post("/leaderboard").json(&(10u32)).dispatch();
    assert_eq!(response.status(), Status::Ok);

    let response = client.get("/leaderboard").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response
            .into_json::<crate::leaderboard::Leaderboard<u32>>()
            .expect("failed to deserialize leaderboard")
            .iter()
            .collect::<Vec<_>>(),
        vec![&10]
    );
}
