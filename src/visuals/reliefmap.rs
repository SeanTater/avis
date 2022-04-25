use crate::errors::*;
use crate::feature::{Feature, Overflow, Pipe};
use crate::usmap::USMap;
use anyhow::*;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ScatterPoint {
    lat: f32,
    lon: f32,
    alt: f32,
    size: f32,
}

pub fn main() -> Result<()> {
    let theater = crate::theater::Theater::default();
    let lat_pipe = Pipe::new(71.0..=25.0, -2.0..=2.0);
    let lon_pipe = Pipe::new(-180.0..=65.0, -8.0..=8.0);

    let points: Vec<ScatterPoint> = serde_json::from_slice(&std::fs::read("points.json")?)?;

    let lats = points.iter().map(|p| p.lat).collect::<Vec<_>>();
    let lats = lat_pipe.clone().bundle(lats);

    let lons = points.iter().map(|p| p.lon).collect::<Vec<_>>();
    let lons = lon_pipe.clone().bundle(lons);

    let alts = points.iter().map(|p| p.alt).collect::<Vec<_>>();
    let alts = Pipe::from(&alts[..]).fit_to(&(0.1..=1.0)).bundle(alts);

    let sizes = points.iter().map(|p| p.size).collect::<Vec<_>>();
    let sizes = Pipe::from(&sizes[..]).fit_to(&(0.01..=0.05)).bundle(sizes);

    let colors = points
        .iter()
        .map(|p| Color::rgb(p.size, p.size, p.size))
        .collect::<Vec<_>>();
    App::new()
        .add_plugin(crate::scatterplot::Scatterplot {
            lats,
            lons,
            alts,
            sizes,
            colors
        })
        .add_plugin(crate::usmap::USMapPlugin)
        .add_plugin(theater)
        .add_startup_system(setup_map)
        .run();
    Ok(())
}

fn setup_map(mut commands: Commands) {
    let lat_pipe = Pipe::new(71.0..=25.0, -2.0..=2.0).overflow(Overflow::Saturate);
    let lon_pipe = Pipe::new(-180.0..=65.0, -8.0..=8.0).overflow(Overflow::Saturate);
    println!("lon_pipe: {:?}, lat_pipe: {:?}", lon_pipe, lat_pipe);
    let map = USMap::new(lon_pipe, lat_pipe).expect("Failed to load US map");
    commands.spawn().insert(map);
}
