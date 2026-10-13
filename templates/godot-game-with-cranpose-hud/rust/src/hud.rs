//! The HUD: a Cranpose composition over the scene. The `CranposeHud` node
//! reads and writes its state through [`Hud`].

use std::{cell::OnceCell, rc::Rc};

use cranpose::{liquid::prelude::*, prelude::*, text::TextUnit};

/// The state the HUD and the node share. The composition makes it.
#[derive(Clone, Copy)]
pub struct HudState {
    /// The turns per second the player picked.
    pub speed: MutableState<f32>,
    /// The turns the target made, which the node counts.
    pub turns: MutableState<u32>,
}

/// The node's handle to the HUD's state.
#[derive(Clone, Default)]
pub struct Hud(Rc<OnceCell<HudState>>);

impl Hud {
    /// The HUD's state once it composed.
    pub fn state(&self) -> Option<HudState> {
        self.0.get().copied()
    }
}

impl PartialEq for Hud {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[composable]
pub fn Panel(hud: Hud) {
    let state = remember(|| HudState {
        speed: mutableStateOf(0.25),
        turns: mutableStateOf(0),
    })
    .with(|state| *state);
    hud.0.get_or_init(|| state);
    let theme = LiquidThemeSpec {
        scheme: SchemeMode::Dark,
        ..LiquidThemeSpec::default()
    };
    LiquidTheme(theme, move || {
        Column(
            Modifier::empty()
                .padding(20.0)
                .width(300.0)
                .shadow_with(
                    14.0,
                    LayerShape::Rounded(RoundedCornerShape::uniform(20.0)),
                    true,
                    Color::BLACK,
                    Color::BLACK,
                )
                .background(Color::from_rgba_u8(14, 18, 30, 180))
                .rounded_corners(20.0)
                .padding(18.0),
            ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(10.0)),
            move || {
                Label(format!("Turns: {}", state.turns.value()));
                Label(format!(
                    "Speed: {:.2} turns per second",
                    state.speed.value()
                ));
                LiquidSlider(
                    Modifier::empty().fill_max_width(),
                    state.speed.value(),
                    move |value| state.speed.set(value),
                );
                GlassButton(
                    Modifier::empty(),
                    GlassButtonSpec::glass(),
                    move || state.speed.set(0.0),
                    || Label("Stop"),
                );
            },
        );
    });
}

#[composable]
fn Label(text: impl Into<String>) {
    let style = TextStyle::from_span_style(SpanStyle {
        color: Some(Color::WHITE),
        font_size: TextUnit::Sp(15.0),
        ..Default::default()
    });
    Text(text.into(), Modifier::empty(), style);
}
