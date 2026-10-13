//! Bevy's valley inside a Cranpose window, for the desktop and the web.

use std::{cell::Cell, rc::Rc};

use cranpose::{prelude::*, *};
use cranpose_bevy::{BevyScene, rememberBevyView};
use demo_scene::SceneControls;
use demo_ui::{EngineInfo, EngineWindow, rememberWindowStates};

const BEVY: EngineInfo = EngineInfo {
    name: "Bevy",
    description: "Bevy 0.20 renders this scene on its own thread with Cranpose's wgpu device. Cranpose shows each finished frame as an image, clips it and draws glass over it. No pixels are copied, and the interface never waits for Bevy.",
    features: "Earth atmosphere, cascaded shadows, fog, bloom",
};

/// The launcher with Bevy's device requests.
pub fn launcher() -> AppLauncher {
    AppLauncher::new()
        .with_title("Bevy inside Cranpose")
        .with_size(1320, 820)
        .with_gpu_requirements(cranpose_bevy::gpu_requirements())
        .with_developer_inspector(false)
}

/// The window: the scene controls beside Bevy's view of the valley.
#[composable]
pub fn Root() {
    let states = rememberWindowStates();
    let bevy = rememberBevyView(demo_scene::build);
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

    /// Runs the window in the page's `app` canvas.
    #[wasm_bindgen]
    pub async fn run_app() -> Result<(), JsValue> {
        super::launcher()
            .with_web_fill_viewport(true)
            .run_web("app", super::Root)
            .await
    }
}
