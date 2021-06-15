use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::{Context, Error};
use chrono::{DateTime, Local, TimeZone};
use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Line {
    time: String,
    power: f32,
    irradiation: f32,
    height: f32,
    temp: f32,
    wind: f32,
    reconstructed: f32,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub(crate) struct TimeSlot {
    pub(crate) time: DateTime<Local>,
    pub(crate) power: f32,
    pub(crate) temp: f32,
}

impl TryFrom<Line> for TimeSlot {
    type Error = Error;

    fn try_from(l: Line) -> Result<Self, Error> {
        let time = Local.datetime_from_str(&l.time, "%Y%m%d:%H%M")
            .with_context(|| format!("Invalid date-time {}", l.time))?;

        Ok(Self {
            time,
            power: l.power / 1000.0,
            temp: l.temp,
        })
    }
}

pub(crate) type TimeSlots = Vec<TimeSlot>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Reference {
    file: PathBuf,
}

impl Reference {
    pub(crate) fn load(&self) -> Result<TimeSlots, Error> {
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(&self.file)
            .with_context(|| format!("Failed to open {}", self.file.display()))?;
        let mut headers = Default::default();
        reader.read_record(&mut headers)
            .with_context(|| format!("Missing headers in {}", self.file.display()))?;
        reader
            .into_deserialize()
            .map(|l: Result<Line, _>| {
                l
                    .map_err(Error::from)
                    .and_then(TimeSlot::try_from)
                    .with_context(|| format!("In input file {}", self.file.display()))
            })
            .collect()
    }
}
