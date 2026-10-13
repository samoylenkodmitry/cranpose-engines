use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};

use cranpose::{liquid::LiquidTheme, prelude::*};

use crate::{Controls, MUTED, SceneSliders, TouchStick, text_style, walk_hint};

pub enum HudCommand {
    BackToStart,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Stats {
    pub fps: u32,
    pub altitude: f32,
    pub stamina: f32,
    pub size: (u32, u32),
}

#[derive(Clone, Copy)]
pub struct HudStates {
    pub controls: MutableState<Controls>,
    pub stats: MutableState<Stats>,
    /// A finger touched the game, so the HUD shows the touch stick.
    pub touch: MutableState<bool>,
}

#[derive(Clone)]
pub struct HudBridge {
    pub engine: &'static str,
    /// Whether the screen is a touch screen before the first touch tells.
    pub touch: bool,
    pub states: Rc<OnceCell<HudStates>>,
    pub commands: Rc<RefCell<Vec<HudCommand>>>,
}

impl HudBridge {
    pub fn new(engine: &'static str, touch: bool) -> Self {
        Self {
            engine,
            touch,
            states: Rc::default(),
            commands: Rc::default(),
        }
    }
}

impl PartialEq for HudBridge {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.states, &other.states) && Rc::ptr_eq(&self.commands, &other.commands)
    }
}

const PANEL: Color = Color::from_rgba_u8(14, 18, 30, 170);
fn panel(modifier: Modifier) -> Modifier {
    modifier
        .shadow_with(
            14.0,
            LayerShape::Rounded(RoundedCornerShape::uniform(20.0)),
            true,
            Color::BLACK,
            Color::BLACK,
        )
        .background(PANEL)
        .rounded_corners(20.0)
}

// Narrower or lower than this, the HUD folds into its phone layout.
const ROOMY: (f32, f32) = (760.0, 600.0);

/// The HUD: the walker's status, the frame rate, the scene's sliders, the
/// button back to the start and, on a touch screen, the stick. On a phone the
/// status takes the top, and the sliders fold away behind a button.
#[composable]
pub fn Hud(bridge: HudBridge) {
    let controls = rememberMutableStateOf(Controls::default);
    let stats = rememberMutableStateOf(Stats::default);
    let touch = rememberMutableStateOf(|| bridge.touch);
    let settings = rememberMutableStateOf(|| true);
    bridge.states.get_or_init(|| HudStates {
        controls,
        stats,
        touch,
    });
    LiquidTheme(crate::dark_theme(), move || {
        let bridge = bridge.clone();
        BoxWithConstraints(Modifier::empty().fill_max_size(), move |scope| {
            let space = scope.constraints();
            let roomy = space.max_width >= ROOMY.0 && space.max_height >= ROOMY.1;
            let bridge = bridge.clone();
            Box(
                Modifier::empty()
                    .fill_max_size()
                    .padding(if roomy { 20.0 } else { 12.0 }),
                BoxSpec::default(),
                move || {
                    StatusPanel(bridge.engine, stats, touch, roomy);
                    if roomy {
                        FrameChip(stats);
                        SettingsPanel(
                            Modifier::empty().align(Alignment::CENTER_END).width(280.0),
                            controls,
                            false,
                        );
                        ActionBar(bridge.clone(), touch);
                    } else {
                        let upright = space.max_height > space.max_width;
                        CompactActions(bridge.clone(), controls, settings, touch, upright);
                    }
                    if roomy && touch.value() {
                        TouchStick(Modifier::empty().align(Alignment::BOTTOM_START), controls);
                    }
                },
            );
        });
    });
}

#[composable]
fn StatusPanel(
    engine: &'static str,
    stats: MutableState<Stats>,
    touch: MutableState<bool>,
    roomy: bool,
) {
    let width = if roomy {
        Modifier::empty().width(300.0)
    } else {
        Modifier::empty().width_in(0.0, 340.0)
    };
    Column(
        panel(width.align(Alignment::TOP_START)).padding(if roomy { 18.0 } else { 14.0 }),
        ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(if roomy {
            10.0
        } else {
            8.0
        })),
        move || {
            Row(
                Modifier::empty(),
                RowSpec::default()
                    .horizontal_arrangement(LinearArrangement::SpacedBy(12.0))
                    .vertical_alignment(VerticalAlignment::CenterVertically),
                move || {
                    Text(
                        "Cranpose HUD",
                        Modifier::empty(),
                        text_style(if roomy { 20.0 } else { 17.0 }, Color::WHITE),
                    );
                    if !roomy {
                        Text(
                            format!("{} fps", stats.value().fps),
                            Modifier::empty(),
                            text_style(14.0, Color::from_rgb_u8(150, 230, 170)),
                        );
                    }
                },
            );
            if roomy {
                Text(
                    format!(
                        "Drawn by Cranpose on {engine}'s GPU device, over {engine}'s 3D frame."
                    ),
                    Modifier::empty(),
                    text_style(12.0, MUTED),
                );
            }
            Meter(
                "Stamina",
                stats.value().stamina,
                [
                    Color::from_rgb_u8(255, 94, 98),
                    Color::from_rgb_u8(255, 196, 92),
                ],
            );
            Text(
                format!("Altitude {:.0} m", stats.value().altitude),
                Modifier::empty(),
                text_style(13.0, MUTED),
            );
            if !roomy {
                Text(
                    walk_hint(touch.value()),
                    Modifier::empty(),
                    text_style(12.0, MUTED),
                );
            }
        },
    );
}

