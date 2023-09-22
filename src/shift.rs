use crate::Employee;
use chrono::{DateTime, NaiveTime, Timelike, Utc};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Shift {
    #[serde(rename = "itemId")]
    pub id: u32,
    #[serde(rename = "startDateTime")]
    pub start: DateTime<Utc>,
    #[serde(rename = "endDateTime")]
    pub end: DateTime<Utc>,
    #[serde(rename = "locationName")]
    pub location: String,
    #[serde(rename = "roleName")]
    role: String,
    message: String,
    #[serde(skip)]
    pub working_with: Vec<Employee>,
}

impl Shift {
    fn section_totals(&self) -> (i32, i32, i32, i32) {
        let mut foh_lunch_total = 0;
        let mut foh_dinner_total = 0;
        let mut boh_lunch_total = 0;
        let mut boh_dinner_total = 0;
        let lunch = NaiveTime::from_hms_opt(13, 30, 0).unwrap();
        let dinner = NaiveTime::from_hms_opt(18, 30, 0).unwrap();

        for colleague in &self.working_with {
            if (colleague.start.time() < lunch) && (lunch < colleague.end.time()) {
                if colleague.is_foh() {
                    foh_lunch_total += 1
                } else {
                    boh_lunch_total += 1
                };
            }
            if (colleague.start.time() < dinner) && (dinner < colleague.end.time()) {
                if colleague.is_foh() {
                    foh_dinner_total += 1
                } else {
                    boh_dinner_total += 1
                };
            }
        }

        (
            foh_lunch_total,
            foh_dinner_total,
            boh_lunch_total,
            boh_dinner_total,
        )
    }

    fn format_time(&self) -> String {
        format!(
            "{}:{:02} - {}:{:02}",
            self.start.hour(),
            self.start.minute(),
            self.end.hour(),
            self.end.minute()
        )
    }
    fn time_diff(&self) -> String {
        let diff = self.end.signed_duration_since(self.start);
        let mins = diff.num_minutes() % 60;
        if mins == 0 {
            return format!("{}h", diff.num_hours());
        }
        format!("{}h {}m", diff.num_hours(), mins)
    }

    pub fn generate_description(&self) -> String {
        let my_times = self.format_time();
        let my_length = self.time_diff();
        let my_role = &self.role;
        let message = {
            if !self.message.is_empty() {
                format!("\n{}", &self.message)
            } else {
                String::from("")
            }
        };

        let foh = self
            .working_with
            .iter()
            .filter(|p| p.is_foh())
            .map(|c| {
                format!(
                    "{} - {} {} {}",
                    c.start.format("%H:%M"),
                    c.end.format("%H:%M"),
                    c.name,
                    c.role
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let boh = self
            .working_with
            .iter()
            .filter(|p| !p.is_foh())
            .map(|c| {
                format!(
                    "{} - {} {} {}",
                    c.start.format("%H:%M"),
                    c.end.format("%H:%M"),
                    c.name,
                    c.role
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let (fl, fd, bl, bd) = self.section_totals();

        let now = Utc::now();

        format!(
            r#"{my_times} ({my_length})
{my_role}{message}

FOH ({fl} lunch, {fd} dinner):
{foh}

BOH ({bl} lunch, {bd} dinner):
{boh}

Last updated {now}.
"#
        )
    }

    pub async fn get_working_with(&mut self, client: &Client) -> &Vec<Employee> {
        let url = format!(
            "https://api.fourth.com/api/myschedules/shifts/{}/workingwith",
            self.id
        );
        self.working_with = client
            .get(url)
            .send()
            .await
            .expect("Expected a result")
            .json::<serde_json::Value>()
            .await
            .expect("Expected json response")
            .map(|p| {
                print!("JSON RESPONSE:");
                print!("{:#?}", p);
                p
            })
            .as_array()
            .expect("Colleagues should be an array")
            .iter()
            .map(|p| serde_json::from_value(p.clone()).expect("Expected person"))
            .collect();
        self.working_with.sort_by(|a, b| a.start.cmp(&b.start));
        self.working_with.sort_by(|a, b| a.name.cmp(&b.name));
        self.working_with.sort_by(|a, b| a.role.cmp(&b.role));
        &self.working_with
    }
}
