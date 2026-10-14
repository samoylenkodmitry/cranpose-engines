use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};

use cranpose::{liquid::prelude::*, prelude::*};

use crate::{Controls, MUTED, SceneSliders, TouchStick, text_style, walk_hint};

#[derive(Clone, Copy, PartialEq)]
pub struct EngineInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub features: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
pub struct WindowStates {
    pub controls: MutableState<Controls>,
    pub frame: MutableState<Option<ImageBitmap>>,
    pub fps: MutableState<u32>,
    /// A finger touched the window, so it shows the touch stick and hints.
    pub touch: MutableState<bool>,
}

#[derive(Clone, Default)]
pub struct TargetSize(pub Rc<Cell<(u32, u32)>>);

impl PartialEq for TargetSize {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// The window's states. `touch` says whether the screen is a touch screen
/// before the first touch tells.
#[composable]
pub fn rememberWindowStates(touch: bool) -> WindowStates {
    remember(|| WindowStates {
        controls: mutableStateOf(Controls::default()),
        frame: mutableStateOf(None),
        fps: mutableStateOf(0),
        touch: mutableStateOf(touch),
    })
    .with(|states| *states)
}

#[derive(Clone, Default)]
pub struct WindowBridge {
    pub states: Rc<OnceCell<WindowStates>>,
    pub size: TargetSize,
    pub pointer: ScenePointers,
    /// The screen is a touch screen, so the window shows the touch stick
    /// before the first touch.
    pub touch: bool,
}

impl PartialEq for WindowBridge {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.states, &other.states)
            && self.size == other.size
            && self.pointer == other.pointer
            && self.touch == other.touch
    }
}

/// A pointer sample on the engine's view, in the pixels of its frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScenePointer {
    Moved([f32; 2]),
    Pressed([f32; 2]),
    Released([f32; 2]),
}

/// The pointer samples that landed on the engine's view, for the host to
/// hand its world, and whether something in that world holds the press: the
/// view then leaves the drag to it rather than look around.
#[derive(Clone, Default)]
pub struct ScenePointers {
    samples: Rc<RefCell<Vec<ScenePointer>>>,
    held: Rc<Cell<bool>>,
}

impl PartialEq for ScenePointers {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.samples, &other.samples)
    }
}

impl ScenePointers {
    /// Hands each sample since the last call to `deliver`, which returns
    /// whether the world holds the press after it.
    pub fn deliver(&self, mut deliver: impl FnMut(ScenePointer) -> bool) {
        for sample in self.samples.borrow_mut().drain(..) {
            self.held.set(deliver(sample));
        }
    }
}

#[composable]
pub fn HostedEngineWindow(engine: EngineInfo, bridge: WindowBridge) {
    let states = rememberWindowStates(bridge.touch);
    bridge.states.get_or_init(|| states);
    let size = bridge.size.clone();
    let pointer = bridge.pointer.clone();
    EngineWindow(engine, states, move || {
        FrameView(states.frame, size.clone(), pointer.clone())
    });
}

