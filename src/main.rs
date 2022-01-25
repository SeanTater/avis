pub mod errors;
use errors::*;
mod meshutil;
use bevy::prelude::*;
use r2d2_sqlite::SqliteConnectionManager;
use std::f32::consts::PI;

use crate::meshutil::estimate_vertex_normals;

/// This example shows various ways to configure texture materials in 3D
fn main() -> Result<()> {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(BuildingLoader::new()?)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(spin_cube)
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

pub struct BuildingLoader {
    pool: r2d2::Pool<SqliteConnectionManager>,
}
impl BuildingLoader {
    pub fn new() -> Result<Self> {
        let manager = SqliteConnectionManager::file("avis.db");
        let pool = r2d2::Pool::new(manager)?;
        Ok(Self { pool })
    }

    pub fn load_building(
        &mut self,
        assets: &AssetServer,
        id: i64,
    ) -> Result<Vec<(Transform, Handle<Scene>)>> {
        let res = self
            .pool
            .get()?
            .prepare("SELECT * FROM furniture_with_context WHERE building_id = ?")?
            .query_map(&[&id], |row| {
                Ok((
                    row.get::<_, f32>("x")?,
                    row.get::<_, f32>("y")?,
                    row.get::<_, f32>("z")?,
                    row.get::<_, f32>("rx")?,
                    row.get::<_, f32>("ry")?,
                    row.get::<_, f32>("rz")?,
                    row.get::<_, f32>("angle")?,
                    row.get::<_, String>("scene_path")?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .map(|(x, y, z, rx, ry, rz, angle, path)| (Transform::from_xyz(x, y, z).with_rotation(Quat::from_axis_angle(Vec3::from([rx, ry, rz]).normalize(), angle)), assets.load(&path)))
            .collect();
        Ok(res)
    }
}

#[derive(Component)]
struct Cube;

#[derive(Component)]
struct Associate;

/// sets up a scene with textured entities
fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut buildings: ResMut<BuildingLoader>,
) {
    let associate = assets.load("models/person/associate.glb");
    for _ in 0..10 {
        let transform = Transform
            ::from_xyz(5. - 10.*rand::random::<f32>(), 0., 5. - 10.*rand::random::<f32>());
            //.with_rotation(Quat::from_axis_angle(Vec3::Y, PI * rand::random::<f32>()));
        commands
            .spawn_bundle((Associate, transform, GlobalTransform::identity()))
            .with_children(|parent| {
                parent.spawn_scene(associate.clone());
            });
    }

    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::DARK_GREEN,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..Default::default()
    });

    let cube_rotation = Quat::from_axis_angle(Vec3::Y, PI / 4.);
    let mesh = create_map();

    // cube
    commands.spawn().insert(Cube).insert_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        transform: Transform::from_rotation(cube_rotation),
        material: material_handle.clone(),
        ..Default::default()
    });

    // floor
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 100. }.into()),
        material: materials.add(StandardMaterial {
            base_color: Color::ANTIQUE_WHITE,
            ..Default::default()
        }),
        ..Default::default()
    });

    // Load the furniture, which is controlled from an SQLite database
    for (transform, scene) in buildings
        .load_building(&assets, 0)
        .expect("Could not load building")
    {
        // to be able to position our 3d model:
        // spawn a parent entity with a Transform and GlobalTransform
        // and spawn our gltf as a scene under it
        commands
            .spawn_bundle((transform, GlobalTransform::identity()))
            .with_children(|parent| {
                parent.spawn_scene(scene);
            });
    }

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 8.0, 0.0),
            rotation: Quat::from_rotation_x(-PI/4.),
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
    });
}

fn spin_cube(mut transforms: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    for mut transform in transforms.iter_mut() {
        let sec = 0.1 * time.seconds_since_startup() as f32;
        *transform = Transform::from_xyz(10. * sec.sin(), 5., 15. * sec.cos())
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
}
