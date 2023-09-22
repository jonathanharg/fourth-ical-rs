mod employee;
mod login;
mod shift;
use axum::{routing::get, Router};
use chrono::{Datelike, Duration, Utc};
use employee::Employee;
use icalendar::{Calendar, Component, Event, EventLike};
use reqwest::{cookie::Jar, Client};
use shift::Shift;
use std::sync::Arc;
use urlencoding::encode;

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
        "/work.ical",
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

fn shift_api_url() -> String {
    // Format YYYY/MM/DD
    let now = Utc::now();
    let from = now.checked_sub_signed(Duration::weeks(2)).unwrap();
    let from = &format!("{}/{}/{}", from.year(), from.month(), from.day());
    let to = now.checked_add_signed(Duration::weeks(4)).unwrap();
    let to = &format!("{}/{}/{}", to.year(), to.month(), to.day());
    let from = encode(from);
    let to = encode(to);
    format!("https://api.fourth.com/api/myschedules/schedule?%24orderby=StartDateTime+asc&%24top=50&fromDate={}&toDate={}", from, to)
}

async fn get_shifts(client: &Client) -> Result<Vec<Shift>, reqwest::Error> {
    let url = shift_api_url();
    let mut shifts: Vec<Shift> = client
        .get(url)
        .send()
        .await
        .expect("Expected a result")
        .json::<serde_json::Value>()
        .await?
        .get("entities")
        .expect("Results should have an entities json")
        .as_array()
        .expect("Entities should be an array")
        .iter()
        .map(|s| {
            serde_json::from_value(
                s.get("properties")
                    .expect("Entities should have a properties")
                    .clone(),
            )
            .unwrap()
        })
        .collect();

    for shift in &mut shifts {
        shift.get_working_with(client).await;
    }

    Ok(shifts)
}

fn shifts_to_ical(shifts: Vec<Shift>) -> String {
    let mut calendar = Calendar::new();
    for shift in shifts {
        let event = Event::new()
            .summary("Zizzi Shift")
            .description(&shift.generate_description())
            .starts(shift.start.naive_utc())
            .ends(shift.end.naive_utc())
            .location(&shift.location)
            .done();
        calendar.push(event);
    }
    calendar.to_string()
}
