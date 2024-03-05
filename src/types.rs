use std::ops::Deref;

use chrono::{DateTime, Utc};
use strum::EnumIter;

mod datetime_without_seconds {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%MZ";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

#[derive(Debug,serde::Deserialize)]
pub struct Data<T> {
    pub data: Vec<T>,
}

impl<T> Deref for Data<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug,serde::Deserialize,EnumIter)]
#[serde(rename_all="lowercase")]
pub enum PowerType {
    Gas,
    Coal,
    Biomass,
    Nuclear,
    Hydro,
    Imports,
    Other,
    Wind,
    Solar,
}

#[derive(Debug,serde::Deserialize)]
pub struct GenerationMix {
    pub fuel: PowerType,
    #[serde(rename = "perc")]
    pub percent: f64,
}

#[derive(Debug,serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntensityIndex {
    #[serde(rename="very low")]
    VeryLow,
    Low,
    Moderate,
    High,
    #[serde(rename="very high")]
    VeryHigh,
}

impl IntensityIndex {
    pub fn metric_value(&self) -> u8 {
        match self {
            IntensityIndex::VeryLow => 0,
            IntensityIndex::Low => 1,
            IntensityIndex::Moderate => 2,
            IntensityIndex::High => 3,
            IntensityIndex::VeryHigh => 4,
        }
    }
}

#[derive(Debug,serde::Deserialize)]
pub struct IntensitySummary {
    pub forecast: f64,
    pub index: IntensityIndex,
}

#[derive(Debug,serde::Deserialize)]
pub struct IntensityData {
    #[serde(with = "datetime_without_seconds")]
    pub from: DateTime<Utc>,
    #[serde(with = "datetime_without_seconds")]
    pub to: DateTime<Utc>,
    pub intensity: IntensitySummary,
    #[serde(rename="generationmix")]
    pub generation_mix: Vec<GenerationMix>,
}

#[derive(Debug,serde::Deserialize)]
pub struct Region {
    #[serde(rename = "regionid")]
    pub region_id: u64,
    pub dnoregion: String,
    pub shortname: String,
    pub postcode: String,
    #[serde(rename = "data")]
    pub region_data: Vec<IntensityData>,
}
