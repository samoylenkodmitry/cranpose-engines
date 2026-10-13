mod hud;
mod stick;
mod window;

use cranpose::{
    liquid::{LiquidSlider, LiquidThemeSpec, SchemeMode},
    prelude::*,
    text::TextUnit,
};
pub use hud::{Hud, HudBridge, HudCommand, HudStates, Stats};
pub use stick::TouchStick;
pub use window::{
    EngineInfo, EngineWindow, HostedEngineWindow, TargetSize, WindowBridge, WindowStates,
    rememberWindowStates,
};

/// How far the stick is pushed before the walker runs.
const RUN_PUSH: f32 = 0.9;

/// The keys held and the touch stick: the stick's `x` turns, its `y` walks.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Walk {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub stick: [f32; 2],
}

impl Walk {
    pub fn forward_axis(&self) -> f32 {
        (f32::from(i8::from(self.forward) - i8::from(self.back)) + self.stick[1]).clamp(-1.0, 1.0)
    }

    pub fn right_axis(&self) -> f32 {
        f32::from(i8::from(self.right) - i8::from(self.left))
    }

    pub fn turn_axis(&self) -> f32 {
        self.stick[0]
    }

    pub fn running(&self) -> bool {
        self.run || self.stick[0].hypot(self.stick[1]) >= RUN_PUSH
    }

    pub fn key(&mut self, key: KeyCode, down: bool) -> bool {
        let held = match key {
            KeyCode::W | KeyCode::ArrowUp => &mut self.forward,
            KeyCode::S | KeyCode::ArrowDown => &mut self.back,
            KeyCode::A | KeyCode::ArrowLeft => &mut self.left,
            KeyCode::D | KeyCode::ArrowRight => &mut self.right,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => &mut self.run,
            _ => return false,
        };
        *held = down;
        true
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Controls {
    pub yaw: f32,
    pub pitch: f32,
    pub time_of_day: f32,
    pub fog: f32,
    pub bloom: f32,
    pub walk: Walk,
    pub respawn: u32,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.08,
            time_of_day: 0.3,
            fog: 0.35,
            bloom: 0.3,
            walk: Walk::default(),
            respawn: 0,
        }
    }
}

impl Controls {
    pub fn look(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.005;
        self.pitch = (self.pitch - dy * 0.004).clamp(-1.3, 1.3);
    }

    pub fn back_to_start(&mut self) {
        let start = Self::default();
        self.yaw = start.yaw;
        self.pitch = start.pitch;
        self.respawn += 1;
    }
}

/// The scene's sliders. `compact` puts each label beside its slider, for
/// phones.
#[composable]
fn SceneSliders(controls: MutableState<Controls>, compact: bool) {
    Labeled("Time of day", compact, move |modifier| {
        LiquidSlider(modifier, controls.value().time_of_day, move |value| {
            controls.update(|c| c.time_of_day = value)
        });
    });
    Labeled("Fog", compact, move |modifier| {
        LiquidSlider(modifier, controls.value().fog, move |value| {
            controls.update(|c| c.fog = value)
        });
    });
    Labeled("Bloom", compact, move |modifier| {
        LiquidSlider(modifier, controls.value().bloom / 0.8, move |value| {
            controls.update(|c| c.bloom = value * 0.8)
        });
    });
}

#[composable]
fn Labeled(label: &'static str, compact: bool, content: impl Fn(Modifier) + 'static) {
    let label_style = text_style(14.0, Color::WHITE);
    if compact {
        Row(
            Modifier::empty().fill_max_width(),
            RowSpec::default()
                .horizontal_arrangement(LinearArrangement::SpacedBy(12.0))
                .vertical_alignment(VerticalAlignment::CenterVertically),
            move || {
                Text(label, Modifier::empty().width(92.0), label_style.clone());
                content(Modifier::empty().weight(1.0));
            },
        );
    } else {
        Column(
            Modifier::empty().fill_max_width(),
            ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(4.0)),
            move || {
                Text(label, Modifier::empty(), label_style.clone());
                content(Modifier::empty().fill_max_width());
            },
        );
    }
}

/// How to walk and look, for a keyboard and mouse or for touch.
fn walk_hint(touch: bool) -> &'static str {
    if touch {
        "Drag to look. The stick walks and turns; push it to the edge to run."
    } else {
        "WASD to walk, Shift to run, drag to look."
    }
}

const MUTED: Color = Color::from_rgb_u8(200, 206, 220);

fn text_style(size: f32, color: Color) -> TextStyle {
    TextStyle::from_span_style(SpanStyle {
        color: Some(color),
        font_size: TextUnit::Sp(size),
        ..Default::default()
    })
}

fn dark_theme() -> LiquidThemeSpec {
    LiquidThemeSpec {
        scheme: SchemeMode::Dark,
        ..LiquidThemeSpec::default()
    }
}
