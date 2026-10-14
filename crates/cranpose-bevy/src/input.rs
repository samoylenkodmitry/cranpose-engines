use bevy::{
    camera::{ManualTextureViewHandle, RenderTarget},
    ecs::system::SystemParam,
    input::{
        ButtonState, InputSystems,
        keyboard::{Key, KeyboardInput},
        mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
        touch::{TouchInput, TouchPhase},
    },
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
    prelude::*,
    window::{CursorMoved, WindowRef},
};
use cranpose::{
    GuestView,
    prelude::{KeyCode as CranposeKey, KeyEvent, KeyEventType, Modifiers, PointerSource},
};

use crate::{hud::HudView, screen::ScreenViews};

const WHEEL_LINE: f32 = 40.0;
// Where a screen's cursor goes when the pointer leaves it, so nothing on it
// stays hovered.
const AWAY: Vec2 = Vec2::splat(-1.0e6);

/// What the Cranpose surfaces, the HUD and the screens in the world, took of
/// the input, for the game to leave alone.
///
/// A press goes to the HUD first, then to the screen nearest the camera
/// under the pointer, and stays with the surface that took it until it
/// lifts. Each surface takes one finger at a time, so the game can follow
/// the others, such as a finger that looks around while the HUD's stick
/// walks.
#[derive(Resource, Default, Debug)]
pub struct CranposeInput {
    /// A surface took the mouse press that is still held.
    pub pointer: bool,
    /// A surface took the last key press, such as typing into a focused
    /// field.
    pub keyboard: bool,
    fingers: Vec<u64>,
}

impl CranposeInput {
    /// Whether a surface took this finger, while it stays down.
    pub fn took_finger(&self, finger: u64) -> bool {
        self.fingers.contains(&finger)
    }

    /// Whether a surface holds the mouse press or a finger.
    pub fn holds_press(&self) -> bool {
        self.pointer || !self.fingers.is_empty()
    }
}

/// Where the pointer's positions lie: in the primary window, or in the
/// frames of a [`crate::BevyView`], which go to this texture view.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PointerTarget {
    #[default]
    PrimaryWindow,
    TextureView(ManualTextureViewHandle),
}

impl PointerTarget {
    fn draws_to(self, target: &RenderTarget) -> bool {
        match (self, target) {
            (Self::PrimaryWindow, RenderTarget::Window(WindowRef::Primary)) => true,
            (Self::TextureView(view), RenderTarget::TextureView(target)) => view == *target,
            _ => false,
        }
    }
}

/// Routes input to the HUD and the screens. The plugins that draw them add
/// it.
pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CranposeInput>()
            .init_resource::<PointerTarget>()
            .add_systems(PreUpdate, route_input.after(InputSystems));
    }
}

