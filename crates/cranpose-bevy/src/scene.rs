use std::{cell::Cell, rc::Rc};

use bevy::{app::App, camera::RenderTarget, log::error};
use cranpose::{FrameEffect, gpu_context, prelude::*};

use crate::BevyView;

/// Starts a Bevy app on its own thread the first time this call composes,
/// with Cranpose's device, and keeps it running while the call stays in the
/// composition. `None` where Cranpose shares no device, or where Bevy's
/// thread could not start.
///
/// The app needs Bevy's device features, which the launcher asks for with
/// `AppLauncher::with_gpu_requirements(cranpose_bevy::gpu_requirements())`.
#[composable]
pub fn rememberBevyView(
    setup: impl FnOnce(&mut App, RenderTarget) + Send + 'static,
) -> Option<Rc<BevyView>> {
    let gpu = gpu_context();
    remember(move || {
        let started = BevyView::new(gpu.as_ref()?, setup);
        started
            .inspect_err(|error| error!("Bevy's thread did not start: {error}"))
            .ok()
            .map(Rc::new)
    })
    .with(Option::clone)
}

/// Shows the frames of `view` in the bounds `modifier` gives, and asks Bevy
/// for frames of that size in physical pixels. Bevy runs one update for each
/// frame this draws; the newest frame Bevy finished is shown.
#[composable]
pub fn BevyScene(modifier: Modifier, view: Rc<BevyView>) {
    let frame = rememberMutableStateOf(|| None::<ImageBitmap>);
    let size = remember(|| Rc::new(Cell::new((0u32, 0u32)))).with(Rc::clone);
    let wanted = Rc::clone(&size);
    FrameEffect((), true, move |_| {
        let (width, height) = wanted.get();
        if width > 0
            && height > 0
            && let Some(next) = view.show(width, height)
        {
            frame.set(Some(next));
        }
    });
    Box(
        modifier.draw_behind(move |scope| {
            let bounds = scope.size();
            let density = current_density();
            size.set((
                (bounds.width * density).round() as u32,
                (bounds.height * density).round() as u32,
            ));
            if let Some(image) = frame.value() {
                scope.draw_image_at_sampled(
                    Rect::from_size(bounds),
                    image,
                    1.0,
                    None,
                    ImageSampling::Linear,
                );
            }
        }),
        BoxSpec::default(),
        || {},
    );
}