#[composable]
fn Meter(label: &'static str, fraction: f32, colors: [Color; 2]) {
    Column(
        Modifier::empty().fill_max_width(),
        ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(4.0)),
        move || {
            Text(label, Modifier::empty(), text_style(13.0, Color::WHITE));
            Box(
                Modifier::empty()
                    .fill_max_width()
                    .height(10.0)
                    .draw_behind(move |scope| {
                        let size = scope.size();
                        let radii = CornerRadii::uniform(size.height / 2.0);
                        scope.draw_round_rect(
                            Brush::solid(Color::from_rgba_u8(255, 255, 255, 40)),
                            radii,
                        );
                        scope.draw_round_rect_at(
                            Rect {
                                x: 0.0,
                                y: 0.0,
                                width: size.width * fraction.clamp(0.02, 1.0),
                                height: size.height,
                            },
                            Brush::horizontal_gradient(colors.to_vec(), 0.0, size.width),
                            radii,
                        );
                    }),
                BoxSpec::default(),
                || {},
            );
        },
    );
}

#[composable]
fn FrameChip(stats: MutableState<Stats>) {
    Row(
        panel(Modifier::empty().align(Alignment::TOP_END)).padding_symmetric(16.0, 10.0),
        RowSpec::default().horizontal_arrangement(LinearArrangement::SpacedBy(12.0)),
        move || {
            let stats = stats.value();
            Text(
                format!("{} fps", stats.fps),
                Modifier::empty(),
                text_style(16.0, Color::from_rgb_u8(150, 230, 170)),
            );
            Text(
                format!("{} x {}", stats.size.0, stats.size.1),
                Modifier::empty(),
                text_style(16.0, MUTED),
            );
        },
    );
}

#[composable]
fn SettingsPanel(modifier: Modifier, controls: MutableState<Controls>, compact: bool) {
    Column(
        panel(modifier).padding(if compact { 14.0 } else { 18.0 }),
        ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(10.0)),
        move || {
            Text("Scene", Modifier::empty(), text_style(18.0, Color::WHITE));
            SceneSliders(controls, compact);
        },
    );
}

#[composable]
fn ActionBar(bridge: HudBridge, touch: MutableState<bool>) {
    Row(
        panel(Modifier::empty().align(Alignment::BOTTOM_CENTER)).padding_symmetric(16.0, 12.0),
        RowSpec::default()
            .horizontal_arrangement(LinearArrangement::SpacedBy(14.0))
            .vertical_alignment(VerticalAlignment::CenterVertically),
        move || {
            BackToStart(bridge.clone());
            Text(
                walk_hint(touch.value()),
                Modifier::empty(),
                text_style(14.0, MUTED),
            );
        },
    );
}

// The phone layout's bottom: the scene's sliders, which fold away, the
// buttons and, on a touch screen, the stick. A phone held upright stacks the
// sliders over the stick and the buttons; one held sideways keeps them to the
// right, clear of the stick in the lower left corner.
#[composable]
fn CompactActions(
    bridge: HudBridge,
    controls: MutableState<Controls>,
    settings: MutableState<bool>,
    touch: MutableState<bool>,
    upright: bool,
) {
    if upright {
        Column(
            Modifier::empty()
                .align(Alignment::BOTTOM_CENTER)
                .fill_max_width(),
            ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(12.0)),
            move || {
                if settings.value() {
                    SettingsPanel(Modifier::empty().fill_max_width(), controls, true);
                }
                let bridge = bridge.clone();
                Row(
                    Modifier::empty().fill_max_width(),
                    RowSpec::default().vertical_alignment(VerticalAlignment::Bottom),
                    move || {
                        if touch.value() {
                            TouchStick(Modifier::empty(), controls);
                        }
                        Spacer(Modifier::empty().weight(1.0));
                        let bridge = bridge.clone();
                        Column(
                            Modifier::empty(),
                            ColumnSpec::default()
                                .vertical_arrangement(LinearArrangement::SpacedBy(10.0))
                                .horizontal_alignment(HorizontalAlignment::End),
                            move || {
                                SceneButton(settings);
                                BackToStart(bridge.clone());
                            },
                        );
                    },
                );
            },
        );
    } else {
        Column(
            Modifier::empty().align(Alignment::BOTTOM_END),
            ColumnSpec::default()
                .vertical_arrangement(LinearArrangement::SpacedBy(12.0))
                .horizontal_alignment(HorizontalAlignment::End),
            move || {
                if settings.value() {
                    SettingsPanel(Modifier::empty().width(300.0), controls, true);
                }
                let bridge = bridge.clone();
                Row(
                    Modifier::empty(),
                    RowSpec::default().horizontal_arrangement(LinearArrangement::SpacedBy(10.0)),
                    move || {
                        SceneButton(settings);
                        BackToStart(bridge.clone());
                    },
                );
            },
        );
        if touch.value() {
            TouchStick(Modifier::empty().align(Alignment::BOTTOM_START), controls);
        }
    }
}

#[composable]
fn SceneButton(settings: MutableState<bool>) {
    HudButton(
        if settings.value() {
            "Hide scene"
        } else {
            "Scene"
        },
        move || settings.set(!settings.value()),
    );
}

#[composable]
fn BackToStart(bridge: HudBridge) {
    let commands = Rc::clone(&bridge.commands);
    HudButton("Back to start", move || {
        commands.borrow_mut().push(HudCommand::BackToStart)
    });
}

// A button drawn as a panel. Glass needs the frame under it, which a HUD
// drawn into its own texture does not have.
#[composable]
fn HudButton(label: &'static str, on_click: impl Fn() + 'static) {
    Box(
        panel(Modifier::empty())
            .clickable(move |_| on_click())
            .padding_symmetric(18.0, 10.0),
        BoxSpec::default(),
        move || {
            Text(label, Modifier::empty(), text_style(15.0, Color::WHITE));
        },
    );
}