/// Adds [`InputPlugin`] unless another plugin of this crate did.
pub(crate) fn add_to(app: &mut App) {
    if !app.is_plugin_added::<InputPlugin>() {
        app.add_plugins(InputPlugin);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Surface {
    Hud,
    Screen(Entity),
}

// Which surface holds what, between frames.
#[derive(Default)]
struct Holders {
    mouse: Option<Surface>,
    keys: Option<Surface>,
    fingers: Vec<(u64, Surface)>,
    hovered: Option<Entity>,
    cursor: Option<Vec2>,
}

#[derive(SystemParam)]
struct PointerInput<'w, 's> {
    cursor: MessageReader<'w, 's, CursorMoved>,
    buttons: MessageReader<'w, 's, MouseButtonInput>,
    touches: MessageReader<'w, 's, TouchInput>,
    wheel: MessageReader<'w, 's, MouseWheel>,
    keys: MessageReader<'w, 's, KeyboardInput>,
    held: Res<'w, ButtonInput<KeyCode>>,
}

// Finds the screen under a point of the window.
#[derive(SystemParam)]
struct Picker<'w, 's> {
    ray_cast: MeshRayCast<'w, 's>,
    target: Res<'w, PointerTarget>,
    cameras: Query<
        'w,
        's,
        (
            &'static Camera,
            &'static GlobalTransform,
            &'static RenderTarget,
        ),
    >,
    screens: Query<'w, 's, &'static crate::screen::CranposeScreen>,
}

// The HUD's and the screens' compositions.
#[derive(SystemParam)]
struct Surfaces<'w> {
    hud: Option<NonSendMut<'w, HudView>>,
    screens: Option<NonSendMut<'w, ScreenViews>>,
}

fn route_input(
    mut surfaces: Surfaces,
    mut picker: Picker,
    mut input: PointerInput,
    mut holders: Local<Holders>,
    mut taken: ResMut<CranposeInput>,
) {
    let holders = &mut *holders;
    route_mouse(&mut surfaces, &mut picker, &mut input, holders, &mut taken);
    for touch in input.touches.read() {
        route_touch(&mut surfaces, &mut picker, touch, holders, &mut taken);
    }
    for scrolled in input.wheel.read() {
        let scale = match scrolled.unit {
            MouseScrollUnit::Line => WHEEL_LINE,
            MouseScrollUnit::Pixel => 1.0,
        };
        let (x, y) = (-scrolled.x * scale, -scrolled.y * scale);
        let on_hud = surfaces
            .view(Surface::Hud)
            .is_some_and(|hud| hud.shell().pointer_scrolled(x, y));
        if !on_hud
            && let Some(screen) = holders.hovered
            && let Some(view) = surfaces.view(Surface::Screen(screen))
        {
            view.shell().pointer_scrolled(x, y);
        }
    }
    route_keys(&mut surfaces, &mut input, holders, &mut taken);
}

fn route_mouse(
    surfaces: &mut Surfaces,
    picker: &mut Picker,
    input: &mut PointerInput,
    holders: &mut Holders,
    taken: &mut CranposeInput,
) {
    let mut moved = None;
    for event in input.cursor.read() {
        if let Some(hud) = surfaces.view(Surface::Hud) {
            let shell = hud.shell();
            shell.set_pointer_source(PointerSource::Mouse);
            shell.set_cursor(event.position.x, event.position.y);
        }
        moved = Some(event.position);
    }
    if let Some(at) = moved {
        holders.cursor = Some(at);
        match holders.mouse {
            Some(Surface::Screen(screen)) => {
                if let Some((_, point)) = picker.screen_at(at, Some(screen)) {
                    surfaces.point(Surface::Screen(screen), point, PointerSource::Mouse);
                }
            }
            _ => hover(surfaces, holders, picker.screen_at(at, None)),
        }
    }
    for button in input.buttons.read() {
        if button.button != MouseButton::Left {
            continue;
        }
        match button.state {
            ButtonState::Pressed => {
                let at = holders.cursor;
                holders.mouse = surfaces.offer(picker, at, &[], PointerSource::Mouse);
                holders.keys = holders.mouse;
            }
            ButtonState::Released => {
                if let Some(view) = holders.mouse.take().and_then(|held| surfaces.view(held)) {
                    view.shell().pointer_released();
                }
            }
        }
        taken.pointer = holders.mouse.is_some();
    }
}

// Moves the cursor of the screen under the pointer, and away from the screen
// it left.
fn hover(surfaces: &mut Surfaces, holders: &mut Holders, under: Option<(Entity, Vec2)>) {
    let now = under.map(|(screen, _)| screen);
    if holders.hovered != now
        && let Some(left) = holders.hovered
    {
        surfaces.point(Surface::Screen(left), AWAY, PointerSource::Mouse);
    }
    holders.hovered = now;
    if let Some((screen, point)) = under {
        surfaces.point(Surface::Screen(screen), point, PointerSource::Mouse);
    }
}

fn route_touch(
    surfaces: &mut Surfaces,
    picker: &mut Picker,
    touch: &TouchInput,
    holders: &mut Holders,
    taken: &mut CranposeInput,
) {
    let held = holders
        .fingers
        .iter()
        .position(|(finger, _)| *finger == touch.id);
    match (touch.phase, held) {
        (TouchPhase::Started, None) => {
            if let Some(surface) = surfaces.offer(
                picker,
                Some(touch.position),
                &holders.fingers,
                PointerSource::Touch,
            ) {
                holders.fingers.push((touch.id, surface));
                taken.fingers.push(touch.id);
            }
        }
        (TouchPhase::Moved, Some(index)) => {
            let surface = holders.fingers[index].1;
            if let Some(point) = surfaces.point_of(picker, surface, touch.position) {
                surfaces.point(surface, point, PointerSource::Touch);
            }
        }
        (TouchPhase::Ended | TouchPhase::Canceled, Some(index)) => {
            let (finger, surface) = holders.fingers.swap_remove(index);
            taken.fingers.retain(|taken| *taken != finger);
            let at = surfaces.point_of(picker, surface, touch.position);
            if let Some(view) = surfaces.view(surface) {
                let shell = view.shell();
                shell.set_pointer_source(PointerSource::Touch);
                if touch.phase == TouchPhase::Canceled {
                    shell.cancel_gesture();
                } else {
                    if let Some(point) = at {
                        shell.set_cursor(point.x, point.y);
                    }
                    shell.pointer_released();
                }
            }
        }
        _ => {}
    }
}

// Keys go to the HUD, then to the surface pressed last.
fn route_keys(
    surfaces: &mut Surfaces,
    input: &mut PointerInput,
    holders: &Holders,
    taken: &mut CranposeInput,
) {
    let held = &input.held;
    let modifiers = Modifiers {
        shift: held.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        ctrl: held.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
        alt: held.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
        meta: held.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]),
    };
    for key in input.keys.read() {
        let (kind, text) = match (key.state, &key.logical_key) {
            (ButtonState::Pressed, Key::Character(text)) => (KeyEventType::KeyDown, text.as_str()),
            (ButtonState::Pressed, _) => (KeyEventType::KeyDown, ""),
            (ButtonState::Released, _) => (KeyEventType::KeyUp, ""),
        };
        let event = KeyEvent::new(cranpose_key(key.key_code), text, modifiers, kind);
        let mut took = surfaces
            .view(Surface::Hud)
            .is_some_and(|hud| hud.shell().on_key_event(&event));
        if !took
            && let Some(screen @ Surface::Screen(_)) = holders.keys
            && let Some(view) = surfaces.view(screen)
        {
            took = view.shell().on_key_event(&event);
        }
        if kind == KeyEventType::KeyDown {
            taken.keyboard = took;
        }
    }
}

