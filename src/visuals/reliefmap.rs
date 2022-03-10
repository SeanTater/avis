use crate::errors::*;
use crate::feature::Feature;
use anyhow::*;
use bevy::prelude::*;

pub fn main() -> Result<()> {
    let theater = crate::theater::Theater::default();

    let lats = Feature::from(vec![1.0, 2.0, 3.0]).fit_to(&theater.depth);
    let lons = Feature::from(vec![0.0, -100.0, -200.0]).fit_to(&theater.width);
    let alts = Feature::from(vec![-0.2, 2.0, -20.0]).fit_to(&theater.height);
    let sizes = Feature::from(vec![1.0, 0.0, -2.0]).fit_to(&(0.05..=0.5));

    App::new()
        .add_plugin(crate::scatterplot::Scatterplot{ lats, lons, alts, sizes })
        .add_plugin(crate::usmap::USMap { width: -5.0..=5.0, length: -5.0..=5.0 })
        .add_plugin(theater)
        .run();
    Ok(())
}
