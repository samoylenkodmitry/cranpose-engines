#[cfg(target_os = "emscripten")]
mod gl;
#[cfg(target_vendor = "apple")]
mod metal;

use cranpose::{AppLauncher, GuestView, prelude::ExternalTexture};
use godot::{
    classes::{
        CanvasItemMaterial, CanvasLayer, DisplayServer, InputEvent, InputEventKey,
        InputEventMouseButton, InputEventMouseMotion, SubViewport, Texture2D, TextureRect,
        canvas_item_material::BlendMode,
        control::{LayoutPreset, MouseFilter},
        texture_rect::{ExpandMode, StretchMode},
    },
    global::MouseButton,
    prelude::*,
};

#[cfg(target_os = "emscripten")]
use gl::Backend;
#[cfg(target_vendor = "apple")]
use metal::Backend;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const WHEEL_STEP: f32 = 40.0;

/// A Cranpose composition shown over Godot's frame.
///
/// It draws on Godot's own GPU device into a texture that a `TextureRect` on
/// the given `CanvasLayer` shows, so the composition sits above whatever is
/// below that layer: on macOS on Godot's Metal device, and in Godot's web
/// export on its WebGL2 context. Call [`Self::draw`] once per frame and pass
/// input events to [`Self::input`] first.
pub struct Surface {
    guest: Guest,
    pressed: bool,
}

/// A Cranpose composition for a mesh in Godot's world to show, such as the
/// screen of a computer in a game.
///
/// Give a material [`Self::texture`], call [`Self::draw`] once per frame,
/// and hand it the pointer where it lands on the mesh, in the composition's
/// dp: [`Self::point`], [`Self::press`] and [`Self::release`].
pub struct Screen {
    guest: Guest,
    pixels: (u32, u32),
    pressed: bool,
}

// A composition on Godot's device, drawn into the texture a surface or a
// screen shows.
struct Guest {
    backend: Backend,
    view: GuestView,
    density: f32,
}

impl Guest {
    fn new(
        transparent: bool,
        density: f32,
        content: impl FnMut() + 'static,
    ) -> Result<Self, String> {
        let backend = Backend::new()?;
        let mut view = AppLauncher::new()
            .create_guest_view(backend.context(), FORMAT, 1, 1, density, content)
            .map_err(|error| error.to_string())?;
        view.shell()
            .renderer()
            .set_transparent_background(transparent);
        Ok(Self {
            backend,
            view,
            density,
        })
    }

    fn draw(&mut self, width: u32, height: u32) -> Result<(), String> {
        let resized = self
            .view
            .resize(width, height, self.density)
            .map_err(|error| error.to_string())?;
        self.backend.draw(&mut self.view, width, height, resized)
    }
}

impl Surface {
    /// Shows `content` on `layer`. A `transparent` surface lets Godot's frame
    /// show where the composition draws nothing, such as a HUD; an opaque one
    /// fills the layer, such as a whole window drawn by Cranpose.
    pub fn new(
        layer: &mut Gd<CanvasLayer>,
        transparent: bool,
        content: impl FnMut() + 'static,
    ) -> Result<Self, String> {
        let guest = Guest::new(
            transparent,
            DisplayServer::singleton().screen_get_scale(),
            content,
        )?;
        let mut rect = TextureRect::new_alloc();
        rect.set_anchors_preset(LayoutPreset::FULL_RECT);
        rect.set_expand_mode(ExpandMode::IGNORE_SIZE);
        rect.set_stretch_mode(StretchMode::SCALE);
        rect.set_mouse_filter(MouseFilter::IGNORE);
        rect.set_texture(&guest.backend.display());
        if transparent {
            let mut material = CanvasItemMaterial::new_gd();
            material.set_blend_mode(BlendMode::PREMULT_ALPHA);
            rect.set_material(&material);
        }
        layer.add_child(&rect);
        Ok(Self {
            guest,
            pressed: false,
        })
    }

    /// Draws the composition at `size` physical pixels when it changed, and
    /// hands the frame to Godot.
    pub fn draw(&mut self, size: Vector2i) -> Result<(), String> {
        self.guest.draw(size.x.max(1) as u32, size.y.max(1) as u32)
    }

