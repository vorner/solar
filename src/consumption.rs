use std::collections::HashMap;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de::{Deserializer, Error, Unexpected};

trait RangeType {
    fn validate<'d, T, D: Deserializer<'d>>(range: &Range<T>) -> Result<(), D::Error>;
}

#[derive(Clone, Debug)]
struct BasicRange;

impl RangeType for BasicRange {
    fn validate<'d, T, D: Deserializer<'d>>(range: &Range<T>) -> Result<(), D::Error> {
        if range.from > range.to {
            Err(Error::custom("range bounds crossed"))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
struct DayHours;

impl RangeType for DayHours {
    fn validate<'d, T, D: Deserializer<'d>>(range: &Range<T>) -> Result<(), D::Error> {
        BasicRange::validate::<_, D>(range)?;

        if range.to > 24 {
            Err(Error::invalid_value(Unexpected::Unsigned(range.to as _), &"a hour in a day"))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
struct Range<T: ?Sized> {
    from: usize,
    to: usize,
    _type: PhantomData<T>,
}

impl<'d, T: RangeType> Deserialize<'d> for Range<T> {
    fn deserialize<D: Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct UncheckedRange {
            from: usize,
            to: usize,
        }

        let uncheded = UncheckedRange::deserialize(deserializer)?;

        let range = Range {
            from: uncheded.from,
            to: uncheded.to,
            _type: PhantomData,
        };

        T::validate::<T, D>(&range)?;

        if uncheded.from > uncheded.to {
            return Err(Error::custom("crossed range ‒ from bigger than to"))
        }

        Ok(range)
    }
}

fn restrict_whole_day() -> Vec<Range<DayHours>> {
    vec![
        Range {
            from: 0,
            to: 24,
            _type: PhantomData,
        }
    ]
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Schedule {
    interval_hours: Range<BasicRange>,

    #[serde(default = "restrict_whole_day")]
    restrict_hours: Vec<Range<DayHours>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Source {
    Line1,
    Line2,
    Line3,
    /// Generator picks randomly which line this is connected to during each run.
    RandomLine,
    /// Smart selection from which line (or multiple ones) the power is taken.
    AnyLine,
    /// Taken from all 3 lines in equal way.
    AllLines,

    /// Doesn't take electricity, but hot water (the equivalent power)
    HotWater,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Usage {
    /// In watts
    power_kwh: Range<BasicRange>,

    /// In minutes
    minutes: Range<BasicRange>,

    source: Source,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
struct Name(String);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Trigger {
    other: Name,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Delay {
    max_hours: usize,
    max_instances: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct Request {
    schedule: Vec<Schedule>,
    usage: Vec<Usage>,

    #[serde(default)]
    trigger: Vec<Trigger>,

    #[serde(default)]
    delay: Option<Delay>,
}

type Requests = HashMap<Name, Request>;
