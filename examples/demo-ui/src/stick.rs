use std::{cell::Cell, rc::Rc};

use cranpose::prelude::*;

use crate::Controls;

const SIZE: f32 = 128.0;
const KNOB: f32 = 54.0;
// The knob's center moves this far from the stick's center at most.
const TRAVEL: f32 = (SIZE - KNOB) / 2.0;
// A push shorter than this, as a part of the travel, does not walk.
const DEAD: f32 = 0.12;

/// A stick for a thumb: pushing it up walks forward, pushing it sideways
/// turns, and pushing it to the edge runs. It writes `controls.walk.stick`.
#[composable]
pub fn TouchStick(modifier: Modifier, controls: MutableState<Controls>) {
    let finger = remember(|| Rc::new(Cell::new(None::<u64>))).with(Rc::clone);
    Box(
        modifier
            .size_points(SIZE, SIZE)
            .pointer_input((), move |scope| {
                let finger = Rc::clone(&finger);
                async move {
                    scope
                        .await_pointer_event_scope(|events| async move {
                            loop {
                                let event = events.await_pointer_event().await;
                                track(&finger, &event, controls);
                            }
                        })
                        .await;
                }
            })
            .draw_behind(move |scope| {
                let [x, y] = controls.value().walk.stick;
                let center = Point::new(SIZE / 2.0, SIZE / 2.0);
                scope.draw_circle(
                    Brush::solid(Color::from_rgba_u8(14, 18, 30, 120)),
                    center,
                    SIZE / 2.0,
                );
                scope.draw_circle(
                    Brush::solid(Color::from_rgba_u8(255, 255, 255, 46)),
                    center,
                    SIZE / 2.0 - 2.0,
                );
                scope.draw_circle(
                    Brush::solid(Color::from_rgba_u8(255, 255, 255, 200)),
                    Point::new(center.x + x * TRAVEL, center.y - y * TRAVEL),
                    KNOB / 2.0,
                );
            }),
        BoxSpec::default(),
        || {},
    );
}

// Follows the finger that pressed the stick until it lifts.
fn track(finger: &Cell<Option<u64>>, event: &PointerEvent, controls: MutableState<Controls>) {
    let mine = finger.get() == Some(event.id);
    let stick = match event.kind {
        PointerEventKind::Down if finger.get().is_none() => {
            finger.set(Some(event.id));
            push(event.position)
        }
        PointerEventKind::Move if mine => push(event.position),
        PointerEventKind::Up | PointerEventKind::Cancel if mine => {
            finger.set(None);
            [0.0, 0.0]
        }
        _ => return,
    };
    event.consume();
    if controls.value().walk.stick != stick {
        controls.update(|c| c.walk.stick = stick);
    }
}

// The stick's value for a finger at `at`, in the stick's own coordinates:
// `x` to the right, `y` up, each in -1..=1 and at most 1 long together.
fn push(at: Point) -> [f32; 2] {
    let (x, y) = ((at.x - SIZE / 2.0) / TRAVEL, (SIZE / 2.0 - at.y) / TRAVEL);
    let length = x.hypot(y);
    if length < DEAD {
        return [0.0, 0.0];
    }
    let scale = length.min(1.0) / length;
    [x * scale, y * scale]
}
