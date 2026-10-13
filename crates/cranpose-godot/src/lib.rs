//! Cranpose in Godot 4.
//!
//! [`Surface`] draws a composition into a texture that a `CanvasLayer` shows
//! over Godot's frame, on Godot's own Metal device, and hands it Godot's
//! input. [`godot_extension!`] declares a GDExtension's entry point. It runs
//! on macOS with Godot's Metal driver for now.

#[cfg(target_vendor = "apple")]
mod keys;
#[cfg(target_vendor = "apple")]
mod surface;

#[cfg(target_vendor = "apple")]
pub use cranpose_gpu_interop::{
    godot::{Alpha, GodotGpu, GodotTexture},
    godot_extension,
};
#[cfg(target_vendor = "apple")]
pub use surface::Surface;
