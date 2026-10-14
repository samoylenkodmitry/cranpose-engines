//! Cranpose in Godot 4.
//!
//! [`Surface`] draws a composition into a texture that a `CanvasLayer` shows
//! over Godot's frame, on Godot's own GPU device, hands it Godot's input,
//! and shows it the frames of Godot's viewports. [`godot_extension!`]
//! declares a GDExtension's entry point. It runs on macOS with Godot's Metal
//! driver, and in Godot's web export on its WebGL2 context.

#[cfg(any(target_vendor = "apple", target_os = "emscripten"))]
mod keys;
#[cfg(any(target_vendor = "apple", target_os = "emscripten"))]
mod surface;

pub use cranpose_gpu_interop::godot_extension;
#[cfg(any(target_vendor = "apple", target_os = "emscripten"))]
pub use surface::{Screen, Surface};
