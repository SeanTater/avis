use crate::errors::*;
use crate::feature::{Feature, Pipe, Overflow};
use crate::usmap::USMap;
use anyhow::*;
use bevy::prelude::*;

pub fn main() -> Result<()> {
    let theater = crate::theater::Theater::default();

    let lats = vec![1.0, 2.0, 3.0];
    let lats = Pipe::from(&lats[..]).fit_to(&theater.depth).bundle(lats);
    let lons = vec![0.0, -100.0, -200.0];
    let lons = Pipe::from(&lons[..]).fit_to(&theater.width).bundle(lons);
    let alts = vec![-0.2, 2.0, -20.0];
    let alts = Pipe::from(&alts[..]).fit_to(&theater.height).bundle(alts);
    let sizes = vec![1.0, 0.0, -2.0];
    let sizes = Pipe::from(&sizes[..]).fit_to(&(0.05..=0.5)).bundle(sizes);

    App::new()
        .add_plugin(crate::scatterplot::Scatterplot {
            lats,
            lons,
            alts,
            sizes,
        })
        .add_plugin(crate::usmap::USMapPlugin)
        .add_plugin(theater)
        .add_startup_system(setup_map)
        .run();
    Ok(())
}

fn setup_map(mut commands: Commands) {
    let lat_pipe = Pipe::new(25.0..=71.0, -2.0..=2.0).overflow(Overflow::Saturate);
    let lon_pipe = Pipe::new(-180.0..=65.0, -5.0..=5.0).overflow(Overflow::Saturate);
    println!("lon_pipe: {:?}, lat_pipe: {:?}", lon_pipe, lat_pipe);
    let map = USMap::new(lon_pipe, lat_pipe)
    .expect("Failed to load US map");
    commands.spawn().insert(map);
}
