//! A Cranpose HUD over Bevy's valley, for the desktop and the web.

use bevy::{
    camera::RenderTarget,
    input::{mouse::AccumulatedMouseMotion, touch::Touches},
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};
use cranpose_bevy::hud::{CranposeHudPlugin, HudCamera, HudInput};
use demo_scene::{Quality, SceneControls, SceneStats};
use demo_ui::{Controls, HudBridge, HudCommand, Stats, Walk};

/// The game: Bevy's valley with a Cranpose HUD over its camera. On the web
/// it draws into the page's `app` canvas. `touch` says whether the screen is
/// a phone's touch screen: the HUD then shows its stick before the first
/// touch, and the scene renders the phone's quality.
pub fn app(touch: bool) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Cranpose HUD in Bevy".into(),
            resolution: WindowResolution::new(1320, 820),
            canvas: Some("#app".into()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(CranposeHudPlugin::new(move |world: &mut World| {
        let bridge = HudBridge::new("Bevy", touch);
        world.insert_non_send(bridge.clone());
        Box::new(move || demo_ui::Hud(bridge.clone()))
    }))
    .add_systems(PostStartup, draw_hud_over_the_camera)
    .add_systems(Update, (look_and_walk, run_commands, sync_hud).chain());
    let quality = if touch { Quality::Phone } else { Quality::Full };
    demo_scene::build(&mut app, RenderTarget::default(), quality);
    app
}

fn draw_hud_over_the_camera(mut commands: Commands, cameras: Query<Entity, With<Camera3d>>) {
    for camera in &cameras {
        commands.entity(camera).insert(HudCamera);
    }
}

// Keys walk; the mouse and every finger the HUD did not take look around.
// The HUD's stick writes its part of the walk itself.
fn look_and_walk(
    bridge: Option<NonSend<HudBridge>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    touches: Res<Touches>,
    hud: Res<HudInput>,
) {
    let Some(states) = bridge
        .as_ref()
        .and_then(|bridge| bridge.states.get().copied())
    else {
        return;
    };
    if touches.any_just_pressed() && !states.touch.value() {
        states.touch.set(true);
    }
    let held = |codes: [KeyCode; 2]| !hud.keyboard && keys.any_pressed(codes);
    let current = states.controls.value();
    let walk = Walk {
        forward: held([KeyCode::KeyW, KeyCode::ArrowUp]),
        back: held([KeyCode::KeyS, KeyCode::ArrowDown]),
        left: held([KeyCode::KeyA, KeyCode::ArrowLeft]),
        right: held([KeyCode::KeyD, KeyCode::ArrowRight]),
        run: held([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        stick: current.walk.stick,
    };
    let mut look = if buttons.pressed(MouseButton::Left) && !hud.pointer {
        motion.delta
    } else {
        Vec2::ZERO
    };
    for touch in touches.iter() {
        if hud.finger != Some(touch.id()) {
            look += touch.delta();
        }
    }
    if current.walk != walk || look != Vec2::ZERO {
        states.controls.update(|controls| {
            controls.walk = walk;
            controls.look(look.x, look.y);
        });
    }
}

fn sync_hud(
    bridge: Option<NonSend<HudBridge>>,
    mut controls: ResMut<SceneControls>,
    scene: Res<SceneStats>,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut since_report: Local<f32>,
    mut frames: Local<u32>,
) {
    let Some(states) = bridge
        .as_ref()
        .and_then(|bridge| bridge.states.get().copied())
    else {
        return;
    };
    let wanted = states.controls.value();
    if controls.0 != wanted {
        controls.0 = wanted;
    }
    *since_report += time.delta_secs();
    *frames += 1;
    if *since_report < 0.25 {
        return;
    }
    let size = windows
        .single()
        .map(|window| (window.physical_width(), window.physical_height()))
        .unwrap_or_default();
    states.stats.set(Stats {
        fps: (*frames as f32 / *since_report).round() as u32,
        altitude: scene.altitude,
        stamina: scene.stamina,
        size,
    });
    *since_report = 0.0;
    *frames = 0;
}

fn run_commands(bridge: Option<NonSend<HudBridge>>) {
    let Some(bridge) = bridge else {
        return;
    };
    let Some(states) = bridge.states.get().copied() else {
        return;
    };
    for command in bridge.commands.borrow_mut().drain(..) {
        match command {
            HudCommand::BackToStart => states.controls.update(Controls::back_to_start),
        }
    }
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
mod web {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn start() {
        console_error_panic_hook::set_once();
    }

    /// Runs the game in the page's `app` canvas. `touch` says whether the
    /// page's main pointer is a finger.
    #[wasm_bindgen]
    pub fn run_app(touch: bool) {
        super::app(touch).run();
    }
}
