#![allow(dead_code, unused_variables)]
mod login;
mod colleagues;
use axum::{routing::get, Router};
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use colleagues::Person;
use icalendar::{Calendar, Component, Event, EventLike};
use reqwest::cookie::Jar;
use reqwest::Client;
use serde::{Deserialize};
use std::sync::Arc;
use time::OffsetDateTime;
use urlencoding::encode;

#[derive(Deserialize, Debug)]
struct Shift {
    #[serde(rename = "itemId")]
    id: u32,
    #[serde(with = "time::serde::rfc3339", rename = "startDateTime")]
    start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339", rename = "endDateTime")]
    end: OffsetDateTime,
    #[serde(rename = "locationName")]
    location: String,
    #[serde(rename = "roleName")]
    role: String,
    message: String,
    #[serde(skip)]
    working_with: Vec<Person>,
}

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
                    return shifts_to_ical(shifts);
                }
                Err(_) => {
                    login::get_cookie(&client).await;
                    let shifts = get_shifts(&client)
                        .await
                        .expect("Expected second attempt will log in");
                    println!("Responding with ical");
                    return shifts_to_ical(shifts);
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

// async fn get_shifts(client: &Client) -> Result<Vec<Shift>, Box<dyn Error>> {
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
        shift.working_with = colleagues::working_with(&client, &shift.id).await;
    }

    Ok(shifts)
}

fn short_time(start_time: &DateTime<Utc>, end_time: &DateTime<Utc>) -> String {
    format!(
        "{}:{:02} - {}:{:02}",
        start_time.hour(),
        start_time.minute(),
        end_time.hour(),
        end_time.minute()
    )
}

fn time_diff(start: &DateTime<Utc>, end: &DateTime<Utc>) -> String {
    let diff = end.signed_duration_since(*start);
    let mins = diff.num_minutes() % 60;
    if mins == 0 {
        return format!("{}h", diff.num_hours());
    }
    format!("{}h {}m", diff.num_hours(), mins)
}

fn shifts_to_ical(shifts: Vec<Shift>) -> String {
    let mut calendar = Calendar::new();
    for shift in shifts {
        let starts = Utc.timestamp_opt(shift.start.unix_timestamp(), 0).unwrap();
        let ends = Utc.timestamp_opt(shift.end.unix_timestamp(), 0).unwrap();
        let desc = generate_description(&shift);
        let event = Event::new()
            .summary("Zizzi Shift")
            .description(&*generate_description(&shift))
            .starts(starts)
            .ends(ends)
            .location(&shift.location)
            .done();
        calendar.push(event);
    }
    calendar.to_string()
}

fn generate_description(shift: &Shift) -> String {
    // 17:00 - 21:00 (3h 40m)
    // Role
    // Message
    //
    // FOH:
    // 12:00 - 15:00    First Last
    // 17:00 - 21:00    Role
    // 13:00 - 18:00    First Last
    //                  Role
    let starts = Utc.timestamp_opt(shift.start.unix_timestamp(), 0).unwrap();
    let ends = Utc.timestamp_opt(shift.end.unix_timestamp(), 0).unwrap();
    let my_times = short_time(&starts, &ends);
    let my_length = time_diff(&starts, &ends);
    let my_role = &shift.role;
    let message = {
        if shift.message != "" {
            format!("\n{}", &shift.message)
        } else {
            String::from("")
        }
    };
    let colleagues = shift
        .working_with
        // .sort_by(|a, b| a.role.cmp(&b.role))
        .iter()
        .map(|c| format!("{}\n{}", c.name, c.role))
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        r#"{my_times} ({my_length})
{my_role}{message}

Working with:
{colleagues}
"#
    )
}