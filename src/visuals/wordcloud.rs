use crate::errors::*;

use crate::visuals::VisualAction;
use bevy::{app::Events, prelude::*, render::camera::Camera, ecs::world::WorldBorrowMut};
use bevy_text_mesh::prelude::*;
use rand::prelude::*;

use super::{Visual, OneshotReceiver, VisualMessage};

// tessellation quality
const MESH_QUALITY: Quality = Quality::Low;

#[derive(Debug, Clone)]
pub struct WordCloudVisual {
    sender: flume::Sender<VisualMessage>,
    receiver: flume::Receiver<VisualMessage>,
}
impl WordCloudVisual {
    pub fn new() -> Self {
        let (sender, receiver) = flume::unbounded();
        Self { sender, receiver }
    }
}

impl Visual for WordCloudVisual {
    fn start(&self, control: super::RemoteControl) -> Result<()> {
        App::new()
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(self.receiver.clone())
            .add_plugins(DefaultPlugins)
            .add_plugin(TextMeshPlugin)
            .add_startup_system(setup)
            .add_startup_system(setup_text_mesh)
            .add_system(rotate_camera)
            .add_system(update_legend)
            .add_system(handle_action)
            .add_stage("Remote Control", control)
            .run();
        Ok(())
    }
    fn react(&self, action: VisualAction) -> OneshotReceiver {
        let (reply, reply_recv) = tokio::sync::oneshot::channel();
        self.sender.send(VisualMessage { reply, action })
            .unwrap_or_else(|_| { tracing::error!("Ignoring message sent to dead visual")});
        reply_recv
    }
}

struct Cloud {
    font: Handle<TextMeshFont>,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Word;
impl Word {
    /// Add a word in any random direction
    fn add(
        commands: &mut Commands,
        font: &Handle<TextMeshFont>,
        material: &Handle<StandardMaterial>,
        text: &str,
    ) {
        let mut rng = rand::thread_rng();
        let transform = Transform {
            translation: Vec3::new(
                rng.gen_range(-1.0..1.0) * 2.0,
                rng.gen::<f32>() * 2.0,
                rng.gen_range(-1.0..1.0) * 2.0,
            ),
            scale: Vec3::ONE * (1. - rng.gen::<f32>() * 0.8) * 0.5,
            ..Default::default()
        }
        .looking_at(
            Vec3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()),
            Vec3::Y,
        );

        commands
            .spawn_bundle(TextMeshBundle {
                text_mesh: TextMesh {
                    text: text.into(),
                    style: TextMeshStyle {
                        font: font.clone(),
                        mesh_quality: MESH_QUALITY,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                transform,
                ..Default::default()
            })
            .insert(Word)
            .insert(material.clone());
    }
}

#[derive(Component)]
struct Legend;

fn setup_text_mesh(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let state = Cloud {
        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
        material: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..Default::default()
        }),
    };

    commands
        .spawn_bundle(TextMeshBundle {
            text_mesh: TextMesh {
                text: String::from("Word Cloud Title"),
                style: TextMeshStyle {
                    font: state.font.clone(),
                    font_size: SizeUnit::NonStandard(18.),
                    color: Color::rgb(
                        0x09 as f32 / 256.0,
                        0x69 as f32 / 256.0,
                        0xda as f32 / 256.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 3., 0.),
            ..Default::default()
        })
        .insert(Legend);

    commands.insert_resource(state);
}

/// Apply any incoming actions
fn handle_action(
    mut commands: Commands,
    state: Res<Cloud>,
    receiver: Res<flume::Receiver<VisualMessage>>,
) {
    for message in receiver.try_iter() {
        let VisualMessage{ reply, action} = message;
        match action {
            VisualAction::AddWord(text) => {
                Word::add(&mut commands, &state.font, &state.material, &text)
            }
        }
        // TODO: For exit, can we confirm this is done?
        reply.send(Ok(())).unwrap_or_else(|_| tracing::info!("Reply sent to dead mailbox"));
    }
}

fn rotate_camera(mut camera: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    for mut camera in camera.iter_mut() {
        let angle = time.seconds_since_startup() as f32 / 2. + 1.55 * std::f32::consts::PI;

        let distance = 6.5;

        camera.translation = Vec3::new(
            angle.sin() as f32 * distance,
            camera.translation.y,
            angle.cos() as f32 * distance,
        );

        *camera = camera.looking_at(Vec3::new(0.0, 1.5, 0.), Vec3::Y);
    }
}

/// Keep the legend pointing at the camera all the time
fn update_legend(
    mut transform_pair: QuerySet<(
        QueryState<&Transform, With<Camera>>,
        QueryState<&mut Transform, With<Legend>>,
    )>,
) {
    let camera_translation = transform_pair.q0().single().translation;
    for mut legend_transform in transform_pair.q1().iter_mut() {
        // eh - why negative?
        *legend_transform = legend_transform.looking_at(-camera_translation, Vec3::Y);
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
