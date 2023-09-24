use std::sync::Arc;
use axum::{routing::get, Router};
use fourth_ical_rs::{login, get_shifts, shifts_to_ical};
use reqwest::cookie::Jar;


#[tokio::main]
async fn main() {
    let jar = Arc::new(Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(jar)
        .cookie_store(true)
        .build()
        .unwrap();
    login::get_cookie(&client).await;
    println!("Created client");
    let app = Router::new().route(
        "/api/calendar",
        get(|| async {
            let client = client;
            println!("GET Request for shifts");
            match get_shifts(&client).await {
                Ok(shifts) => {
                    println!("Responding with ical");
                    shifts_to_ical(shifts)
                }
                Err(_) => {
                    login::get_cookie(&client).await;
                    let shifts = get_shifts(&client)
                        .await
                        .expect("Expected second attempt will log in");
                    println!("Responding with ical");
                    shifts_to_ical(shifts)
                }
            }
        }),
    );

    println!("Listening for requests on 0.0.0.0:7878/work.ical...");
    axum::Server::bind(&"0.0.0.0:7878".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}