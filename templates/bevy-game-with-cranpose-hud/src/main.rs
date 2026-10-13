//! A Bevy game with a Cranpose HUD over its camera. This file is the game;
//! `hud.rs` is the HUD, a composition Bevy's main thread draws.

mod hud;

use std::f32::consts::TAU;

use bevy::prelude::*;
use cranpose_bevy::hud::{CranposeHudPlugin, HudCamera};

#[derive(Component)]
struct Cube;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy game with a Cranpose HUD".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CranposeHudPlugin::new(hud::content))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.09)))
        .add_systems(Startup, spawn)
        .add_systems(Update, turn)
        .run()
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        HudCamera,
        Transform::from_xyz(-2.5, 2.2, 4.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0, 6.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(60.0, 60.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.32, 0.36, 0.42))),
    ));
    commands.spawn((
        Cube,
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.95, 0.45, 0.25))),
        Transform::from_xyz(0.0, 0.8, 0.0),
    ));
}

/// Turns the cube at the speed the HUD sets, and tells the HUD how many
/// turns it made.
fn turn(
    time: Res<Time>,
    hud: Option<NonSend<hud::Hud>>,
    mut cubes: Query<&mut Transform, With<Cube>>,
    mut angle: Local<f32>,
) {
    let Some(state) = hud.and_then(|hud| hud.state()) else {
        return;
    };
    let step = state.speed.value() * TAU * time.delta_secs();
    for mut transform in &mut cubes {
        transform.rotate_y(step);
    }
    *angle += step;
    let turns = (*angle / TAU) as u32;
    if state.turns.value() != turns {
        state.turns.set(turns);
    }
}
