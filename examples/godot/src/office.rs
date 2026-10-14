//! The office things left on the river's bank (`nature::office`), in
//! Godot: their screen is a Cranpose composition, and their tube light
//! flickers.

use cranpose_godot::Screen;
use demo_ui::{Controls, Light, SceneCommand, ScenePointer, TerminalBridge, TerminalState};
use godot::{
    classes::{
        ArrayMesh, BoxMesh, Camera3D, CylinderMesh, Mesh, MeshInstance3D, Node3D,
        StandardMaterial3D,
        base_material_3d::{Feature, ShadingMode, TextureFilter, TextureParam},
        mesh::{ArrayType, PrimitiveType},
    },
    prelude::*,
};
use nature::{
    Terrain,
    office::{
        Look, Office, Part, REACH, SCREEN_DENSITY, SCREEN_DP, Shape, glass_mesh, lamp_lit,
        screen_hit,
    },
};

// Emission energy of the glowing parts, bright enough for the glow to catch.
const GLOW_ENERGY: f32 = 4.0;

pub(crate) struct OfficeNodes {
    screen: Option<Screen>,
    glass: Part,
    lamp: Option<Gd<StandardMaterial3D>>,
    lit: bool,
    terminal: TerminalBridge,
}

impl OfficeNodes {
    /// Builds the office on `terrain`, handing each part's node to `add`.
    pub(crate) fn place(terrain: &Terrain, mut add: impl FnMut(Gd<Node3D>)) -> Option<Self> {
        let office = Office::place(terrain);
        let glass = *office.screen()?;
        let terminal = TerminalBridge::default();
        let content = terminal.clone();
        let screen = Screen::new(
            Vector2::new(SCREEN_DP[0], SCREEN_DP[1]),
            SCREEN_DENSITY,
            move || demo_ui::Terminal(content.clone()),
        )
        .inspect_err(|error| {
            godot_error!("Cranpose screen: {error}");
        })
        .ok();
        let mut lamp = None;
        for part in &office.parts {
            let mut node = MeshInstance3D::new_alloc();
            node.set_mesh(&mesh(part.shape));
            let material = match (part.look, &screen) {
                (Look::Screen, Some(screen)) => screen_material(screen),
                (look, _) => material(look),
            };
            if part.look == Look::Lamp {
                lamp = Some(material.clone());
            }
            node.set_material_override(&material);
            let [x, y, z, w] = part.rotation;
            let [tx, ty, tz] = part.translation;
            node.set_transform(Transform3D::new(
                Basis::from_quaternion(Quaternion::new(x, y, z, w)),
                Vector3::new(tx, ty, tz),
            ));
            add(node.upcast());
        }
        Some(Self {
            screen,
            glass,
            lamp,
            lit: true,
            terminal,
        })
    }

    /// Flickers the lamp, shows the terminal the light, whether the walker
    /// at `eye` stands near enough to use it and its cursor's blink, and
    /// draws the screen.
    pub(crate) fn update(&mut self, seconds: f32, eye: Vector3, controls: &Controls) {
        let lit = lamp_lit(seconds);
        if lit != self.lit
            && let Some(lamp) = self.lamp.as_mut()
        {
            lamp.set_emission_energy_multiplier(if lit { GLOW_ENERGY } else { 0.0 });
            self.lit = lit;
        }
        let [x, y, z] = self.glass.translation;
        let state = TerminalState {
            light: Light::of(controls),
            near: eye.distance_to(Vector3::new(x, y, z)) <= REACH,
            cursor: seconds.fract() < 0.5,
        };
        if let Some(shown) = self.terminal.state.get()
            && shown.value() != state
        {
            shown.set(state);
        }
        if let Some(screen) = self.screen.as_mut()
            && let Err(error) = screen.draw()
        {
            godot_error!("Cranpose screen draw: {error}");
        }
    }

    /// Hands `command`s the terminal sent since the last call to `apply`.
    pub(crate) fn drain_commands(&self, apply: impl FnMut(SceneCommand)) {
        self.terminal
            .commands
            .borrow_mut()
            .drain(..)
            .for_each(apply);
    }

