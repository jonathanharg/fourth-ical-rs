use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_plain::derive_display_from_serialize;

#[derive(Deserialize, Debug)]
pub struct Person {
    id: u32,
    #[serde(rename = "employeeName")]
    pub name: String,
    pub role: Role,
    #[serde(rename = "shiftStartDateTime", with = "colleague_date_format")]
    pub start: DateTime<Utc>,
    #[serde(rename = "shiftEndDateTime", with = "colleague_date_format")]
    pub end: DateTime<Utc>,
    #[serde(rename = "sameRole")]
    same_role: bool,
    #[serde(rename = "sameShift")]
    same_shift: bool,
}

mod colleague_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum Role {
    #[serde(rename(deserialize = "Zizzi General Manager"))]
    GM,
    #[serde(rename(deserialize = "Zizzi Assistant Manager"))]
    AM,
    #[serde(rename(deserialize = "Zizzi Supervisor"))]
    SU,
    #[serde(rename(deserialize = "Zizzi FOH Team Member"))]
    FOH,
    #[serde(rename(deserialize = "Zizzi Head Chef"))]
    HC,
    #[serde(rename(deserialize = "Zizzi Assistant Chef"))]
    AC,
    #[serde(rename(deserialize = "Zizzi Section Chef 3"))]
    C3,
    #[serde(rename(deserialize = "Zizzi Section Chef 2"))]
    C2,
    #[serde(rename(deserialize = "Zizzi Section Chef 1"))]
    C1,
}

derive_display_from_serialize!(Role);

impl Person {
    pub fn is_foh(&self) -> bool {
        matches!(self.role, Role::GM | Role::AM | Role::SU | Role::FOH)
    }
}


