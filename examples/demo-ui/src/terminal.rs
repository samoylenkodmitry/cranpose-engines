use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};

use cranpose::prelude::*;

use crate::{Controls, SceneCommand, text_style};

// The light presets: time of day at dawn, at noon and at dusk.
const LIGHTS: [(&str, f32); 3] = [("DAWN", 0.08), ("NOON", 0.5), ("DUSK", 0.92)];
const PHOSPHOR: Color = Color::from_rgb_u8(120, 255, 160);
const FAINT: Color = Color::from_rgb_u8(50, 140, 85);
const GLASS: Color = Color::from_rgb_u8(5, 16, 9);

/// The part of the scene's controls the terminal shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Light {
    pub time_of_day: f32,
    pub fog: f32,
    pub bloom: f32,
}

impl Light {
    pub fn of(controls: &Controls) -> Self {
        Self {
            time_of_day: controls.time_of_day,
            fog: controls.fog,
            bloom: controls.bloom,
        }
    }
}

/// What the terminal shows: the light, whether the walker stands near
/// enough to use it, and whether its cursor is lit.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TerminalState {
    pub light: Light,
    pub near: bool,
    pub cursor: bool,
}

/// What the game shares with the terminal: the state it shows, and the
/// commands it sends.
#[derive(Clone, Default)]
pub struct TerminalBridge {
    pub state: Rc<OnceCell<MutableState<TerminalState>>>,
    pub commands: Rc<RefCell<Vec<SceneCommand>>>,
}

impl PartialEq for TerminalBridge {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state) && Rc::ptr_eq(&self.commands, &other.commands)
    }
}

/// An old computer's green screen: the light's presets, fog and bloom
/// switches, and the way back to the start, under scan lines.
#[composable]
pub fn Terminal(bridge: TerminalBridge) {
    let state = rememberMutableStateOf(|| TerminalState {
        light: Light::of(&Controls::default()),
        ..TerminalState::default()
    });
    bridge.state.get_or_init(|| state);
    let commands = bridge.commands;
    Box(
        Modifier::empty().fill_max_size().background(GLASS),
        BoxSpec::default(),
        move || {
            let commands = Rc::clone(&commands);
            Column(
                Modifier::empty()
                    .fill_max_size()
                    .padding_symmetric(16.0, 12.0),
                ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(7.0)),
                move || {
                    let shown = state.value();
                    let light = shown.light;
                    let send = {
                        let commands = Rc::clone(&commands);
                        move |command| commands.borrow_mut().push(command)
                    };
                    Row(
                        Modifier::empty().fill_max_width(),
                        RowSpec::default(),
                        move || {
                            Text(
                                "VALLEY-OS 1.0",
                                Modifier::empty().weight(1.0),
                                text_style(15.0, PHOSPHOR),
                            );
                            Text(
                                clock(light.time_of_day),
                                Modifier::empty(),
                                text_style(15.0, PHOSPHOR),
                            );
                        },
                    );
                    Box(
                        Modifier::empty()
                            .fill_max_width()
                            .height(1.0)
                            .background(FAINT),
                        BoxSpec::default(),
                        || {},
                    );
                    let pick = send.clone();
                    Setting("LIGHT", move || {
                        let lit = nearest_light(light.time_of_day);
                        for (index, (label, time)) in LIGHTS.into_iter().enumerate() {
                            let pick = pick.clone();
                            Key(label, index == lit, move || {
                                pick(SceneCommand::TimeOfDay(time))
                            });
                        }
                    });
                    let fog = send.clone();
                    Setting("FOG", move || {
                        let on = light.fog > 0.05;
                        let fog = fog.clone();
                        Key(if on { "ON" } else { "OFF" }, on, move || {
                            fog(SceneCommand::Fog(if on { 0.0 } else { 0.45 }))
                        });
                    });
                    let bloom = send.clone();
                    Setting("BLOOM", move || {
                        let on = light.bloom > 0.02;
                        let bloom = bloom.clone();
                        Key(if on { "ON" } else { "OFF" }, on, move || {
                            bloom(SceneCommand::Bloom(if on { 0.0 } else { 0.3 }))
                        });
                    });
                    let back = send.clone();
                    Key("> RETURN TO START", false, move || {
                        back(SceneCommand::BackToStart)
                    });
                    Spacer(Modifier::empty().weight(1.0));
                    let prompt = if shown.near {
                        "READY"
                    } else {
                        "NO INPUT. WALK CLOSER."
                    };
                    let cursor = if shown.cursor { "_" } else { " " };
                    Text(
                        format!("{prompt} {cursor}"),
                        Modifier::empty(),
                        text_style(13.0, PHOSPHOR),
                    );
                },
            );
            Box(
                Modifier::empty().fill_max_size().draw_behind(|scope| {
                    let size = scope.size();
                    let line = Brush::solid(Color::from_rgba_u8(0, 0, 0, 80));
                    let mut y = 0.0;
                    while y < size.height {
                        scope.draw_rect_at(
                            Rect {
                                x: 0.0,
                                y,
                                width: size.width,
                                height: 1.0,
                            },
                            line.clone(),
                        );
                        y += 3.0;
                    }
                    scope.draw_rect(Brush::radial_gradient(
                        vec![
                            Color::from_rgba_u8(0, 0, 0, 0),
                            Color::from_rgba_u8(0, 0, 0, 170),
                        ],
                        Point::new(size.width / 2.0, size.height / 2.0),
                        size.width * 0.72,
                    ));
                }),
                BoxSpec::default(),
                || {},
            );
        },
    );
}

#[composable]
fn Setting(label: &'static str, keys: impl FnMut() + 'static) {
    Row(
        Modifier::empty().fill_max_width(),
        RowSpec::default()
            .horizontal_arrangement(LinearArrangement::SpacedBy(8.0))
            .vertical_alignment(VerticalAlignment::CenterVertically),
        move || {
            Text(
                label,
                Modifier::empty().width(64.0),
                text_style(13.0, FAINT),
            );
            keys();
        },
    );
}

// A key in inverse video while it is lit.
#[composable]
fn Key(label: &'static str, lit: bool, on_press: impl Fn() + 'static) {
    let (ink, paper) = if lit {
        (GLASS, PHOSPHOR)
    } else {
        (PHOSPHOR, Color::from_rgba_u8(0, 0, 0, 0))
    };
    Box(
        Modifier::empty()
            .background(paper)
            .border(1.0, PHOSPHOR, RoundedCornerShape::uniform(0.0))
            .clickable(move |_| on_press())
            .padding_symmetric(7.0, 2.0),
        BoxSpec::default(),
        move || {
            Text(label, Modifier::empty(), text_style(13.0, ink));
        },
    );
}

// The sun rises at 06:00 and sets at 18:00.
fn clock(time_of_day: f32) -> String {
    let minutes = (6.0 * 60.0 + time_of_day.clamp(0.0, 1.0) * 12.0 * 60.0).round() as u32;
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

fn nearest_light(time_of_day: f32) -> usize {
    LIGHTS
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (a.1 - time_of_day)
                .abs()
                .total_cmp(&(b.1 - time_of_day).abs())
        })
        .map_or(0, |(index, _)| index)
}
