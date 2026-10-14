use std::{cell::Cell, rc::Rc};

use bevy::{app::App, camera::RenderTarget, input::touch::TouchPhase, log::error, math::Vec2};
use cranpose::{FrameEffect, gpu_context, prelude::*};

use crate::{BevyView, ViewPointer};

// The size of the frames asked of Bevy, and their pixels per dp.
#[derive(Clone, Copy, Default)]
struct Frames {
    width: u32,
    height: u32,
    density: f32,
}

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
/// for frames of that size in physical pixels, or in
/// [`BevyView::max_density`] pixels per dp where the screen has more. Bevy
/// runs one update for each frame this draws; the newest frame Bevy finished
/// is shown.
///
/// The pointer and fingers that land here go to Bevy's world too, through
/// [`BevyView::pointer`]. While a Cranpose screen in that world holds the
/// press, this consumes the moves, so a drag the composition around it
/// follows, such as one that looks around, leaves them alone.
#[composable]
pub fn BevyScene(modifier: Modifier, view: Rc<BevyView>) {
    let frame = rememberMutableStateOf(|| None::<ImageBitmap>);
    let frames = remember(|| Rc::new(Cell::new(Frames::default()))).with(Rc::clone);
    let wanted = Rc::clone(&frames);
    let pointed = Rc::clone(&frames);
    let limited = Rc::clone(&view);
    let pointer = Rc::clone(&view);
    FrameEffect((), true, move |_| {
        let Frames { width, height, .. } = wanted.get();
        if width > 0
            && height > 0
            && let Some(next) = view.show(width, height)
        {
            frame.set(Some(next));
        }
    });
    Box(
        modifier
            .pointer_input((), move |scope| {
                let view = Rc::clone(&pointer);
                let frames = Rc::clone(&pointed);
                async move {
                    scope
                        .await_pointer_event_scope(|events| async move {
                            loop {
                                let event = events.await_pointer_event().await;
                                forward(&view, frames.get().density, &event);
                            }
                        })
                        .await;
                }
            })
            .draw_behind(move |scope| {
                let bounds = scope.size();
                let density = current_density().min(limited.max_density());
                frames.set(Frames {
                    width: (bounds.width * density).round() as u32,
                    height: (bounds.height * density).round() as u32,
                    density,
                });
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

// Hands `event` to Bevy's world in the pixels of its frames, and consumes a
// move while a Cranpose screen there holds the press.
fn forward(view: &BevyView, density: f32, event: &PointerEvent) {
    let at = Vec2::new(event.position.x, event.position.y) * density;
    let touch = |phase| ViewPointer::Touch {
        finger: event.id,
        phase,
        at,
    };
    let finger = event.source == PointerSource::Touch;
    let sample = match (event.kind, finger) {
        (PointerEventKind::Down, true) => touch(TouchPhase::Started),
        (PointerEventKind::Move, true) => touch(TouchPhase::Moved),
        (PointerEventKind::Up, true) => touch(TouchPhase::Ended),
        (PointerEventKind::Cancel, true) => touch(TouchPhase::Canceled),
        (PointerEventKind::Down, false) => ViewPointer::Pressed(at),
        (PointerEventKind::Move, false) => ViewPointer::Moved(at),
        (PointerEventKind::Up | PointerEventKind::Cancel, false) => ViewPointer::Released(at),
        _ => return,
    };
    view.pointer(sample);
    if event.kind == PointerEventKind::Move && view.holds_press() {
        event.consume();
    }
}
