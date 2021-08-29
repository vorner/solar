use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Error;
use rayon::prelude::*;
use serde::Deserialize;

use crate::consumption::Requests as ConsumptionRequests;
use crate::plant::Plant;
use crate::reference::{Reference, TimeSlots};

#[derive(Clone, Debug, Deserialize)]
pub struct Site {
    references: HashMap<String, Reference>,
    plant: Plant,
    #[serde(default)]
    consumption: ConsumptionRequests,
}

impl Site {
    pub fn process<P: AsRef<Path>>(def: P) -> Result<(), Error> {
        let input = File::open(def)?;
        let me: Self = serde_yaml::from_reader(input)?;

        dbg!(&me);

        let references: HashMap<String, TimeSlots> = me
            .references
            .into_par_iter()
            .map(|(name, refe)| refe.load().map(|r| (name, r)))
            .collect::<Result<_, _>>()?;

        // Get us a sample from one of them
        let first_ref = references.iter().next().unwrap().1;
        let midnight = first_ref[0].time.date().and_hms(0, 0, 0);
        let starttime = (first_ref[0].time - midnight).num_minutes() as f64 / 60.0;
        let stoptime = (first_ref.last().unwrap().time - midnight).num_minutes() as f64 / 60.0;

        let usages = me
            .consumption
            .into_iter()
            .skip_while(|r| r.start_at < starttime)
            .take_while(|r| r.start_at <= stoptime)
            .collect::<Vec<_>>();

        dbg!(&usages);
        todo!();

        dbg!(&references);

        let plants = me.plant.into_alternatives().collect::<Vec<_>>();
        let reports = plants
            .into_par_iter()
            .map(|plant| {
                let report = plant.simulate(&references);
                (plant, report)
            })
            .collect::<Vec<_>>();

        dbg!(reports);

        todo!()
    }
}