/// Shows the frames a host hands over in `frame`, reports the size in
/// physical pixels the host should render at, and passes it the pointer.
#[composable]
fn FrameView(
    frame: MutableState<Option<ImageBitmap>>,
    target_size: TargetSize,
    pointer: ScenePointers,
) {
    // Pixels per dp, which the drawing learns and the pointer needs.
    let density = remember(|| Rc::new(Cell::new(1.0f32))).with(Rc::clone);
    let drawn = Rc::clone(&density);
    Box(
        Modifier::empty()
            .fill_max_size()
            .pointer_input((), move |scope| {
                let pointer = pointer.clone();
                let density = Rc::clone(&density);
                async move {
                    scope
                        .await_pointer_event_scope(|events| async move {
                            loop {
                                let event = events.await_pointer_event().await;
                                forward(&pointer, density.get(), &event);
                            }
                        })
                        .await;
                }
            })
            .draw_behind(move |scope| {
                let size = scope.size();
                let density = current_density();
                drawn.set(density);
                target_size.0.set((
                    (size.width * density).round() as u32,
                    (size.height * density).round() as u32,
                ));
                if let Some(image) = frame.value() {
                    scope.draw_image_at_sampled(
                        Rect::from_size(size),
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

// Queues `event` for the host in physical pixels, and consumes a move while
// the host's world holds the press.
fn forward(pointer: &ScenePointers, density: f32, event: &PointerEvent) {
    let at = [event.position.x * density, event.position.y * density];
    let sample = match event.kind {
        PointerEventKind::Down => ScenePointer::Pressed(at),
        PointerEventKind::Move => ScenePointer::Moved(at),
        PointerEventKind::Up | PointerEventKind::Cancel => ScenePointer::Released(at),
        _ => return,
    };
    pointer.samples.borrow_mut().push(sample);
    if event.kind == PointerEventKind::Move && pointer.held.get() {
        event.consume();
    }
}

// Narrower than this, the controls go under the engine's view.
const NARROW: f32 = 760.0;
// Lower than this, the control panel leaves out the description and hint.
const ROOMY: f32 = 600.0;

/// The demo window: the scene controls beside `scene`, the engine's view,
/// which fills its rounded frame under the glass. On a narrow screen, such as
/// a phone held upright, the controls go under the view.
#[composable]
pub fn EngineWindow(engine: EngineInfo, states: WindowStates, scene: impl FnMut() + 'static) {
    let scene = Rc::new(RefCell::new(scene));
    LiquidTheme(crate::dark_theme(), move || {
        let scene = Rc::clone(&scene);
        BoxWithConstraints(
            Modifier::empty()
                .fill_max_size()
                .background(Color::from_rgb_u8(12, 14, 22)),
            move |scope| {
                let space = scope.constraints();
                let scene = Rc::clone(&scene);
                let show = move || (scene.borrow_mut())();
                if space.max_width < NARROW {
                    Column(
                        Modifier::empty().fill_max_size().padding(12.0),
                        ColumnSpec::default()
                            .vertical_arrangement(LinearArrangement::SpacedBy(12.0)),
                        move || {
                            Viewport(
                                Modifier::empty().weight(1.0).fill_max_width(),
                                engine,
                                states,
                                true,
                                show.clone(),
                            );
                            ControlPanel(Modifier::empty().fill_max_width(), engine, states, false);
                        },
                    );
                } else {
                    let roomy = space.max_height >= ROOMY;
                    Row(
                        Modifier::empty().fill_max_size().padding(20.0),
                        RowSpec::default()
                            .horizontal_arrangement(LinearArrangement::SpacedBy(20.0)),
                        move || {
                            ControlPanel(
                                Modifier::empty().width(300.0).fill_max_height(),
                                engine,
                                states,
                                roomy,
                            );
                            Viewport(
                                Modifier::empty().weight(1.0).fill_max_height(),
                                engine,
                                states,
                                false,
                                show.clone(),
                            );
                        },
                    );
                }
            },
        );
    });
}

/// The engine's name, the scene's sliders and the frame rate. A `roomy`
/// panel fills its height and adds the description and the walking hint.
#[composable]
fn ControlPanel(modifier: Modifier, engine: EngineInfo, states: WindowStates, roomy: bool) {
    LiquidCard(modifier, move || {
        let body = if roomy {
            Modifier::empty().fill_max_size().padding(20.0)
        } else {
            Modifier::empty().fill_max_width().padding(16.0)
        };
        Column(
            body,
            ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(if roomy {
                14.0
            } else {
                10.0
            })),
            move || {
                Text(
                    format!("{} inside Cranpose", engine.name),
                    Modifier::empty(),
                    text_style(if roomy { 22.0 } else { 18.0 }, Color::WHITE),
                );
                if roomy {
                    Text(
                        engine.description,
                        Modifier::empty(),
                        text_style(13.0, MUTED),
                    );
                }
                SceneSliders(states.controls, !roomy);
                if roomy {
                    Spacer(Modifier::empty().weight(1.0));
                }
                Text(
                    format!("{} frames: {} per second", engine.name, states.fps.value()),
                    Modifier::empty(),
                    text_style(13.0, Color::from_rgb_u8(150, 220, 170)),
                );
                if roomy {
                    let hint = walk_hint(states.touch.value());
                    Text(
                        if states.touch.value() {
                            hint.to_string()
                        } else {
                            format!("Click the scene first. {hint}")
                        },
                        Modifier::empty(),
                        text_style(13.0, MUTED),
                    );
                }
            },
        );
    });
}

/// The engine's view with glass over it: a caption, the button back to the
/// start and, on a touch screen, the stick. `narrow` moves them for a phone.
#[composable]
fn Viewport(
    modifier: Modifier,
    engine: EngineInfo,
    states: WindowStates,
    narrow: bool,
    scene: impl FnMut() + 'static,
) {
    let controls = states.controls;
    let shape = RoundedCornerShape::uniform(if narrow { 22.0 } else { 28.0 });
    let last = remember(|| Rc::new(Cell::new(None::<(u64, Point)>))).with(Rc::clone);
    let focus = remember(FocusRequester::new).with(Clone::clone);
    let press_focus = focus.clone();
    let inset = if narrow { 12.0 } else { 20.0 };
    Box(
        modifier
            .shadow_with(
                18.0,
                LayerShape::Rounded(shape),
                true,
                Color::BLACK,
                Color::BLACK,
            )
            .focus_requester(&focus)
            .focus_target()
            .on_key_event(move |event| walk_key(controls, event))
            .pointer_input((), move |scope| {
                let last = Rc::clone(&last);
                let focus = press_focus.clone();
                async move {
                    scope
                        .await_pointer_event_scope(|events| async move {
                            loop {
                                let event = events.await_pointer_event().await;
                                look(&last, &focus, &event, states);
                            }
                        })
                        .await;
                }
            }),
        BoxSpec::default(),
        move || {
            scene();
            GlassSurface(
                Modifier::empty()
                    .align(if narrow {
                        Alignment::TOP_START
                    } else {
                        Alignment::BOTTOM_START
                    })
                    .padding(inset)
                    .width_in(0.0, 380.0),
                Glass::regular(),
                move || {
                    Column(
                        Modifier::empty().padding_symmetric(if narrow { 14.0 } else { 18.0 }, 12.0),
                        ColumnSpec::default()
                            .vertical_arrangement(LinearArrangement::SpacedBy(4.0)),
                        move || {
                            Text(
                                format!("Glass over a {} frame", engine.name),
                                Modifier::empty(),
                                text_style(if narrow { 15.0 } else { 17.0 }, Color::WHITE),
                            );
                            Text(
                                engine.features,
                                Modifier::empty(),
                                text_style(
                                    if narrow { 12.0 } else { 13.0 },
                                    Color::from_rgb_u8(225, 230, 240),
                                ),
                            );
                        },
                    );
                },
            );
            GlassButton(
                Modifier::empty()
                    .align(if narrow {
                        Alignment::BOTTOM_END
                    } else {
                        Alignment::TOP_END
                    })
                    .padding(inset),
                GlassButtonSpec::glass(),
                move || controls.update(Controls::back_to_start),
                || {
                    Text(
                        "Back to start",
                        Modifier::empty(),
                        text_style(15.0, Color::WHITE),
                    );
                },
            );
            if states.touch.value() {
                TouchStick(
                    Modifier::empty()
                        .align(if narrow {
                            Alignment::BOTTOM_START
                        } else {
                            Alignment::BOTTOM_END
                        })
                        .padding(inset),
                    controls,
                );
            }
        },
    );
}

// Dragging the view looks around. The finger or button that pressed the view
// looks until it lifts; a touch also turns on the touch stick.
fn look(
    last: &Cell<Option<(u64, Point)>>,
    focus: &FocusRequester,
    event: &PointerEvent,
    states: WindowStates,
) {
    if event.source == PointerSource::Touch && !states.touch.value() {
        states.touch.set(true);
    }
    match (event.kind, last.get()) {
        (PointerEventKind::Down, None) if !event.is_consumed() => {
            let _ = focus.request_focus();
            last.set(Some((event.id, event.position)));
        }
        (PointerEventKind::Move, Some((id, previous))) if id == event.id => {
            states
                .controls
                .update(|c| c.look(event.position.x - previous.x, event.position.y - previous.y));
            last.set(Some((id, event.position)));
            event.consume();
        }
        (PointerEventKind::Up | PointerEventKind::Cancel, Some((id, _))) if id == event.id => {
            last.set(None)
        }
        _ => {}
    }
}

fn walk_key(controls: MutableState<Controls>, event: &KeyEvent) -> bool {
    let mut walk = controls.value().walk;
    let handled = walk.key(event.key_code, event.event_type == KeyEventType::KeyDown);
    if handled {
        controls.update(|c| c.walk = walk);
    }
    handled
}
