//! A Cranpose app with a Bevy scene in its layout. This file is the
//! interface; `scene.rs` is Bevy's side, which runs on a thread of its own.

mod scene;

use cranpose::{liquid::prelude::*, prelude::*, text::TextUnit, *};
use cranpose_bevy::{BevyScene, rememberBevyView};

fn main() -> Result<(), LaunchError> {
    AppLauncher::new()
        .with_title("Cranpose app with Bevy")
        .with_size(1000, 680)
        .with_gpu_requirements(cranpose_bevy::gpu_requirements())
        .with_developer_inspector(false)
        .try_run(Root)
}

#[composable]
fn Root() {
    let bevy = rememberBevyView(scene::build);
    let speed = rememberMutableStateOf(|| 0.25f32);
    let turns = speed.value();
    if let Some(bevy) = bevy.clone() {
        LaunchedEffect(turns, move |_| {
            bevy.send(move |world| world.resource_mut::<scene::Spin>().turns_per_second = turns);
        });
    }
    let theme = LiquidThemeSpec {
        scheme: SchemeMode::Dark,
        ..LiquidThemeSpec::default()
    };
    LiquidTheme(theme, move || {
        let bevy = bevy.clone();
        Row(
            Modifier::empty()
                .fill_max_size()
                .background(Color::from_rgb_u8(12, 14, 22))
                .padding(20.0),
            RowSpec::default().horizontal_arrangement(LinearArrangement::SpacedBy(20.0)),
            move || {
                Controls(speed);
                match bevy.clone() {
                    Some(bevy) => BevyScene(Modifier::empty().weight(1.0).fill_max_height(), bevy),
                    None => Label("This platform does not share its GPU device with Bevy yet."),
                }
            },
        );
    });
}

#[composable]
fn Controls(speed: MutableState<f32>) {
    LiquidCard(
        Modifier::empty().width(260.0).fill_max_height(),
        move || {
            Column(
                Modifier::empty().fill_max_width().padding(20.0),
                ColumnSpec::default().vertical_arrangement(LinearArrangement::SpacedBy(12.0)),
                move || {
                    Label(format!("Spin: {:.2} turns per second", speed.value()));
                    LiquidSlider(
                        Modifier::empty().fill_max_width(),
                        speed.value(),
                        move |value| speed.set(value),
                    );
                },
            );
        },
    );
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
