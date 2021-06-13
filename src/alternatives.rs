use std::iter::repeat_with;
use std::ops::AddAssign;
use std::vec::IntoIter;

use serde::Deserialize;
use itertools::Itertools;

fn one() -> usize { 1 }

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Alternatives<Of, Inc = Of> {
    Any(Vec<Alternatives<Of, Inc>>),
    Parametric {
        #[serde(flatten)]
        start: Of,
        #[serde(default)]
        increment: Inc,
        #[serde(default = "one")]
        cnt: usize,
    },
}

impl<Of, Inc> IntoIterator for Alternatives<Of, Inc>
where
    Of: AddAssign<Inc> + Clone,
    Inc: Clone + Default,
{
    type Item = Of;

    type IntoIter = IntoIter<Of>;

    fn into_iter(self) -> IntoIter<Of> {
        match self {
            Self::Any(sub) => sub
                .into_iter()
                .map(IntoIterator::into_iter)
                .flatten()
                .collect_vec(),
            Self::Parametric {
                mut start,
                increment,
                cnt,
            } => repeat_with(|| {
                let val = start.clone();
                start += increment.clone();
                val
            })
            .take(cnt)
            .collect_vec(),
        }
        .into_iter()
    }
}
