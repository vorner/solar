use std::path::PathBuf;

use anyhow::{Context, Error};
use log::debug;
use rayon::prelude::*;
use structopt::StructOpt;

use self::site::Site;

mod alternatives;
mod site;

/// Simulate a system with a solar (or other, wind is planned) power plant.
///
/// It simulates the power production (according to somewhat realistic data downloaded from
/// <https://re.jrc.ec.europa.eu/>) and consumption (by randomly scheduling common tasks & their
/// estimated power hungriness according to user input) and power accumulation (both positive and
/// negative ‒ hot water, heat debts, batteries, laundry baskets...), and figuring out how much
/// power would be:
/// * Produced and used right away
/// * Produced and used later
/// * Wasted in system inefficiencies (battery roundtrip, the hot water cooling down as it
///   waits...)
/// * Exported to the grid
/// * Imported from the grid
#[derive(Clone, Debug, StructOpt)]
struct Opts {
    /// Configuration of each input task.
    ///
    /// Each configuration describes a site with all the alternatives to be tried out.
    ///
    /// See the readme and examples.
    #[structopt(parse(from_os_str))]
    site: Vec<PathBuf>,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let opts = Opts::from_args();
    debug!("Opts: {:?}", opts);
    let reports = opts
        .site
        .par_iter()
        .map(|p| {
            debug!("Processing {}", p.display());
            Site::process(p).with_context(|| format!("Site {}", p.display()))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    for (site, report) in opts.site.iter().zip(reports) {
        println!("==============");
        println!("{}", site.display());
        println!("{}", serde_yaml::to_string(&report).expect("Report not serializable"));
    }
    Ok(())
}
