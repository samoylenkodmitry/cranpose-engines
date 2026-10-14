use demo_ui::{HudBridge, ScenePointer, Stats, Walk};
use godot::{
    classes::{
        CanvasLayer, DisplayServer, Engine, INode, Input, InputEvent, InputEventMouseButton,
        InputEventMouseMotion, Node,
    },
    global::{Key, MouseButton, MouseButtonMask},
    prelude::*,
};

use cranpose_godot::Surface;

use crate::world::DemoWorld;

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct CranposeHudDemo {
    base: Base<Node>,
    world: Option<Gd<DemoWorld>>,
    surface: Option<Surface>,
    bridge: Option<HudBridge>,
    since_report: f64,
    frames: u32,
}

#[godot_api]
impl INode for CranposeHudDemo {
    fn ready(&mut self) {
        let scale = crate::scene_scale();
        if let Some(mut viewport) = self.base().get_viewport() {
            viewport.set_scaling_3d_scale(scale);
        }
        let world = DemoWorld::new_alloc();
        self.base_mut().add_child(&world);
        self.world = Some(world);
        let mut layer = CanvasLayer::new_alloc();
        self.base_mut().add_child(&layer);
        // The web page passes `touch` when its main pointer is a finger; the
        // HUD then shows its stick before the first touch.
        let bridge = HudBridge::new("Godot", crate::user_arg("touch"));
        let content = bridge.clone();
        match Surface::new(&mut layer, true, move || demo_ui::Hud(content.clone())) {
            Ok(surface) => self.surface = Some(surface),
            Err(error) => {
                godot_error!("Cranpose HUD: {error}");
            }
        }
        self.bridge = Some(bridge);
    }

    fn process(&mut self, delta: f64) {
        let size = self
            .base()
            .get_viewport()
            .map(|viewport| viewport.get_visible_rect().size);
        let (Some(bridge), Some(world)) = (self.bridge.clone(), self.world.as_mut()) else {
            return;
        };
        if let Some(states) = bridge.states.get().copied() {
            for command in bridge.commands.borrow_mut().drain(..) {
                states.controls.update(|controls| command.apply(controls));
            }
            world.bind().drain_office_commands(|command| {
                states.controls.update(|controls| command.apply(controls))
            });
            let walk = Walk {
                stick: states.controls.value().walk.stick,
                ..held_walk()
            };
            if states.controls.value().walk != walk {
                states.controls.update(|controls| controls.walk = walk);
            }
            world.bind_mut().set_controls(states.controls.value());
            self.since_report += delta;
            self.frames += 1;
            if self.since_report >= 0.25 {
                let (altitude, stamina) = world.bind().altitude_and_stamina();
                states.stats.set(Stats {
                    fps: Engine::singleton().get_frames_per_second().round() as u32,
                    altitude,
                    stamina,
                    size: size.map_or((0, 0), |size| (size.x as u32, size.y as u32)),
                });
                self.since_report = 0.0;
                self.frames = 0;
            }
        }
        if let (Some(surface), Some(size)) = (self.surface.as_mut(), size)
            && let Err(error) = surface.draw(Vector2i::new(size.x as i32, size.y as i32))
        {
            godot_error!("Cranpose HUD draw: {error}");
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        let took = self
            .surface
            .as_mut()
            .is_some_and(|surface| surface.input(&event));
        // A press the HUD left may land on the office's screen, which then
        // keeps the pointer until the release.
        let on_screen = scene_pointer(&event).is_some_and(|sample| {
            self.world
                .as_mut()
                .is_some_and(|world| world.bind_mut().office_pointer(sample))
        });
        if took || on_screen {
            if let Some(mut viewport) = self.base().get_viewport() {
                viewport.set_input_as_handled();
            }
            return;
        }
        let Ok(motion) = event.try_cast::<InputEventMouseMotion>() else {
            return;
        };
        if !motion.get_button_mask().is_set(MouseButtonMask::LEFT) {
            return;
        }
        let Some(states) = self
            .bridge
            .as_ref()
            .and_then(|bridge| bridge.states.get().copied())
        else {
            return;
        };
        let delta = motion.get_relative() / DisplayServer::singleton().screen_get_scale();
        states
            .controls
            .update(|controls| controls.look(delta.x, delta.y));
    }
}

// The left button's presses and the pointer's moves, in the window's pixels.
fn scene_pointer(event: &Gd<InputEvent>) -> Option<ScenePointer> {
    if let Ok(motion) = event.clone().try_cast::<InputEventMouseMotion>() {
        let at = motion.get_position();
        return Some(ScenePointer::Moved([at.x, at.y]));
    }
    let button = event.clone().try_cast::<InputEventMouseButton>().ok()?;
    if button.get_button_index() != MouseButton::LEFT {
        return None;
    }
    let at = [button.get_position().x, button.get_position().y];
    Some(if button.is_pressed() {
        ScenePointer::Pressed(at)
    } else {
        ScenePointer::Released(at)
    })
}

fn held_walk() -> Walk {
    let input = Input::singleton();
    let held = |keys: [Key; 2]| {
        keys.into_iter()
            .any(|key| input.is_physical_key_pressed(key))
    };
    Walk {
        forward: held([Key::W, Key::UP]),
        back: held([Key::S, Key::DOWN]),
        left: held([Key::A, Key::LEFT]),
        right: held([Key::D, Key::RIGHT]),
        run: held([Key::SHIFT, Key::SHIFT]),
        stick: [0.0, 0.0],
    }
}
