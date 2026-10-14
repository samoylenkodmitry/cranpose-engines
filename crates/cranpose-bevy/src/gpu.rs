use std::sync::Arc;

use bevy::{
    prelude::*,
    render::{
        RenderApp,
        renderer::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue},
    },
};
use cranpose::GpuContext;

/// Bevy's instance, adapter, device and queue as Cranpose's GPU context, for
/// the compositions this crate draws on Bevy's device.
#[derive(Resource)]
pub(crate) struct CranposeGpu(pub(crate) GpuContext);

/// Inserts [`CranposeGpu`] once the render world has its device. Plugins
/// call it from `finish`; the first call does it.
pub(crate) fn insert(app: &mut App) {
    if app.world().contains_resource::<CranposeGpu>() {
        return;
    }
    let Some(render) = app.get_sub_app(RenderApp) else {
        return;
    };
    let world = render.world();
    let (Some(instance), Some(adapter), Some(device), Some(queue)) = (
        world.get_resource::<RenderInstance>(),
        world.get_resource::<RenderAdapter>(),
        world.get_resource::<RenderDevice>(),
        world.get_resource::<RenderQueue>(),
    ) else {
        return;
    };
    let gpu = GpuContext {
        instance: (**instance).clone(),
        adapter: (**adapter).clone(),
        device: Arc::new(device.wgpu_device().clone()),
        queue: Arc::new((**queue).clone()),
    };
    app.insert_resource(CranposeGpu(gpu));
}
