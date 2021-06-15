use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Error;
use rayon::prelude::*;
use serde::Deserialize;

use crate::plant::Plant;
use crate::reference::{Reference, TimeSlots};

#[derive(Clone, Debug, Deserialize)]
pub struct Site {
    references: HashMap<String, Reference>,
    plant: Plant,
}

impl Site {
    pub fn process<P: AsRef<Path>>(def: P) -> Result<(), Error> {
        let input = File::open(def)?;
        let me: Self = serde_yaml::from_reader(input)?;

        let references: HashMap<String, TimeSlots> = me
            .references
            .into_par_iter()
            .map(|(name, refe)| refe.load().map(|r| (name, r)))
            .collect::<Result<_, _>>()?;

        dbg!(&references);

        let plants = me.plant.into_alternatives().collect::<Vec<_>>();
        let reports = plants.into_par_iter().map(|plant| {
            let report = plant.simulate(&references);
            (plant, report)
        }).collect::<Vec<_>>();

        dbg!(reports);

        todo!()
    }
}
