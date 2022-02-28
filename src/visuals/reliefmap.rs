use crate::errors::*;
use crate::meshutil::estimate_vertex_normals;
use bevy::prelude::*;
use std::f32::consts::PI;

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
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(1.0, 2.0, 4.0)
            .looking_at(Vec3::from((10.0, 0.0, -10.)), Vec3::Y),
        ..Default::default()
    }).insert(bevy_fly_camera::FlyCamera::default());
}

fn orbit_camera(mut transforms: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    for mut transform in transforms.iter_mut() {
        let sec = 0.1 * time.seconds_since_startup() as f32;
        *transform = Transform::from_xyz(20. * sec.sin(), 10., 35. * sec.cos())
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
}
