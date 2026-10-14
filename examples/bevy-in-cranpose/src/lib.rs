//! Bevy's valley inside a Cranpose window, for the desktop and the web.

use std::{cell::Cell, rc::Rc, sync::mpsc};

use bevy::prelude::{ResMut, Update};
use cranpose::{prelude::*, *};
use cranpose_bevy::{BevyScene, rememberBevyView};
use demo_scene::{Quality, SceneCommands, SceneControls};
use demo_ui::SceneCommand;
use demo_ui::{EngineInfo, EngineWindow, rememberWindowStates};

const BEVY: EngineInfo = EngineInfo {
    name: "Bevy",
    description: "Bevy 0.20 renders this scene on its own thread with Cranpose's wgpu device. Cranpose shows each finished frame as an image, clips it and draws glass over it. No pixels are copied, and the interface never waits for Bevy.",
    #[cfg(not(feature = "webgl2"))]
    features: "Earth atmosphere, cascaded shadows, fog, bloom",
    #[cfg(feature = "webgl2")]
    features: "Cascaded shadows, fog, bloom; WebGL2 has no atmosphere",
};

/// The launcher with Bevy's device requests.
pub fn launcher() -> AppLauncher {
    AppLauncher::new()
        .with_title("Bevy inside Cranpose")
        .with_size(1320, 820)
        .with_gpu_requirements(cranpose_bevy::gpu_requirements())
        .with_developer_inspector(false)
}

// The most pixels per dp Bevy renders on a phone; Cranpose draws its own
// interface at the screen's full density.
const PHONE_DENSITY: f32 = 1.5;

/// The window: the scene controls beside Bevy's view of the valley. `touch`
/// says whether the screen is a phone's touch screen: the window then shows
/// the touch stick before the first touch, and Bevy renders the phone's
/// quality at no more than 1.5 pixels per dp.
#[composable]
pub fn Root(touch: bool) {
    let states = rememberWindowStates(touch);
    let quality = if touch { Quality::Phone } else { Quality::Full };
    // The scene's commands, such as the old computer's, come back over this
    // channel from Bevy's thread.
    let (to_window, from_scene) = remember(|| {
        let (sender, receiver) = mpsc::channel::<SceneCommand>();
        (sender, Rc::new(receiver))
    })
    .with(|(sender, receiver)| (sender.clone(), Rc::clone(receiver)));
    let bevy = rememberBevyView(move |app, target| {
        demo_scene::build(app, target, quality);
        app.add_systems(Update, move |mut commands: ResMut<SceneCommands>| {
            for command in commands.0.drain(..) {
                let _ = to_window.send(command);
            }
        });
    });
    if let Some(bevy) = &bevy
        && touch
    {
        bevy.set_max_density(PHONE_DENSITY);
    }
    let controls = states.controls.value();
    if let Some(bevy) = bevy.clone() {
        LaunchedEffect(controls, move |_| {
            bevy.send(move |world| world.resource_mut::<SceneControls>().0 = controls);
        });
    }
    let counted = bevy.clone();
    let counter = remember(|| Rc::new(Cell::new((0u64, 0u64)))).with(Rc::clone);
    FrameEffect((), bevy.is_some(), move |now| {
        let Some(bevy) = counted.as_ref() else {
            return;
        };
        for command in from_scene.try_iter() {
            states.controls.update(|controls| command.apply(controls));
        }
        let (since, shown) = counter.get();
        let elapsed = now.saturating_sub(since);
        if since == 0 {
            counter.set((now, bevy.shown_frames()));
        } else if elapsed >= 500_000_000 {
            let frames = bevy.shown_frames() - shown;
            let per_second = (frames * 1_000_000_000 + elapsed / 2) / elapsed;
            states.fps.set(per_second as u32);
            counter.set((now, bevy.shown_frames()));
        }
    });
    EngineWindow(BEVY, states, move || {
        if let Some(bevy) = bevy.clone() {
            BevyScene(Modifier::empty().fill_max_size(), bevy);
        }
    });
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
mod web {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn start() {
        console_error_panic_hook::set_once();
    }

    /// Runs the window in the page's `app` canvas. `touch` says whether the
    /// page's main pointer is a finger.
    #[wasm_bindgen]
    pub async fn run_app(touch: bool) -> Result<(), JsValue> {
        super::launcher()
            .with_web_fill_viewport(true)
            .run_web("app", move || super::Root(touch))
            .await
    }
}
