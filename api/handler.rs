use fourth_ical_rs::{get_shifts, login, shifts_to_ical};
use reqwest::cookie::Jar;
use std::sync::Arc;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let jar = Arc::new(Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(jar)
        .cookie_store(true)
        .build()
        .unwrap();
    login::get_cookie(&client).await;
    let shifts = get_shifts(&client).await?;
    println!("Responding with ical");
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/calendar")
        .body(shifts_to_ical(shifts).into())?)
}
