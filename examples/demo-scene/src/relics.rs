//! Office things left in the valley, as in the backrooms: an old desk on the
//! river's bank with a computer whose screen is a Cranpose composition, a
//! chair fallen in the grass, and a piece of yellow wall with a flickering
//! light.

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use cranpose_bevy::screen::CranposeScreen;
use demo_ui::{Light, TerminalBridge, TerminalState};

use crate::{Ground, SceneCommands, SceneControls};

// How near the camera must be for the pointer to reach the screen, in metres.
const REACH: f32 = 3.0;
const DESK_HEIGHT: f32 = 0.74;

/// The computer's screen.
#[derive(Component)]
pub(crate) struct OldScreen;

// A light that goes out now and then.
#[derive(Component)]
pub(crate) struct Flicker {
    material: Handle<StandardMaterial>,
    lit: LinearRgba,
}

struct Materials {
    plastic: Handle<StandardMaterial>,
    desk: Handle<StandardMaterial>,
    fabric: Handle<StandardMaterial>,
    wallpaper: Handle<StandardMaterial>,
    carpet: Handle<StandardMaterial>,
    lamp: Handle<StandardMaterial>,
    led: Handle<StandardMaterial>,
}

pub(crate) fn place(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ground: Res<Ground>,
) {
    let looks = Materials::new(&mut materials);
    let start = ground.start();
    // The start stands on a slope eleven metres over the river, looking
    // along +x; the bank comes into its view from about 34 metres on.
    let desk = ground.bank(start.x + 38.0, start.z, 0.35);
    let facing = Quat::from_rotation_y((start.x - desk.x).atan2(start.z - desk.z));
    place_desk(&mut commands, &mut meshes, &looks, &ground, desk, facing);
    let wall = ground.bank(desk.x + 7.0, start.z, 0.6);
    place_wall(&mut commands, &mut meshes, &looks, wall, facing);
}

impl Materials {
    fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        let mut matte = |color: Color, roughness: f32| {
            materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: roughness,
                ..default()
            })
        };
        let plastic = matte(Color::srgb(0.76, 0.72, 0.6), 0.7);
        let desk = matte(Color::srgb(0.6, 0.56, 0.48), 0.8);
        let fabric = matte(Color::srgb(0.18, 0.2, 0.26), 0.95);
        let wallpaper = matte(Color::srgb(0.8, 0.72, 0.42), 0.95);
        let carpet = matte(Color::srgb(0.52, 0.47, 0.3), 1.0);
        // Emissive light is in nits, and the camera is exposed for sunlight.
        let mut glowing = |lit: LinearRgba| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: lit,
                ..default()
            })
        };
        let lamp = glowing(LAMP);
        let led = glowing(LinearRgba::rgb(0.0, 30_000.0, 0.0));
        Self {
            plastic,
            desk,
            fabric,
            wallpaper,
            carpet,
            lamp,
            led,
        }
    }
}

const LAMP: LinearRgba = LinearRgba::rgb(40_000.0, 40_000.0, 34_000.0);

