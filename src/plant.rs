use std::collections::HashMap;
use std::cmp::min;
use std::ops::{AddAssign, Deref};
use std::sync::Arc;

use itertools::{iproduct, Itertools};
use serde::{Deserialize, Serialize};

use crate::alternatives::Alternatives;
use crate::reference::TimeSlots;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Report {
    price: usize,
    power_generated: usize,
    power_per_year: usize,
    lost_to_invertor: usize,
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
    reference: Arc<String>,
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

        let arrays = self
            .arrays
            .iter()
            .map(|arr| (arr.power, &references[arr.reference.deref()]))
            .collect::<Vec<_>>();

        let mut power_generated = 0;
        let mut lost_to_invertor = 0;

        for i in 0..arrays[0].1.len() {
            let generated = arrays.iter().map(|(arr, refe)| {
                (*arr as f32 * refe[i].power) as usize
            }).sum();
            let produced = min(generated, self.invertor.power);
            power_generated += produced;
            lost_to_invertor += generated - produced;
        }

        let power_per_year = power_generated * 24 * 365 / arrays[0].1.len();

        Report {
            price,
            power_generated,
            power_per_year,
            lost_to_invertor,
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
        iproduct!(
            self.invertor,
            self.arrays.into_iter().multi_cartesian_product()
        )
        .map(|(invertor, arrays)| PlantAlternative { invertor, arrays })
    }
}
