//! A Godot extension with one node, `CranposeHud`: it draws a Cranpose HUD
//! over the scene and turns the node its `target` names at the speed the HUD
//! sets. `hud.rs` is the HUD.

// godot-rust's `GodotClass` derive writes `base: base`.
#![allow(clippy::redundant_field_names)]

mod hud;

use std::f64::consts::TAU;

use cranpose_godot::Surface;
use godot::{
    classes::{CanvasLayer, INode, InputEvent, Node, Node3D},
    prelude::*,
};

cranpose_godot::godot_extension!(Extension);

#[derive(GodotClass)]
#[class(base = Node, init)]
struct CranposeHud {
    base: Base<Node>,
    /// The node the HUD turns.
    #[export]
    target: Option<Gd<Node3D>>,
    hud: hud::Hud,
    surface: Option<Surface>,
    angle: f64,
}

#[godot_api]
impl INode for CranposeHud {
    fn ready(&mut self) {
        let mut layer = CanvasLayer::new_alloc();
        self.base_mut().add_child(&layer);
        let hud = self.hud.clone();
        match Surface::new(&mut layer, true, move || hud::Panel(hud.clone())) {
            Ok(surface) => self.surface = Some(surface),
            Err(error) => {
                godot_error!("Cranpose HUD: {error}");
            }
        }
    }

    fn process(&mut self, delta: f64) {
        if let Some(state) = self.hud.state() {
            let step = f64::from(state.speed.value()) * TAU * delta;
            if let Some(target) = self.target.as_mut() {
                target.rotate_y(step as f32);
            }
            self.angle += step;
            let turns = (self.angle / TAU) as u32;
            if state.turns.value() != turns {
                state.turns.set(turns);
            }
        }
        let Some(viewport) = self.base().get_viewport() else {
            return;
        };
        let size = viewport.get_visible_rect().size;
        if let Some(surface) = self.surface.as_mut()
            && let Err(error) = surface.draw(Vector2i::new(size.x as i32, size.y as i32))
        {
            godot_error!("Cranpose HUD: {error}");
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        let took = self
            .surface
            .as_mut()
            .is_some_and(|surface| surface.input(&event));
        if took && let Some(mut viewport) = self.base().get_viewport() {
            viewport.set_input_as_handled();
        }
    }
}