// The desk's legs reach down to the ground under each of them, so it stands
// level on the sloping bank.
fn place_desk(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    looks: &Materials,
    ground: &Ground,
    at: Vec3,
    facing: Quat,
) {
    let corners = [(-0.55, -0.25), (0.55, -0.25), (-0.55, 0.25), (0.55, 0.25)];
    let ground_under = |x: f32, z: f32| {
        let world = at + facing * Vec3::new(x, 0.0, z);
        ground.height(world.x, world.z)
    };
    let top = corners
        .iter()
        .map(|&(x, z)| ground_under(x, z))
        .fold(f32::MIN, f32::max)
        + DESK_HEIGHT;
    let mut desk = commands.spawn((
        Transform::from_xyz(at.x, top, at.z).with_rotation(facing),
        Visibility::default(),
    ));
    desk.with_children(|desk| {
        desk.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.2, 0.04, 0.6))),
            MeshMaterial3d(looks.desk.clone()),
            Transform::from_xyz(0.0, 0.02, 0.0),
        ));
        for (x, z) in corners {
            let length = top - ground_under(x, z) + 0.05;
            desk.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.04, length, 0.04))),
                MeshMaterial3d(looks.desk.clone()),
                Transform::from_xyz(x, -length / 2.0, z),
            ));
        }
        desk.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.44, 0.03, 0.15))),
            MeshMaterial3d(looks.plastic.clone()),
            Transform::from_xyz(0.02, 0.055, 0.19).with_rotation(Quat::from_rotation_y(-0.06)),
        ));
        desk.spawn((
            Transform::from_xyz(0.0, 0.04, -0.06).with_rotation(Quat::from_rotation_y(0.1)),
            Visibility::default(),
        ))
        .with_children(|monitor| place_monitor(monitor, meshes, looks));
    });
    // The computer lies on its side by the desk, and the chair fell over.
    let tower = at + facing * Vec3::new(0.85, 0.0, 0.05);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.19, 0.42, 0.44))),
        MeshMaterial3d(looks.plastic.clone()),
        Transform::from_xyz(tower.x, ground.height(tower.x, tower.z) + 0.08, tower.z)
            .with_rotation(facing * Quat::from_rotation_z(1.52)),
    ));
    let chair = at + facing * Vec3::new(-0.95, 0.0, 0.7);
    commands
        .spawn((
            Transform::from_xyz(chair.x, ground.height(chair.x, chair.z) + 0.24, chair.z)
                .with_rotation(facing * Quat::from_rotation_y(0.7) * Quat::from_rotation_z(1.4)),
            Visibility::default(),
        ))
        .with_children(|chair| {
            chair.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.46, 0.07, 0.46))),
                MeshMaterial3d(looks.fabric.clone()),
                Transform::default(),
            ));
            chair.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.44, 0.52, 0.06))),
                MeshMaterial3d(looks.fabric.clone()),
                Transform::from_xyz(0.0, 0.3, -0.21),
            ));
            chair.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.03, 0.38))),
                MeshMaterial3d(looks.desk.clone()),
                Transform::from_xyz(0.0, -0.22, 0.0),
            ));
        });
}

// A fourteen-inch tube monitor, its front along +z, standing on its foot.
fn place_monitor(monitor: &mut ChildSpawnerCommands, meshes: &mut Assets<Mesh>, looks: &Materials) {
    let parts = [
        (Cuboid::new(0.24, 0.03, 0.22), Vec3::new(0.0, 0.015, 0.0)),
        (Cuboid::new(0.1, 0.05, 0.1), Vec3::new(0.0, 0.05, 0.0)),
        (Cuboid::new(0.4, 0.36, 0.3), Vec3::new(0.0, 0.25, 0.0)),
        (Cuboid::new(0.3, 0.28, 0.16), Vec3::new(0.0, 0.25, -0.22)),
    ];
    for (shape, at) in parts {
        monitor.spawn((
            Mesh3d(meshes.add(shape)),
            MeshMaterial3d(looks.plastic.clone()),
            Transform::from_translation(at),
        ));
    }
    monitor.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.012, 0.012, 0.004))),
        MeshMaterial3d(looks.led.clone()),
        Transform::from_xyz(0.16, 0.1, 0.151),
    ));
    // A VGA screen: 320 by 240 dp at two pixels per dp.
    monitor.spawn((
        Mesh3d(meshes.add(tube_glass(0.31, 0.235, 0.014))),
        Transform::from_xyz(0.0, 0.265, 0.151),
        OldScreen,
        CranposeScreen::new(Vec2::new(320.0, 240.0), 2.0, |world| {
            let bridge = TerminalBridge::default();
            world.insert_non_send(bridge.clone());
            Box::new(move || demo_ui::Terminal(bridge.clone()))
        })
        .with_reach(REACH),
    ));
}

