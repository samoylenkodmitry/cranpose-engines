//! Bevy's side: a lit cube that turns over a floor.

use std::f32::consts::TAU;

use bevy::{camera::RenderTarget, prelude::*};

/// How fast the cube turns. The interface sets it with `BevyView::send`.
#[derive(Resource)]
pub struct Spin {
    pub turns_per_second: f32,
}

#[derive(Component)]
struct Cube;

/// Adds the scene to Bevy's app. Its camera draws into `target`, the texture
/// Cranpose shows.
pub fn build(app: &mut App, target: RenderTarget) {
    app.insert_resource(Spin {
        turns_per_second: 0.25,
    })
    .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.09)))
    .add_systems(
        Startup,
        move |commands: Commands,
              meshes: ResMut<Assets<Mesh>>,
              materials: ResMut<Assets<StandardMaterial>>| {
            spawn(commands, meshes, materials, target.clone());
        },
    )
    .add_systems(Update, turn);
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    target: RenderTarget,
) {
    commands.spawn((
        Camera3d::default(),
        target,
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

fn turn(time: Res<Time>, spin: Res<Spin>, mut cubes: Query<&mut Transform, With<Cube>>) {
    for mut transform in &mut cubes {
        transform.rotate_y(spin.turns_per_second * TAU * time.delta_secs());
    }
}