    /// The frames `viewport` renders, for the composition to show, at
    /// `size` physical pixels; it sizes the viewport to match. Call it once
    /// per frame while the composition shows them; `None` until a frame is
    /// ready. Pass [`ExternalTexture::frame`] a new number for each call.
    pub fn viewport_frame(
        &mut self,
        viewport: &mut Gd<SubViewport>,
        size: (u32, u32),
    ) -> Option<ExternalTexture> {
        if size.0 == 0 || size.1 == 0 {
            return None;
        }
        let wanted = Vector2i::new(size.0 as i32, size.1 as i32);
        self.guest.backend.viewport_frame(viewport, wanted)
    }

    /// Passes a pointer, wheel or key event to the composition and returns
    /// whether it took the event. A taken press keeps the pointer until its
    /// release.
    pub fn input(&mut self, event: &Gd<InputEvent>) -> bool {
        let density = self.guest.density;
        let shell = self.guest.view.shell();
        if let Ok(key) = event.clone().try_cast::<InputEventKey>() {
            return shell.on_key_event(&crate::keys::key_event(&key));
        }
        if let Ok(motion) = event.clone().try_cast::<InputEventMouseMotion>() {
            let position = motion.get_position() / density;
            shell.set_cursor(position.x, position.y);
            return self.pressed;
        }
        let Ok(button) = event.clone().try_cast::<InputEventMouseButton>() else {
            return false;
        };
        let position = button.get_position() / density;
        shell.set_cursor(position.x, position.y);
        match button.get_button_index() {
            MouseButton::LEFT if button.is_pressed() => {
                self.pressed = shell.pointer_pressed();
                self.pressed
            }
            MouseButton::LEFT => {
                shell.pointer_released();
                std::mem::take(&mut self.pressed)
            }
            MouseButton::WHEEL_UP if button.is_pressed() => {
                shell.pointer_scrolled(0.0, -WHEEL_STEP)
            }
            MouseButton::WHEEL_DOWN if button.is_pressed() => {
                shell.pointer_scrolled(0.0, WHEEL_STEP)
            }
            _ => false,
        }
    }
}

// Where a screen's cursor goes when the pointer leaves it, so nothing on it
// stays hovered.
const AWAY: Vector2 = Vector2::new(-1.0e6, -1.0e6);

impl Screen {
    /// A composition of `size` dp drawn at `density` pixels per dp.
    pub fn new(
        size: Vector2,
        density: f32,
        content: impl FnMut() + 'static,
    ) -> Result<Self, String> {
        let pixels = (
            (size.x * density).round().max(1.0) as u32,
            (size.y * density).round().max(1.0) as u32,
        );
        Ok(Self {
            guest: Guest::new(false, density, content)?,
            pixels,
            pressed: false,
        })
    }

    /// The texture the composition draws into, for a material.
    pub fn texture(&self) -> Gd<Texture2D> {
        self.guest.backend.display()
    }

    /// Draws the composition when it changed, and hands the frame to Godot.
    pub fn draw(&mut self) -> Result<(), String> {
        self.guest.draw(self.pixels.0, self.pixels.1)
    }

    /// Moves the pointer to `at` on the screen, in dp, or off the screen.
    pub fn point(&mut self, at: Option<Vector2>) {
        let at = at.unwrap_or(AWAY);
        self.guest.view.shell().set_cursor(at.x, at.y);
    }

    /// Presses at `at`, in dp, and returns whether something there took the
    /// press. The screen then holds it until [`Self::release`].
    pub fn press(&mut self, at: Vector2) -> bool {
        let shell = self.guest.view.shell();
        shell.set_cursor(at.x, at.y);
        self.pressed = shell.pointer_pressed();
        if !self.pressed {
            shell.pointer_released();
        }
        self.pressed
    }

    /// Releases the press the screen holds, where the pointer is now, in dp,
    /// or off the screen.
    pub fn release(&mut self, at: Option<Vector2>) {
        if std::mem::take(&mut self.pressed) {
            self.point(at);
            self.guest.view.shell().pointer_released();
        }
    }

    /// Whether the screen holds a press.
    pub fn holds_press(&self) -> bool {
        self.pressed
    }
}
