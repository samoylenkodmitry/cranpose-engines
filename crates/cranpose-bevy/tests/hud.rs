mod support;

use std::sync::{Arc, Mutex};

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyCode, KeyboardInput},
        touch::{TouchInput, TouchPhase},
    },
    math::Vec2,
    prelude::{App, Entity, World},
};
use cranpose::prelude::*;
use cranpose_bevy::{
    CranposeInput,
    hud::{CranposeHudPlugin, HudCamera},
};
use support::{BLUE, HEIGHT, Pixels, RED, WIDTH, shows};

/// A headless Bevy app whose camera renders a blue image with `hud` over it.
/// The image is read back into the returned cell every frame.
fn hud_app(hud: impl Fn(&mut World) -> Box<dyn FnMut()> + Send + Sync + 'static) -> (App, Pixels) {
    let mut app = support::app(false);
    app.add_plugins(CranposeHudPlugin::new(hud));
    let read = support::image_camera(&mut app, HudCamera);
    support::start(&mut app);
    (app, read)
}

#[test]
fn the_hud_draws_over_the_image_of_its_camera() {
    let (mut app, read) = hud_app(|_| {
        Box::new(|| {
            Box(
                Modifier::empty()
                    .size(Size::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0))
                    .background(Color::from_rgb_u8(255, 0, 0)),
                BoxSpec::default(),
                || {},
            );
        })
    });
    assert!(
        shows(
            &mut app,
            &read,
            &[((8, 8), RED), ((WIDTH - 8, HEIGHT - 8), BLUE)]
        ),
        "the HUD never showed over the camera's image"
    );
}

#[test]
fn keys_type_into_a_focused_hud_field_and_the_game_learns_the_hud_took_them() {
    let typed: Arc<Mutex<String>> = Arc::default();
    let sink = Arc::clone(&typed);
    let (mut app, _) = hud_app(move |_| {
        let sink = Arc::clone(&sink);
        Box::new(move || {
            let focus = remember(FocusRequester::new).with(Clone::clone);
            let state = remember(|| TextFieldState::new("")).with(|state| *state);
            BasicTextField(
                state,
                Modifier::empty()
                    .size(Size::new(40.0, 20.0))
                    .focus_requester(&focus),
                TextStyle::default(),
            );
            LaunchedEffect((), move |_| {
                let _ = focus.request_focus();
            });
            if let Ok(mut typed) = sink.lock() {
                *typed = state.text();
            }
        })
    });
    for _ in 0..4 {
        app.update();
    }
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::KeyH,
            logical_key: Key::Character("h".into()),
            state,
            text: Some("h".into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
    app.update();
    assert_eq!(typed.lock().expect("typed").as_str(), "h");
    assert!(app.world().resource::<CranposeInput>().keyboard);
}

#[test]
fn a_finger_on_the_hud_stays_the_huds_and_other_fingers_stay_the_games() {
    let clicks: Arc<Mutex<u32>> = Arc::default();
    let sink = Arc::clone(&clicks);
    let (mut app, _) = hud_app(move |_| {
        let sink = Arc::clone(&sink);
        Box::new(move || {
            let sink = Arc::clone(&sink);
            Box(
                Modifier::empty()
                    .size(Size::new(24.0, 24.0))
                    .clickable(move |_| {
                        if let Ok(mut clicks) = sink.lock() {
                            *clicks += 1;
                        }
                    }),
                BoxSpec::default(),
                || {},
            );
        })
    });
    for _ in 0..4 {
        app.update();
    }
    let mut touch = |id: u64, phase: TouchPhase, x: f32, y: f32| {
        app.world_mut().write_message(TouchInput {
            phase,
            position: Vec2::new(x, y),
            window: Entity::PLACEHOLDER,
            force: None,
            id,
        });
        app.update();
        let input = app.world().resource::<CranposeInput>();
        [1, 2, 3].map(|finger| input.took_finger(finger))
    };
    let first = [true, false, false];
    let none = [false; 3];
    assert_eq!(touch(1, TouchPhase::Started, 10.0, 10.0), first);
    assert_eq!(touch(2, TouchPhase::Started, 50.0, 40.0), first);
    assert_eq!(touch(2, TouchPhase::Moved, 40.0, 30.0), first);
    assert_eq!(touch(1, TouchPhase::Ended, 10.0, 10.0), none);
    assert_eq!(touch(2, TouchPhase::Ended, 40.0, 30.0), none);
    assert_eq!(touch(3, TouchPhase::Started, 50.0, 40.0), none);
    assert_eq!(touch(3, TouchPhase::Ended, 50.0, 40.0), none);
    assert_eq!(*clicks.lock().expect("clicks"), 1);
}

#[test]
fn a_tap_that_hides_a_panel_shows_the_hud_without_it() {
    let clicks: Arc<Mutex<u32>> = Arc::default();
    let sink = Arc::clone(&clicks);
    let (mut app, read) = hud_app(move |_| {
        let sink = Arc::clone(&sink);
        Box::new(move || {
            let shown = rememberMutableStateOf(|| true);
            let sink = Arc::clone(&sink);
            Column(
                Modifier::empty().fill_max_size(),
                ColumnSpec::default(),
                move || {
                    if shown.value() {
                        Box(
                            Modifier::empty()
                                .size(Size::new(WIDTH as f32, 16.0))
                                .background(Color::from_rgb_u8(255, 0, 0)),
                            BoxSpec::default(),
                            || {},
                        );
                    }
                    Spacer(Modifier::empty().weight(1.0));
                    let sink = Arc::clone(&sink);
                    Box(
                        Modifier::empty()
                            .size(Size::new(WIDTH as f32, 16.0))
                            .clickable(move |_| {
                                if let Ok(mut clicks) = sink.lock() {
                                    *clicks += 1;
                                }
                                shown.set(!shown.value());
                            }),
                        BoxSpec::default(),
                        || {},
                    );
                },
            );
        })
    });
    assert!(
        shows(&mut app, &read, &[((8, 4), RED)]),
        "the panel shows first"
    );
    for phase in [TouchPhase::Started, TouchPhase::Ended] {
        app.world_mut().write_message(TouchInput {
            phase,
            position: Vec2::new(8.0, HEIGHT as f32 - 8.0),
            window: Entity::PLACEHOLDER,
            force: None,
            id: 7,
        });
    }
    let hidden = shows(&mut app, &read, &[((8, 4), BLUE)]);
    let clicks = *clicks.lock().expect("clicks");
    assert!(
        hidden,
        "the panel stayed for 600 frames after {clicks} click(s)"
    );
    assert_eq!(clicks, 1);
}
