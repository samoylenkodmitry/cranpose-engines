//! The office things left on the river's bank (`nature::office`), in
//! Bevy: their screen is a Cranpose composition, and their tube light
//! flickers.

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use cranpose_bevy::screen::CranposeScreen;
use demo_ui::{Light, TerminalBridge, TerminalState};
use nature::office::{Look, Office, REACH, SCREEN_DENSITY, SCREEN_DP, Shape, glass_mesh, lamp_lit};

use crate::{Ground, SceneCommands, SceneControls};

// The tube light's emission in nits: the camera is exposed for sunlight.
const LAMP_NITS: f32 = 40_000.0;

/// The computer's screen.
#[derive(Component)]
pub(crate) struct OldScreen;

// A light that goes out now and then.
#[derive(Component)]
pub(crate) struct Flicker {
    material: Handle<StandardMaterial>,
    lit: LinearRgba,
}

pub(crate) fn place(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ground: Res<Ground>,
) {
    let office = Office::place(&ground.0);
    let looks = [
        Look::Plastic,
        Look::Desk,
        Look::Fabric,
        Look::Wallpaper,
        Look::Carpet,
        Look::Lamp,
        Look::Led,
    ]
    .map(|look| (look, materials.add(material(look))));
    let glass = meshes.add(glass(glass_mesh()));
    for part in &office.parts {
        let mesh = match part.shape {
            Shape::Cuboid(size) => meshes.add(Cuboid::from_size(Vec3::from(size))),
            Shape::Cylinder { radius, height } => meshes.add(Cylinder::new(radius, height)),
            Shape::Glass => glass.clone(),
        };
        let mut entity = commands.spawn((
            Mesh3d(mesh),
            Transform::from_translation(Vec3::from(part.translation))
                .with_rotation(Quat::from_array(part.rotation)),
        ));
        if part.look == Look::Screen {
            entity.insert((
                OldScreen,
                CranposeScreen::new(Vec2::from(SCREEN_DP), SCREEN_DENSITY, |world| {
                    let bridge = TerminalBridge::default();
                    world.insert_non_send(bridge.clone());
                    Box::new(move || demo_ui::Terminal(bridge.clone()))
                })
                .with_reach(REACH),
            ));
        } else if let Some((_, handle)) = looks.iter().find(|(look, _)| *look == part.look) {
            entity.insert(MeshMaterial3d(handle.clone()));
            if part.look == Look::Lamp {
                entity.insert(Flicker {
                    material: handle.clone(),
                    lit: glow(part.look),
                });
            }
        }
    }
}

fn material(look: Look) -> StandardMaterial {
    let [r, g, b] = look.color();
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        perceptual_roughness: look.roughness(),
        emissive: glow(look),
        ..default()
    }
}

fn glow(look: Look) -> LinearRgba {
    look.glow().map_or(LinearRgba::BLACK, |[r, g, b]| {
        LinearRgba::rgb(r * LAMP_NITS, g * LAMP_NITS, b * LAMP_NITS)
    })
}

fn glass(mesh: nature::office::GlassMesh) -> Mesh {
    let normals = vec![[0.0, 0.0, 1.0]; mesh.positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh.uvs)
    .with_inserted_indices(Indices::U32(mesh.indices))
}

// The tube light goes out for a moment every few seconds.
pub(crate) fn flicker(
    time: Res<Time>,
    lights: Query<&Flicker>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lit = lamp_lit(time.elapsed_secs());
    for light in &lights {
        let wanted = if lit { light.lit } else { LinearRgba::BLACK };
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
