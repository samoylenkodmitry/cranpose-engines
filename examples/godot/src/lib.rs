#![expect(
    clippy::redundant_field_names,
    reason = "godot-rust's `#[class(init)]` derive writes `base: base`"
)]

mod hud_demo;
mod office;
mod window_demo;
mod world;

use godot::{
    classes::{DisplayServer, INode, Node, Os, RenderingServer},
    prelude::*,
};

use crate::{hud_demo::CranposeHudDemo, window_demo::CranposeWindowDemo};

cranpose_godot::godot_extension!(CranposeExtension);

/// The web export's root: the page names the demo in the arguments it gives
/// the game, since Godot's web templates take no scene path.
#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct CranposeWebDemo {
    base: Base<Node>,
}

#[godot_api]
impl INode for CranposeWebDemo {
    fn ready(&mut self) {
        let demo: Gd<Node> = if user_arg("window") {
            CranposeWindowDemo::new_alloc().upcast()
        } else {
            CranposeHudDemo::new_alloc().upcast()
        };
        self.base_mut().add_child(&demo);
    }
}

// The most pixels per dp the scene renders on a phone; Cranpose draws its
// own interface at the screen's full density.
const PHONE_DENSITY: f32 = 1.5;

/// The share of the screen's pixels the 3D scene renders, and the quality
/// it renders at. On a phone's touch screen, which the web page names with
/// `touch`, the scene renders at no more than 1.5 pixels per dp with shadow
/// maps of 1024 pixels, as the Bevy examples do; the screen's two to three
/// pixels per dp hide the edges' steps.
fn scene_scale() -> f32 {
    if !user_arg("touch") {
        return 1.0;
    }
    RenderingServer::singleton().directional_shadow_atlas_set_size(1024, true);
    (PHONE_DENSITY / DisplayServer::singleton().screen_get_scale()).min(1.0)
}

// Whether the arguments after `--` include `name`.
fn user_arg(name: &str) -> bool {
    Os::singleton()
        .get_cmdline_user_args()
        .as_slice()
        .iter()
        .any(|arg| *arg == name)
}