// Bevy names its keys as the W3C `KeyboardEvent.code` values do, but for the
// Super keys.
fn cranpose_key(key: KeyCode) -> CranposeKey {
    match key {
        KeyCode::SuperLeft => CranposeKey::MetaLeft,
        KeyCode::SuperRight => CranposeKey::MetaRight,
        other => CranposeKey::from_dom_code(&format!("{other:?}")),
    }
}

impl Surfaces<'_> {
    fn view(&mut self, surface: Surface) -> Option<&mut GuestView> {
        match surface {
            Surface::Hud => self.hud.as_mut().map(|hud| &mut hud.view),
            Surface::Screen(screen) => self.screens.as_mut()?.view(screen),
        }
    }

    fn point(&mut self, surface: Surface, point: Vec2, source: PointerSource) {
        if let Some(view) = self.view(surface) {
            let shell = view.shell();
            shell.set_pointer_source(source);
            shell.set_cursor(point.x, point.y);
        }
    }

    // Where a point of the window lands on `surface`.
    fn point_of(&self, picker: &mut Picker, surface: Surface, at: Vec2) -> Option<Vec2> {
        match surface {
            Surface::Hud => Some(at),
            Surface::Screen(screen) => picker.screen_at(at, Some(screen)).map(|(_, point)| point),
        }
    }

    // Presses the HUD, then the screen under `at`, and returns the surface
    // that had something under the press. One that had nothing gets the
    // release at once. The surfaces in `fingers` already hold a finger.
    fn offer(
        &mut self,
        picker: &mut Picker,
        at: Option<Vec2>,
        fingers: &[(u64, Surface)],
        source: PointerSource,
    ) -> Option<Surface> {
        let free = |surface: Surface| fingers.iter().all(|(_, held)| *held != surface);
        let at = at?;
        let mut candidates = [Some(Surface::Hud), None];
        if let Some((screen, _)) = picker.screen_at(at, None) {
            candidates[1] = Some(Surface::Screen(screen));
        }
        for surface in candidates
            .into_iter()
            .flatten()
            .filter(|surface| free(*surface))
        {
            let Some(point) = self.point_of(picker, surface, at) else {
                continue;
            };
            let Some(view) = self.view(surface) else {
                continue;
            };
            let shell = view.shell();
            shell.set_pointer_source(source);
            shell.set_cursor(point.x, point.y);
            if shell.pointer_pressed() {
                return Some(surface);
            }
            shell.pointer_released();
        }
        None
    }
}

impl Picker<'_, '_> {
    // The ray through a point of the pointer's target, from the active
    // camera drawn last there.
    fn ray(&self, at: Vec2) -> Option<Ray3d> {
        let (camera, transform, _) = self
            .cameras
            .iter()
            .filter(|(camera, _, target)| camera.is_active && self.target.draws_to(target))
            .max_by_key(|(camera, ..)| camera.order)?;
        let origin = camera
            .logical_viewport_rect()
            .map_or(Vec2::ZERO, |rect| rect.min);
        camera.viewport_to_world(transform, at - origin).ok()
    }

    // The nearest screen under a point of the window within its reach, or
    // only `only`, and the point on it in dp.
    fn screen_at(&mut self, at: Vec2, only: Option<Entity>) -> Option<(Entity, Vec2)> {
        let ray = self.ray(at)?;
        let filter = |entity: Entity| only.is_none_or(|only| only == entity);
        let settings = MeshRayCastSettings::default()
            .with_filter(&filter)
            .always_early_exit();
        let (entity, hit) = self.ray_cast.cast_ray(ray, &settings).first()?;
        let screen = self.screens.get(*entity).ok()?;
        if hit.distance > screen.reach {
            return None;
        }
        Some((*entity, hit.uv? * screen.size))
    }
}
