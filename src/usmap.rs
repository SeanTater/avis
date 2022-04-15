use std::ops::RangeInclusive;

use anyhow::*;
use bevy::prelude::*;
use geo::map_coords::TryMapCoords;

use crate::feature::{Pipe, Overflow};
use crate::meshutil::estimate_vertex_normals;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

#[derive(Component)]
pub struct State {
    name: String,
    polygon: geo::Polygon<f32>,
}
pub struct USMapPlugin;

#[derive(Component)]
pub struct USMap {
    lat_pipe: Pipe,
    lon_pipe: Pipe,
    loaded: bool,
}
impl USMap {
    pub fn new(lon_pipe: Pipe, lat_pipe: Pipe) -> Result<Self> {
        Ok(USMap {
            lat_pipe,
            lon_pipe,
            loaded: false,
        })
    }

    fn read_counties(&self) -> Result<Vec<State>> {

        // This JSON is invalid UTF8, unfortunately, so we fix it on the fly, which uses more memory.
        let shapes = std::fs::read("us-maps/geojson/state.geo.json")?;
        let shapes = String::from_utf8_lossy(&shapes)
            .into_owned()
            .parse::<geojson::GeoJson>()?;
        let mut counties = vec![];
        for feature in geojson::FeatureCollection::try_from(shapes)?.features {
            let name = feature
                .property("NAME10")
                .ok_or(anyhow!("State missing name"))?
                .as_str()
                .ok_or(anyhow!("State name is not a string"))?
                .to_string();
            if feature.geometry.is_none() {
                continue;
            }
            let polygons: geo::Geometry<f32> = feature.geometry.unwrap().try_into()?;
            let polygons = match polygons {
                geo::Geometry::MultiPolygon(mp) => mp.0,
                geo::Geometry::Polygon(p) => vec![p],
                _ => vec![],
            };
            for polygon in polygons {
                counties.push(State {
                    name: name.clone(),
                    polygon,
                })
            }
        }
        Ok(counties)
    }
}

impl Plugin for USMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_counties);
    }
}

/// Set up a series of meshes to represent the counties
fn setup_counties(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: Query<&mut USMap>,
) {
    if map.get_single().is_err() { return }
    else if map.single().loaded { return }
    else {
        map.single_mut().loaded = true;
    }
    let map = map.single();
    for county in map.read_counties().expect("Failed to read counties") {
        // We could simplify the polygons to accelerate rendering, but it seems unnecessary.
        //.map(|poly| poly.simplifyvw_preserve(&0.0001))
        let poly = &county.polygon;
        let altitude = rand::random::<f32>() / 50.0;
        let tris = delaunator::triangulate(
            &poly
                .exterior()
                .points()
                .map(|p| delaunator::Point {
                    // These locations not displayed onscreen but for consistency we will scale them
                    x: -map.lon_pipe.apply(p.x()) as f64,
                    y: map.lat_pipe.apply(p.y()) as f64,
                })
                .collect::<Vec<_>>(),
        );

        eprintln!("{}, {} points, {} triangles", county.name, poly.exterior().points().count(), tris.len());

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let positions = poly
            .exterior()
            .points()
            .map(|p| [map.lon_pipe.apply(p.x()), altitude, map.lat_pipe.apply(p.y())])
            .collect::<Vec<_>>();
        let uv0_pipe = Pipe::new(-180.0..=180.0, 0.0..=1.0).overflow(Overflow::Saturate);
        let uv0 = poly
            .exterior()
            .points()
            .map(|p| [uv0_pipe.apply(p.x()), uv0_pipe.apply(p.y())])
            .collect::<Vec<_>>();
        let indices = tris
            .triangles
            .into_iter()
            .map(|u| u as u32)
            .rev()
            .collect::<Vec<_>>();
        let normals = estimate_vertex_normals(
            &positions.iter().map(|v| Vec3::from(*v)).collect::<Vec<_>>(),
            &indices,
        )
        .expect("Couldn't guess normals")
        .into_iter()
        .map(|v| v.to_array())
        .collect::<Vec<_>>();
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv0);
        mesh.set_indices(Some(Indices::U32(indices)));

        let material = materials.add(StandardMaterial {
            base_color: Color::rgb(rand::random(), rand::random(), rand::random()),
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            ..Default::default()
        });
        commands.spawn().insert(county).insert_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material,
            ..Default::default()
        });
    }
}
