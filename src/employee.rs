use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_plain::derive_display_from_serialize;

#[derive(Deserialize, Debug)]
pub struct Employee {
    #[serde(rename = "employeeName")]
    pub name: String,
    pub role: Role,
    #[serde(rename = "shiftStartDateTime")]
    pub start: NaiveDateTime,
    #[serde(rename = "shiftEndDateTime")]
    pub end: NaiveDateTime,
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
    #[serde(rename(deserialize = "Zizzi Cleaner"))]
    CL,
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
        matches!(self.role, Role::GM | Role::AM | Role::SU | Role::FOH | Role::CL)
    }
}