// The screen of a picture tube: `width` by `height`, bulging `bulge` towards
// +z in the middle. Its UVs run from (0, 0) at the top left.
fn tube_glass(width: f32, height: f32, bulge: f32) -> Mesh {
    const COLUMNS: u32 = 16;
    const ROWS: u32 = 12;
    let points = ((COLUMNS + 1) * (ROWS + 1)) as usize;
    let mut positions = Vec::with_capacity(points);
    let mut uvs = Vec::with_capacity(points);
    for row in 0..=ROWS {
        for column in 0..=COLUMNS {
            let (u, v) = (column as f32 / COLUMNS as f32, row as f32 / ROWS as f32);
            let rise = (1.0 - (2.0 * u - 1.0).powi(2)) * (1.0 - (2.0 * v - 1.0).powi(2));
            positions.push([(u - 0.5) * width, (0.5 - v) * height, bulge * rise]);
            uvs.push([u, v]);
        }
    }
    let mut indices = Vec::with_capacity((COLUMNS * ROWS * 6) as usize);
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            let top_left = row * (COLUMNS + 1) + column;
            let bottom_left = top_left + COLUMNS + 1;
            indices.extend([
                top_left,
                bottom_left,
                top_left + 1,
                top_left + 1,
                bottom_left,
                bottom_left + 1,
            ]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; points])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

// A piece of yellow wall with a doorway, standing on a patch of carpet, and
// a tube light above the door.
fn place_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    looks: &Materials,
    at: Vec3,
    facing: Quat,
) {
    commands
        .spawn((
            Transform::from_translation(at - Vec3::Y * 0.15).with_rotation(facing),
            Visibility::default(),
        ))
        .with_children(|wall| {
            let pieces = [
                (Cuboid::new(1.2, 2.8, 0.12), Vec3::new(-1.0, 1.4, 0.0)),
                (Cuboid::new(1.2, 2.8, 0.12), Vec3::new(1.0, 1.4, 0.0)),
                (Cuboid::new(0.8, 0.6, 0.12), Vec3::new(0.0, 2.5, 0.0)),
            ];
            for (shape, at) in pieces {
                wall.spawn((
                    Mesh3d(meshes.add(shape)),
                    MeshMaterial3d(looks.wallpaper.clone()),
                    Transform::from_translation(at),
                ));
            }
            wall.spawn((
                Mesh3d(meshes.add(Cuboid::new(3.8, 0.03, 2.6))),
                MeshMaterial3d(looks.carpet.clone()),
                Transform::from_xyz(0.0, 0.16, 1.2),
            ));
            wall.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.9, 0.05, 0.14))),
                MeshMaterial3d(looks.lamp.clone()),
                Transform::from_xyz(0.0, 2.68, 0.13),
                Flicker {
                    material: looks.lamp.clone(),
                    lit: LAMP,
                },
            ));
        });
}

// The tube light goes out for a moment every few seconds.
pub(crate) fn flicker(
    time: Res<Time>,
    lights: Query<&Flicker>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();
    let out = (t % 4.3 > 4.1) || (t % 7.9 > 7.83);
    for light in &lights {
        let wanted = if out { LinearRgba::BLACK } else { light.lit };
        if materials
            .get(&light.material)
            .is_some_and(|material| material.emissive != wanted)
            && let Some(mut material) = materials.get_mut(&light.material)
        {
            material.emissive = wanted;
        }
    }
}

// The terminal's commands go to the owner of the scene's controls, which the
// terminal shows, with whether the walker stands near enough to use it and
// the blinking of its cursor.
pub(crate) fn sync_terminal(
    terminal: Option<NonSend<TerminalBridge>>,
    controls: Res<SceneControls>,
    mut commands: ResMut<SceneCommands>,
    time: Res<Time>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    screens: Query<&GlobalTransform, With<OldScreen>>,
) {
    let Some(terminal) = terminal else {
        return;
    };
    let mut sent = terminal.commands.borrow_mut();
    if !sent.is_empty() {
        commands.0.append(&mut sent);
    }
    let near = cameras
        .iter()
        .zip(screens.iter())
        .any(|(camera, screen)| camera.translation().distance(screen.translation()) <= REACH);
    let state = TerminalState {
        light: Light::of(&controls.0),
        near,
        cursor: time.elapsed_secs().fract() < 0.5,
    };
    if let Some(shown) = terminal.state.get()
        && shown.value() != state
    {
        shown.set(state);
    }
}
