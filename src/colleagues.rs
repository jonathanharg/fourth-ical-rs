use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_plain::derive_display_from_serialize;

#[derive(Deserialize, Debug)]
pub struct Person {
    id: u32,
    #[serde(rename = "employeeName")]
    pub name: String,
    pub role: Role,
    // #[serde(with = "time::serde::rfc3339", rename = "shiftStartDateTime")]
    // start: OffsetDateTime,
    // #[serde(with = "time::serde::rfc3339", rename = "shiftEndDateTime")]
    // end: OffsetDateTime,
    #[serde(rename = "sameRole")]
    same_role: bool,
    #[serde(rename = "sameShift")]
    same_shift: bool,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum Role {
    #[serde(rename = "Zizzi General Manager")]
    GM,
    #[serde(rename = "Zizzi Assistant Manager")]
    AM,
    #[serde(rename = "Zizzi Supervisor")]
    SU,
    #[serde(rename = "Zizzi FOH Team Member")]
    FOH,
    #[serde(rename = "Zizzi Head Chef")]
    HC,
    #[serde(rename = "Zizzi Assistant Chef")]
    AC,
    #[serde(rename = "Zizzi Section Chef 3")]
    C3,
    #[serde(rename = "Zizzi Section Chef 2")]
    C2,
    #[serde(rename = "Zizzi Section Chef 1")]
    C1,
}

fn is_foh(role: &Role) -> bool {
    matches!(role, Role::GM | Role::AM | Role::SU | Role::FOH)
}

derive_display_from_serialize!(Role);

pub async fn working_with(client: &Client, shift_id: &u32) -> Vec<Person> {
    let url = format!(
        "https://api.fourth.com/api/myschedules/shifts/{}/workingwith",
        shift_id
    );
    client
        .get(url)
        .send()
        .await
        .expect("Expected a result")
        .json::<serde_json::Value>()
        .await
        .expect("Expected json response")
        .as_array()
        .expect("Colleagues should be an array")
        .iter()
        .map(|p| serde_json::from_value(p.clone()).expect("Expected person"))
        .collect()
}

