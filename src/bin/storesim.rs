use avis::errors::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use r2d2_sqlite::SqliteConnectionManager;
use std::f32::consts::PI;

fn main() -> Result<()> {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(BuildingLoader::new()?)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(avis::people::People)
        .add_startup_system(setup)
        .add_system(orbit_camera)
        .run();
    Ok(())
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
            .map(|(x, y, z, rx, ry, rz, angle, path)| {
                (
                    Transform::from_xyz(x, y, z).with_rotation(Quat::from_axis_angle(
                        Vec3::from([rx, ry, rz]).normalize(),
                        angle,
                    )),
                    assets.load(&path),
                )
            })
            .collect();
        Ok(res)
    }
}

/// sets up a scene with textured entities
fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut buildings: ResMut<BuildingLoader>,
) {
    // floor
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(bevy::prelude::shape::Plane { size: 100. }.into()),
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
    });
}

fn orbit_camera(mut transforms: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    for mut transform in transforms.iter_mut() {
        let sec = 0.1 * time.seconds_since_startup() as f32;
        *transform = Transform::from_xyz(20. * sec.sin(), 10., 35. * sec.cos())
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
}
