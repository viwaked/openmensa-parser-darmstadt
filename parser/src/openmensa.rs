use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "openmensa")]
pub struct OpenMensa {
    #[serde(rename = "@version")]
    pub version: String, // required, 2.0 or 2.1
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub parser_version: Option<String>,
    pub canteen: Canteen,
}

impl OpenMensa {
    pub fn serialize_to_string(&self) -> anyhow::Result<String> {
        let mut buf = Vec::<u8>::new();
        let cursor = std::io::Cursor::new(&mut buf);
        let mut writer = quick_xml::Writer::new(cursor);

        writer
            .create_element("openmensa")
            .with_attributes([
                ("xmlns", "http://openmensa.org/open-mensa-v2"),
                ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
                (
                    "xsi:schemaLocation",
                    "http://openmensa.org/open-mensa-v2 http://openmensa.org/open-mensa-v2.xsd",
                ),
                ("version", self.version.as_str()),
            ])
            .write_inner_content(|w| {
                if let Some(parser_version) = &self.parser_version {
                    w.write_serializable("version", parser_version)
                        .map_err(|e| {
                            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                        })?;
                }

                w.write_serializable("canteen", &self.canteen)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

                Ok(())
            })?;

        Ok(String::from_utf8(buf)?)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Canteen {
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "address", skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    #[serde(rename = "city", skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    #[serde(rename = "phone", skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    #[serde(rename = "email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(rename = "location", skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,

    #[serde(rename = "availability", skip_serializing_if = "Option::is_none")]
    pub availability: Option<Availability>,

    #[serde(rename = "times", skip_serializing_if = "Option::is_none")]
    pub times: Option<Times>,

    #[serde(rename = "feed", default)]
    pub feeds: Vec<Feed>,

    #[serde(rename = "day", default)]
    pub days: Vec<Day>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "@latitude")]
    pub latitude: f32,
    #[serde(rename = "@longitude")]
    pub longitude: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Availability {
    Public,
    Restricted,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Times {
    #[serde(rename = "@type")]
    pub kind: String, // must be "opening"

    #[serde(rename = "monday", skip_serializing_if = "Option::is_none")]
    pub monday: Option<Weekday>,

    #[serde(rename = "tuesday", skip_serializing_if = "Option::is_none")]
    pub tuesday: Option<Weekday>,

    #[serde(rename = "wednesday", skip_serializing_if = "Option::is_none")]
    pub wednesday: Option<Weekday>,

    #[serde(rename = "thursday", skip_serializing_if = "Option::is_none")]
    pub thursday: Option<Weekday>,

    #[serde(rename = "friday", skip_serializing_if = "Option::is_none")]
    pub friday: Option<Weekday>,

    #[serde(rename = "saturday", skip_serializing_if = "Option::is_none")]
    pub saturday: Option<Weekday>,

    #[serde(rename = "sunday", skip_serializing_if = "Option::is_none")]
    pub sunday: Option<Weekday>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Weekday {
    #[serde(rename = "@open", skip_serializing_if = "Option::is_none")]
    pub open: Option<String>, // HH:mm-HH:mm

    #[serde(rename = "@closed", skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@priority", skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "source", skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(rename = "schedule", skip_serializing_if = "Option::is_none")]
    pub schedule: Option<Schedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    #[serde(rename = "@hour")]
    pub hour: String,

    #[serde(rename = "@minute", skip_serializing_if = "Option::is_none")]
    pub minute: Option<String>,

    #[serde(rename = "@dayOfWeek", skip_serializing_if = "Option::is_none")]
    pub day_of_week: Option<String>,

    #[serde(rename = "@dayOfMonth", skip_serializing_if = "Option::is_none")]
    pub day_of_month: Option<String>,

    #[serde(rename = "@month", skip_serializing_if = "Option::is_none")]
    pub month: Option<String>,

    #[serde(rename = "@retry", skip_serializing_if = "Option::is_none")]
    pub retry: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Day {
    #[serde(rename = "@date")]
    pub date: String, // YYYY-MM-DD

    // Use enum for xs:choice: either categories or closed
    #[serde(flatten)]
    pub content: DayContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DayContent {
    Open { category: Vec<Category> },
    Closed { closed: Empty },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Empty {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    #[serde(rename = "@name")]
    pub name: String,

    pub meal: Vec<Meal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meal {
    pub name: String,

    #[serde(rename = "note", default)]
    pub notes: Vec<String>,

    #[serde(rename = "price", default)]
    pub prices: Vec<Price>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
    #[serde(rename = "$text")]
    pub value: f32,

    #[serde(rename = "@role")]
    pub role: PriceRole,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriceRole {
    Pupil,
    Student,
    Employee,
    Other,
}
