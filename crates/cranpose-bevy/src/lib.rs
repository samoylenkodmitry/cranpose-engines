//! Bevy and Cranpose on one wgpu device.
//!
//! [`BevyView`] runs a Bevy app on its own thread and hands its frames to a
//! composition; [`rememberBevyView`] starts one and [`BevyScene`] shows it.
//! [`hud::CranposeHudPlugin`] draws a composition over a Bevy camera, and
//! [`screen::CranposeScreenPlugin`] draws compositions on meshes in the
//! world. All share the device without copies: Bevy needs the `wgpu` package
//! that Cranpose uses, which the engines workspace patches in.

mod gpu;
pub mod hud;
mod input;
mod scene;
pub mod screen;
mod view;

use bevy::ecs::world::World;
use bevy::render::{
    renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance, RenderQueue},
    settings::RenderCreation,
};
use cranpose::{GpuContext, GpuRequirements};
pub use input::CranposeInput;
pub use scene::{BevyScene, rememberBevyView};
pub use view::{BevyView, ViewPointer};

// Gets Bevy's world once and returns a composition's root.
type ContentFactory = dyn Fn(&mut World) -> Box<dyn FnMut()> + Send + Sync;

/// What Bevy's renderer needs from Cranpose's device: every feature the
/// adapter offers but mappable primary buffers, and the adapter's limits.
pub fn gpu_requirements() -> GpuRequirements {
    GpuRequirements {
        features: wgpu::Features::all() - wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
        adapter_limits: true,
    }
}

/// Bevy's renderer on `gpu`, for Bevy's `RenderPlugin`.
pub fn render_creation(gpu: &GpuContext) -> RenderCreation {
    RenderCreation::manual(
        RenderDevice::new((*gpu.device).clone()),
        RenderQueue::new((*gpu.queue).clone()),
        RenderAdapterInfo::new(gpu.adapter.get_info()),
        RenderAdapter::new(gpu.adapter.clone()),
        RenderInstance::new(gpu.instance.clone()),
    )
}
