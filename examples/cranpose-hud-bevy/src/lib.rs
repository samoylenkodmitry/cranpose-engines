//! A Cranpose HUD over Bevy's valley, for the desktop and the web.

use bevy::{
    camera::RenderTarget,
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};
use cranpose_bevy::hud::{CranposeHudPlugin, HudCamera, HudInput};
use demo_scene::{SceneControls, SceneStats};
use demo_ui::{Controls, HudBridge, HudCommand, Stats, Walk};

/// The game: Bevy's valley with a Cranpose HUD over its camera. On the web
/// it draws into the page's `app` canvas.
pub fn app() -> App {
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
    .add_plugins(CranposeHudPlugin::new(|world: &mut World| {
        let bridge = HudBridge::new("Bevy");
        world.insert_non_send(bridge.clone());
        Box::new(move || demo_ui::Hud(bridge.clone()))
    }))
    .add_systems(PostStartup, draw_hud_over_the_camera)
    .add_systems(Update, (look_and_walk, run_commands, sync_hud).chain());
    demo_scene::build(&mut app, RenderTarget::default());
    app
}

fn draw_hud_over_the_camera(mut commands: Commands, cameras: Query<Entity, With<Camera3d>>) {
    for camera in &cameras {
        commands.entity(camera).insert(HudCamera);
    }
}

fn look_and_walk(
    bridge: Option<NonSend<HudBridge>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    hud: Res<HudInput>,
) {
    let Some(states) = bridge
        .as_ref()
        .and_then(|bridge| bridge.states.get().copied())
    else {
        return;
    };
    let held = |codes: [KeyCode; 2]| !hud.keyboard && keys.any_pressed(codes);
    let walk = Walk {
        forward: held([KeyCode::KeyW, KeyCode::ArrowUp]),
        back: held([KeyCode::KeyS, KeyCode::ArrowDown]),
        left: held([KeyCode::KeyA, KeyCode::ArrowLeft]),
        right: held([KeyCode::KeyD, KeyCode::ArrowRight]),
        run: held([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
    };
    let looking = buttons.pressed(MouseButton::Left) && !hud.pointer && motion.delta != Vec2::ZERO;
    if states.controls.value().walk != walk || looking {
        states.controls.update(|controls| {
            controls.walk = walk;
            if looking {
                controls.look(motion.delta.x, motion.delta.y);
            }
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

    /// Runs the game in the page's `app` canvas.
    #[wasm_bindgen]
    pub fn run_app() {
        super::app().run();
    }
}
