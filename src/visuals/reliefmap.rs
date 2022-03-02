use crate::errors::*;
use crate::meshutil::estimate_vertex_normals;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use geo::prelude::*;
use geo::simplifyvw::SimplifyVWPreserve;
use geo::MultiPolygon;
use geo::Triangle;
use geojson::Geometry;
use std::f32::consts::PI;
use std::io::Read;

#[derive(Component)]
struct Map;

pub fn main() -> Result<()> {
    App::new()
        .insert_resource(bevy_atmosphere::AtmosphereMat::default())
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: false })
        .add_plugin(bevy_fly_camera::FlyCameraPlugin)
        .add_startup_system(setup)
        .run();
    Ok(())
}

fn create_map() -> Mesh {
    use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let (min_x, max_x, min_y, max_y, min_z, max_z) = (-0.5, 0.5, -0.1, 0.1, -0.5, 0.5);
    let vertices = &[
        // Top
        ([min_x, min_y, max_z], [0., 0., 1.0], [0., 0.]),
        ([max_x, min_y, max_z], [0., 0., 1.0], [1.0, 0.]),
        ([max_x, max_y, max_z], [0., 0., 1.0], [1.0, 1.0]),
        ([min_x, max_y, max_z], [0., 0., 1.0], [0., 1.0]),
        // Bottom
        ([min_x, max_y, min_z], [0., 0., -1.0], [1.0, 0.]),
        ([max_x, max_y, min_z], [0., 0., -1.0], [0., 0.]),
        ([max_x, min_y, min_z], [0., 0., -1.0], [0., 1.0]),
        ([min_x, min_y, min_z], [0., 0., -1.0], [1.0, 1.0]),
        // Right
        ([max_x, min_y, min_z], [1.0, 0., 0.], [0., 0.]),
        ([max_x, max_y, min_z], [1.0, 0., 0.], [1.0, 0.]),
        ([max_x, max_y, max_z], [1.0, 0., 0.], [1.0, 1.0]),
        ([max_x, min_y, max_z], [1.0, 0., 0.], [0., 1.0]),
        // Left
        ([min_x, min_y, max_z], [-1.0, 0., 0.], [1.0, 0.]),
        ([min_x, max_y, max_z], [-1.0, 0., 0.], [0., 0.]),
        ([min_x, max_y, min_z], [-1.0, 0., 0.], [0., 1.0]),
        ([min_x, min_y, min_z], [-1.0, 0., 0.], [1.0, 1.0]),
        // Front
        ([max_x, max_y, min_z], [0., 1.0, 0.], [1.0, 0.]),
        ([min_x, max_y, min_z], [0., 1.0, 0.], [0., 0.]),
        ([min_x, max_y, max_z], [0., 1.0, 0.], [0., 1.0]),
        ([max_x, max_y, max_z], [0., 1.0, 0.], [1.0, 1.0]),
        // Back
        ([max_x, min_y, max_z], [0., -1.0, 0.], [0., 0.]),
        ([min_x, min_y, max_z], [0., -1.0, 0.], [1.0, 0.]),
        ([min_x, min_y, min_z], [0., -1.0, 0.], [1.0, 1.0]),
        ([max_x, min_y, min_z], [0., -1.0, 0.], [0., 1.0]),
    ];
    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices
            .iter()
            .map(|&(loc, _norm, _uv)| loc)
            .collect::<Vec<_>>(),
    );
    let positions: Vec<_> = vertices
        .iter()
        .map(|&(loc, _norm, _uv)| Vec3::from(loc))
        .collect();
    let indices = vec![
        0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16, 17,
        18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
    ];
    let normals: Vec<_> = estimate_vertex_normals(&positions, &indices)
        .unwrap()
        .into_iter()
        .map(|v| v.to_array())
        .collect();
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vertices
            .iter()
            .map(|&(_loc, _norm, uv)| uv)
            .collect::<Vec<_>>(),
    );
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}

/// sets up a scene with textured entities
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::DARK_GREEN,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..Default::default()
    });
    let map_rotation = Quat::from_axis_angle(Vec3::Y, PI / 4.);

    // This JSON is invalid UTF8, unfortunately, so we fix it on the fly, which uses more memory.
    let shapes = std::fs::read("us-maps/geojson/county.geo.json")
        .expect("Couldn't find the county geojson file. Please makes sure you initialized the git submodule");
    let shapes = String::from_utf8_lossy(&shapes)
        .parse::<geojson::GeoJson>()
        .expect("Couldn't read the county-level US map geo-json");
    geojson::quick_collection(&shapes)
        .expect("Failed to interpret map json as a collection")
        .into_iter()
        .flat_map(|geom: geo::Geometry<f32> | match geom {
            geo::Geometry::MultiPolygon(mp) => mp.0,
            geo::Geometry::Polygon(p) => vec![p],
            _ => vec![]
        })//geom.try_into().ok())
        //.flat_map(|mpoly: geo::Polygon<f32>| mpoly.into_iter())
        //.map(|poly| poly.simplifyvw_preserve(&0.0001))
        .map(|poly| {
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
                .map(|p| [0.5+(p.x() / 360.0), 0.5+(p.y() / 360.0)])
                .collect::<Vec<_>>();
            let indices = tris
                .triangles
                .into_iter()
                .map(|u| u as u32)
                .collect::<Vec<_>>();
            // let normals = (0..positions.len() / 3).flat_map(|_| [0.,1.,0.]).collect::<Vec<_>>();
            let normals = estimate_vertex_normals(
                &positions.iter().map(|v| Vec3::from(*v)).collect::<Vec<_>>(),
                &indices,
            )
            .expect("Couldn't guess normals")
            .into_iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>();
            //mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 0.0, 1.0]; positions.len()]);
            mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv0);
            mesh.set_indices(Some(Indices::U32(indices)));
            mesh
        })
        .for_each(|mesh| {
            let material = materials.add(StandardMaterial {
                base_color: Color::rgb(rand::random(), rand::random(), rand::random()),
                alpha_mode: AlphaMode::Blend,
                unlit: false,
                ..Default::default()
            });
            commands.spawn().insert(Map).insert_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                transform: Transform::from_rotation(map_rotation),
                material,
                ..Default::default()
            });
        });

    let mesh = create_map();

    // Map
    commands.spawn().insert(Map).insert_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        transform: Transform::from_rotation(map_rotation),
        material: material_handle.clone(),
        ..Default::default()
    });

    // floor
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

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(1.0, 2.0, 4.0)
                .looking_at(Vec3::from((10.0, 0.0, -10.)), Vec3::Y),
            ..Default::default()
        })
        .insert(bevy_fly_camera::FlyCamera::default());
}
