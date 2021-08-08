use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::marker::PhantomData;

use rand::prelude::*;
use serde::de::{Deserializer, Error, Unexpected};
use serde::Deserialize;

trait RangeType {
    fn validate<'d, T, D: Deserializer<'d>>(range: &Range<T>) -> Result<(), D::Error>;
}

#[derive(Clone, Debug)]
struct BasicRange;

impl RangeType for BasicRange {
    fn validate<'d, T, D: Deserializer<'d>>(range: &Range<T>) -> Result<(), D::Error> {
        if range.from > range.to {
            return Err(Error::custom("range bounds crossed"));
        }
        if range.from < 0.0 {
            return Err(Error::invalid_value(
                Unexpected::Float(range.from),
                &"non-negative number",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct DayHours;

impl RangeType for DayHours {
    fn validate<'d, T, D: Deserializer<'d>>(range: &Range<T>) -> Result<(), D::Error> {
        BasicRange::validate::<_, D>(range)?;

        if range.to > 24.0 {
            Err(Error::invalid_value(
                Unexpected::Unsigned(range.to as _),
                &"a hour in a day",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Range<T: ?Sized> {
    from: f64,
    to: f64,
    _type: PhantomData<T>,
}

impl<T> Range<T> {
    fn pick(&self) -> f64 {
        thread_rng().gen_range(self.from..self.to)
    }
}

impl<'d, T: RangeType> Deserialize<'d> for Range<T> {
    fn deserialize<D: Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct UncheckedRange {
            from: f64,
            to: f64,
        }

        let uncheded = UncheckedRange::deserialize(deserializer)?;

        let range = Range {
            from: uncheded.from,
            to: uncheded.to,
            _type: PhantomData,
        };

        T::validate::<T, D>(&range)?;

        if uncheded.from > uncheded.to {
            return Err(Error::custom("crossed range ‒ from bigger than to"));
        }

        Ok(range)
    }
}

fn restrict_whole_day() -> Range<DayHours> {
    Range {
        from: 0.0,
        to: 24.0,
        _type: PhantomData,
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Schedule {
    /// How long to wait after one run ends and another starts.
    interval_hours: Range<BasicRange>,

    #[serde(default = "restrict_whole_day")]
    restrict_hours: Range<DayHours>,

    /// If this hits outside of the restricted hours, it is delayed up to this number of hours
    /// after the first opportunity.
    #[serde(default)]
    delay_up_to: f64,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
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
    power: Range<BasicRange>,

    /// how long it takes in hours
    ///
    /// The total consumption is power * duration
    duration: Range<BasicRange>,

    source: Source,
}

impl Usage {
    fn pick(&self) -> UsedPower {
        let mut result = UsedPower {
            power: self.power.pick(),
            duration: self.duration.pick(),
            source: self.source,
        };
        if result.source == Source::RandomLine {
            result.source = match thread_rng().gen_range(1..=3) {
                1 => Source::Line1,
                2 => Source::Line2,
                3 => Source::Line3,
                _ => unreachable!(),
            };
        }
        result
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    schedule: Option<Schedule>,
    usage: Vec<Usage>,

    #[serde(default)]
    trigger: Vec<Trigger>,

    #[serde(default)]
    delay: Option<Delay>,
}

impl Request {
    fn next_after(&self, end_time: f64) -> f64 {
        let schedule = self
            .schedule
            .as_ref()
            .expect("Called next_after on something without a schedule");

        let delay = schedule.interval_hours.pick();
        let mut start = end_time + delay;

        let part_of_day = start.rem_euclid(24.0);

        let adjust_delay = if part_of_day < schedule.restrict_hours.from {
            Some(schedule.restrict_hours.from - part_of_day)
        } else if part_of_day > schedule.restrict_hours.to {
            Some(schedule.restrict_hours.from + (24.0 - part_of_day))
        } else {
            None
        };

        if let Some(adjust_delay) = adjust_delay {
            let len = schedule.restrict_hours.to - schedule.restrict_hours.from;
            let max_delay = if len < schedule.delay_up_to {
                len
            } else {
                schedule.delay_up_to
            };
            start += adjust_delay + thread_rng().gen_range(0.0..max_delay);
        }

        start
    }

    fn generate_consumption(&self) -> Vec<UsedPower> {
        self.usage.iter().map(Usage::pick).collect()
    }
}

#[derive(Clone)]
struct UsedPower {
    power: f64,
    duration: f64,
    source: Source,
}

#[derive(Clone)]
struct Run<'a> {
    name: &'a Name,
    request: &'a Request,

    start_at: f64,
    end_at: f64,
    triggered: bool,

    consumption: Vec<UsedPower>,
}

impl<'a> Run<'a> {
    fn new(name: &'a Name, request: &'a Request, start_at: f64, triggered: bool) -> Self {
        let consumption = request.generate_consumption();
        let duration: f64 = consumption.iter().map(|p| p.duration).sum();
        Run {
            name,
            request,
            start_at,
            end_at: start_at + duration,
            triggered,
            consumption,
        }
    }
}

impl PartialEq for Run<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.start_at == other.start_at
            && self.triggered == other.triggered
            && self.name == other.name
    }
}

impl Eq for Run<'_> {}

impl PartialOrd for Run<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Run<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_at
            .partial_cmp(&other.start_at)
            .expect("We don't deal with weird floats here")
            .then_with(|| self.triggered.cmp(&other.triggered))
            .then_with(|| self.name.cmp(other.name))
    }
}

/// Bunch of appliances that consume power.
///
/// Can be iterated over. It'll return a sequence of power-consumption events, in increasing order
/// of their starts. It starts at time 0.0 and is potentially infinite.
#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
struct Requests(HashMap<Name, Request>);

impl<'a> IntoIterator for &'a Requests {
    type Item = Run<'a>;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let scheduled = self
            .0
            .iter()
            .filter(|(_, Request { schedule, .. })| schedule.is_some())
            .map(|(name, req)| {
                let when = req.next_after(0.0);
                Run::new(name, req, when, false)
            })
            .collect();

        Iter {
            requests: self,
            scheduled,
        }
    }
}

struct Iter<'a> {
    requests: &'a Requests,

    scheduled: BinaryHeap<Run<'a>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Run<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let scheduled = self.scheduled.pop()?;

        let Run {
            name,
            request,
            end_at,
            triggered,
            ..
        } = scheduled;

        if !triggered {
            let next_time = request.next_after(end_at);
            self.scheduled
                .push(Run::new(name, request, next_time, false));
        }

        for trig in &request.trigger {
            let trig_req = &self.requests.0[&trig.other]; // TODO: Error handling?
            self.scheduled
                .push(Run::new(&trig.other, trig_req, end_at, true));
        }

        Some(scheduled)
    }
}
