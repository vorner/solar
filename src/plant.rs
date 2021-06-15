use std::collections::HashMap;
use std::ops::AddAssign;
use std::sync::Arc;
use std::path::PathBuf;

use itertools::{iproduct, Itertools};
use serde::{Deserialize, Serialize};

use crate::alternatives::Alternatives;
use crate::reference::TimeSlots;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Report {
    price: usize,
}

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
pub(crate) struct PlantAlternative {
    invertor: Invertor,
    arrays: Vec<Panels>,
}

impl PlantAlternative {
    pub(crate) fn simulate(&self, references: &HashMap<String, TimeSlots>) -> Report {
        let price = self.invertor.price + self.arrays.iter().map(|a| a.price).sum::<usize>();


        Report {
            price
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Plant {
    invertor: Alternatives<Invertor, ExtraPower>,

    arrays: Vec<Alternatives<Panels, ExtraPower>>,
}

impl Plant {
    pub(crate) fn into_alternatives(self) -> impl Iterator<Item = PlantAlternative> {
        iproduct!(self.invertor, self.arrays.into_iter().multi_cartesian_product())
            .map(|(invertor, arrays)| PlantAlternative { invertor, arrays })
    }
}

