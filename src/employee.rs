use chrono::{DateTime, Utc, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_plain::derive_display_from_serialize;

#[derive(Deserialize, Debug)]
pub struct Employee {
    #[serde(rename = "employeeName")]
    pub name: String,
    pub role: Role,
    #[serde(rename = "shiftStartDateTime", with = "date_format")]
    pub start: DateTime<FixedOffset>,
    #[serde(rename = "shiftEndDateTime", with = "date_format")]
    pub end: DateTime<FixedOffset>,
}

mod date_format {
    use chrono::{DateTime, FixedOffset};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[allow(clippy::upper_case_acronyms)]
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

impl Employee {
    pub fn is_foh(&self) -> bool {
        matches!(self.role, Role::GM | Role::AM | Role::SU | Role::FOH)
    }
}