    /// Hands the screen a pointer sample in `camera`'s viewport pixels and
    /// returns whether the screen holds the press after it.
    pub(crate) fn pointer(&mut self, camera: &Gd<Camera3D>, sample: ScenePointer) -> bool {
        let Some(screen) = self.screen.as_mut() else {
            return false;
        };
        let glass = self.glass;
        let on_screen = |[x, y]: [f32; 2]| {
            let position = Vector2::new(x, y);
            let origin = camera.project_ray_origin(position);
            let direction = camera.project_ray_normal(position);
            screen_hit(
                &glass,
                [origin.x, origin.y, origin.z],
                [direction.x, direction.y, direction.z],
            )
            .filter(|(distance, _)| *distance <= REACH)
            .map(|(_, [x, y])| Vector2::new(x, y))
        };
        match sample {
            ScenePointer::Moved(at) => {
                let point = on_screen(at);
                if point.is_some() || !screen.holds_press() {
                    screen.point(point);
                }
            }
            ScenePointer::Pressed(at) => {
                if let Some(point) = on_screen(at) {
                    screen.press(point);
                }
            }
            ScenePointer::Released(at) => screen.release(on_screen(at)),
        }
        screen.holds_press()
    }
}

fn mesh(shape: Shape) -> Gd<Mesh> {
    match shape {
        Shape::Cuboid([width, height, depth]) => {
            let mut mesh = BoxMesh::new_gd();
            mesh.set_size(Vector3::new(width, height, depth));
            mesh.upcast()
        }
        Shape::Cylinder { radius, height } => {
            let mut mesh = CylinderMesh::new_gd();
            mesh.set_top_radius(radius);
            mesh.set_bottom_radius(radius);
            mesh.set_height(height);
            mesh.upcast()
        }
        Shape::Glass => glass().upcast(),
    }
}

// Godot's front faces wind clockwise, so each triangle turns over.
fn glass() -> Gd<ArrayMesh> {
    let mesh = glass_mesh();
    let mut arrays = VarArray::new();
    arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
    let positions: PackedVector3Array = mesh
        .positions
        .iter()
        .map(|[x, y, z]| Vector3::new(*x, *y, *z))
        .collect();
    let normals: PackedVector3Array = mesh.positions.iter().map(|_| Vector3::BACK).collect();
    let uvs: PackedVector2Array = mesh.uvs.iter().map(|[u, v]| Vector2::new(*u, *v)).collect();
    let indices: PackedInt32Array = mesh
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .flat_map(|[a, b, c]| [*a as i32, *c as i32, *b as i32])
        .collect();
    arrays.set(ArrayType::VERTEX.ord() as usize, &positions.to_variant());
    arrays.set(ArrayType::NORMAL.ord() as usize, &normals.to_variant());
    arrays.set(ArrayType::TEX_UV.ord() as usize, &uvs.to_variant());
    arrays.set(ArrayType::INDEX.ord() as usize, &indices.to_variant());
    let mut glass = ArrayMesh::new_gd();
    glass.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
    glass
}

fn material(look: Look) -> Gd<StandardMaterial3D> {
    let mut material = StandardMaterial3D::new_gd();
    let [r, g, b] = look.color();
    material.set_albedo(Color::from_rgb(r, g, b));
    material.set_roughness(look.roughness());
    if let Some([r, g, b]) = look.glow() {
        material.set_feature(Feature::EMISSION, true);
        material.set_emission(Color::from_rgb(r, g, b));
        material.set_emission_energy_multiplier(GLOW_ENERGY);
    }
    material
}

// The screen gives off its own light: no shading, the composition as it is.
fn screen_material(screen: &Screen) -> Gd<StandardMaterial3D> {
    let mut material = StandardMaterial3D::new_gd();
    material.set_shading_mode(ShadingMode::UNSHADED);
    material.set_texture(TextureParam::ALBEDO, &screen.texture());
    material.set_texture_filter(TextureFilter::LINEAR);
    material
}
