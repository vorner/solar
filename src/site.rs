use std::fs::File;
use std::ops::AddAssign;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Error;
use itertools::{iproduct, Itertools};
use serde::{Deserialize, Serialize};

use crate::alternatives::Alternatives;

#[derive(Clone, Debug, Serialize)]
pub struct Report {}

#[derive(Copy, Clone, Debug, Default, Deserialize)]
struct ExtraPower {
    power: usize,
    price: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Panels {
    power: usize,
    price: usize,
    reference: Arc<PathBuf>,
}

impl AddAssign<ExtraPower> for Panels {
    fn add_assign(&mut self, rhs: ExtraPower) {
        self.power += rhs.power;
        self.price += rhs.price;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Invertor {
    power: usize,
    price: usize,
    asymetric: bool,
}

impl AddAssign<ExtraPower> for Invertor {
    fn add_assign(&mut self, rhs: ExtraPower) {
        self.power += rhs.power;
        self.price += rhs.price;
    }
}

#[derive(Debug)]
struct PlantAlternative {
    invertor: Invertor,
    arrays: Vec<Panels>,
}

#[derive(Clone, Debug, Deserialize)]
struct Plant {
    invertor: Alternatives<Invertor, ExtraPower>,

    arrays: Vec<Alternatives<Panels, ExtraPower>>,
}

impl Plant {
    fn into_alternatives(self) -> impl Iterator<Item = PlantAlternative> {
        iproduct!(self.invertor, self.arrays.into_iter().multi_cartesian_product())
            .map(|(invertor, arrays)| PlantAlternative { invertor, arrays })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Site {
    plant: Plant,
}

impl Site {
    pub fn process<P: AsRef<Path>>(def: P) -> Result<Report, Error> {
        let input = File::open(def)?;
        let me: Self = serde_yaml::from_reader(input)?;

        for plant in me.plant.into_alternatives() {
            println!("{:#?}", plant);
        }

        todo!()
    }
}
