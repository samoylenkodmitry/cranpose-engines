use cranpose_godot::Surface;
use demo_ui::{EngineInfo, WindowBridge};
use godot::{
    classes::{
        CanvasLayer, Engine, INode, InputEvent, Node, SubViewport, sub_viewport::UpdateMode,
    },
    prelude::*,
};

use crate::world::DemoWorld;

#[cfg(target_vendor = "apple")]
const GODOT: EngineInfo = EngineInfo {
    name: "Godot",
    description: "Godot 4.7 renders this scene with its own Metal renderer. Cranpose renders on the same Metal device, shows the frame as an image, clips it and draws glass over it. Frames cross between the two queues by GPU copies; no pixels pass through the CPU.",
    features: "Procedural sky, shadows, SSAO, fog, glow",
};

#[cfg(target_os = "emscripten")]
const GODOT: EngineInfo = EngineInfo {
    name: "Godot",
    description: "Godot 4.7's web export renders this scene with WebGL2. Cranpose runs inside Godot's page and draws on the same WebGL2 context: it samples Godot's frame where Godot drew it, clips it and draws glass over it. No pixels are copied.",
    features: "Procedural sky, shadows, fog, glow",
};

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct CranposeWindowDemo {
    base: Base<Node>,
    world: Option<Gd<DemoWorld>>,
    viewport: Option<Gd<SubViewport>>,
    surface: Option<Surface>,
    bridge: WindowBridge,
    frame: u64,
    since_report: f64,
    scale: f32,
}

#[godot_api]
impl INode for CranposeWindowDemo {
    fn ready(&mut self) {
        self.scale = crate::scene_scale();
        self.bridge.touch = crate::user_arg("touch");
        let mut viewport = SubViewport::new_alloc();
        viewport.set_use_own_world_3d(true);
        viewport.set_update_mode(UpdateMode::ALWAYS);
        viewport.set_size(Vector2i::new(64, 64));
        let world = DemoWorld::new_alloc();
        viewport.add_child(&world);
        self.base_mut().add_child(&viewport);
        self.world = Some(world);
        self.viewport = Some(viewport);
        let mut layer = CanvasLayer::new_alloc();
        self.base_mut().add_child(&layer);
        let bridge = self.bridge.clone();
        match Surface::new(&mut layer, false, move || {
            demo_ui::HostedEngineWindow(GODOT, bridge.clone())
        }) {
            Ok(surface) => self.surface = Some(surface),
            Err(error) => {
                godot_error!("Cranpose window: {error}");
            }
        }
    }

    fn process(&mut self, delta: f64) {
        let size = self
            .base()
            .get_viewport()
            .map(|viewport| viewport.get_visible_rect().size);
        if let Some(states) = self.bridge.states.get().copied() {
            if let Some(world) = self.world.as_mut() {
                // The view's pointer reaches the office's screen in Godot's
                // world, whose terminal changes the window's controls.
                self.bridge
                    .pointer
                    .deliver(|sample| world.bind_mut().office_pointer(sample));
                world.bind().drain_office_commands(|command| {
                    states.controls.update(|controls| command.apply(controls))
                });
                world.bind_mut().set_controls(states.controls.value());
            }
            self.since_report += delta;
            if self.since_report >= 0.5 {
                states
                    .fps
                    .set(Engine::singleton().get_frames_per_second().round() as u32);
                self.since_report = 0.0;
            }
            let (width, height) = self.bridge.size.0.get();
            let scaled = |pixels: u32| (pixels as f32 * self.scale).round() as u32;
            let wanted = (scaled(width), scaled(height));
            if let (Some(surface), Some(viewport)) = (self.surface.as_mut(), self.viewport.as_mut())
                && let Some(texture) = surface.viewport_frame(viewport, wanted)
            {
                self.frame += 1;
                states.frame.set(Some(texture.frame(self.frame)));
            }
        }
        if let (Some(surface), Some(size)) = (self.surface.as_mut(), size)
            && let Err(error) = surface.draw(Vector2i::new(size.x as i32, size.y as i32))
        {
            godot_error!("Cranpose window draw: {error}");
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Some(surface) = self.surface.as_mut() {
            surface.input(&event);
        }
        if let Some(mut viewport) = self.base().get_viewport() {
            viewport.set_input_as_handled();
        }
    }
}
