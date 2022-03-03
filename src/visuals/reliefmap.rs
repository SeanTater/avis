use crate::errors::*;
use crate::meshutil::estimate_vertex_normals;
use anyhow::*;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use geo::prelude::*;
use geo::simplifyvw::SimplifyVWPreserve;
use geo::MultiPolygon;
use geo::Triangle;
use std::f32::consts::PI;
use std::str::FromStr;

#[derive(Component)]
struct County {
    name: String,
    state: String,
    polygon: geo::Polygon<f32>,
}

pub fn main() -> Result<()> {
    App::new()
        .insert_resource(bevy_atmosphere::AtmosphereMat::default())
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: false })
        .add_plugin(bevy_fly_camera::FlyCameraPlugin)
        .add_startup_system(setup_counties)
        .run();
    Ok(())
}

fn read_counties() -> Result<Vec<County>> {
    let states = [
        "",
        "Alabama",
        "Alaska",
        "Arizona",
        "Arkansas",
        "California",
        "Colorado",
        "Connecticut",
        "Delaware",
        "Florida",
        "Georgia",
        "Hawaii",
        "Idaho",
        "Illinois",
        "Indiana",
        "Iowa",
        "Kansas",
        "Kentucky",
        "Louisiana",
        "Maine",
        "Maryland",
        "Massachusetts",
        "Michigan",
        "Minnesota",
        "Mississippi",
        "Missouri",
        "Montana",
        "Nebraska",
        "Nevada",
        "New Hampshire",
        "New Jersey",
        "New Mexico",
        "New York",
        "North Carolina",
        "North Dakota",
        "Ohio",
        "Oklahoma",
        "Oregon",
        "Pennsylvania",
        "Rhode Island",
        "South Carolina",
        "South Dakota",
        "Tennessee",
        "Texas",
        "Utah",
        "Vermont",
        "Virginia",
        "Washington",
        "West Virginia",
        "Wisconsin",
        "Wyoming",
    ];

    // This JSON is invalid UTF8, unfortunately, so we fix it on the fly, which uses more memory.
    let shapes = std::fs::read("us-maps/geojson/county.geo.json")?;
    let shapes = String::from_utf8_lossy(&shapes)
        .into_owned()
        .parse::<geojson::GeoJson>()?;
    let mut counties = vec![];
    for feature in geojson::FeatureCollection::try_from(shapes)?.features {
        let name = feature
            .property("NAMELSAD10")
            .ok_or(anyhow!("County missing name"))?
            .as_str()
            .ok_or(anyhow!("County name is not a string"))?
            .to_string();
        let state_number = feature
            .property("STATEFP10")
            .ok_or(anyhow!("County missing state number"))?
            .as_str()
            .ok_or(anyhow!("County state number is not an integer in a string"))?
            .parse::<usize>()?;
        let state = states
            .get(state_number)
            .unwrap_or(&"Other Territory")
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
            counties.push(County {
                name: name.clone(),
                state: state.clone(),
                polygon,
            })
        }
    }
    Ok(counties)
}

/// Set up a series of meshes to represent the counties
fn setup_counties(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for county in read_counties().expect("Failed to read counties") {
        // We could simplify the polygons to accelerate rendering, but it seems unnecessary.
        //.map(|poly| poly.simplifyvw_preserve(&0.0001))
        let ref poly = county.polygon;
        eprint!(".");
        let altitude = rand::random::<f32>() / 50.0;
        let tris = delaunator::triangulate(
            &poly
                .exterior()
                .points()
                .map(|p| delaunator::Point {
                    x: -p.x() as f64,
                    y: p.y() as f64,
                })
                .collect::<Vec<_>>(),
        );

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let positions = poly
            .exterior()
            .points()
            .map(|p| [-p.x() / 10., altitude, p.y() / 10.])
            .collect::<Vec<_>>();
        let uv0 = poly
            .exterior()
            .points()
            .map(|p| [0.5 + (p.x() / 360.0), 0.5 + (p.y() / 360.0)])
            .collect::<Vec<_>>();
        let indices = tris
            .triangles
            .into_iter()
            .map(|u| u as u32)
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

    // Floor, if there is any meaning to it.
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(bevy::prelude::shape::Plane { size: 100. }.into()),
        material: materials.add(StandardMaterial {
            base_color: Color::ANTIQUE_WHITE,
            ..Default::default()
        }),
        ..Default::default()
    });

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 8.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        brightness: 0.25,
        ..Default::default()
    });

    // Flyable camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(1.0, 2.0, 4.0)
                .looking_at(Vec3::from((10.0, 0.0, -10.)), Vec3::Y),
            ..Default::default()
        })
        .insert(bevy_fly_camera::FlyCamera::default());
}
