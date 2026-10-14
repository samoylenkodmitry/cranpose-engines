mod support;

use std::sync::{Arc, Mutex};

use bevy::{
    input::{
        ButtonState,
        mouse::{MouseButton, MouseButtonInput},
    },
    math::{Vec2, Vec3},
    prelude::{
        App, Assets, Camera3d, Commands, Entity, Mesh, Mesh3d, Rectangle, ResMut, Startup,
        Transform,
    },
    window::CursorMoved,
};
use cranpose::prelude::*;
use cranpose_bevy::{
    CranposeInput,
    screen::{CranposeScreen, CranposeScreenPlugin},
};
use support::{BLUE, HEIGHT, RED, WIDTH, shows};

// A one by one metre screen two metres in front of a camera at the origin,
// which shows `content` and takes the pointer from `reach` metres.
fn screen_in_front(
    app: &mut App,
    reach: f32,
    content: impl Fn() -> Box<dyn FnMut()> + Send + Sync + 'static,
) {
    app.add_plugins(CranposeScreenPlugin);
    let content = Arc::new(content);
    app.add_systems(
        Startup,
        move |mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>| {
            let content = Arc::clone(&content);
            commands.spawn((
                Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))),
                Transform::from_xyz(0.0, 0.0, -2.0),
                CranposeScreen::new(Vec2::new(40.0, 40.0), 1.0, move |_| content())
                    .with_reach(reach),
            ));
        },
    );
}

fn looking_ahead() -> Transform {
    Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::NEG_Z, Vec3::Y)
}

#[test]
fn a_screen_shows_its_composition_on_its_mesh() {
    let mut app = support::app(false);
    screen_in_front(&mut app, f32::INFINITY, || {
        Box::new(|| {
            Box(
                Modifier::empty()
                    .fill_max_size()
                    .background(Color::from_rgb_u8(255, 0, 0)),
                BoxSpec::default(),
                || {},
            );
        })
    });
    let read = support::image_camera(&mut app, looking_ahead());
    support::start(&mut app);
    assert!(
        shows(
            &mut app,
            &read,
            &[((WIDTH / 2, HEIGHT / 2), RED), ((1, 1), BLUE)]
        ),
        "the screen never showed red on its mesh"
    );
}

// A window camera looking at a screen that counts its clicks. Returns
// whether a press at the window's middle and one beside the screen were
// taken, and how many clicks the screen counted.
fn presses(reach: f32) -> (bool, bool, u32) {
    let clicks: Arc<Mutex<u32>> = Arc::default();
    let sink = Arc::clone(&clicks);
    let mut app = support::app(true);
    screen_in_front(&mut app, reach, move || {
        let sink = Arc::clone(&sink);
        Box::new(move || {
            let sink = Arc::clone(&sink);
            Box(
                Modifier::empty().fill_max_size().clickable(move |_| {
                    if let Ok(mut clicks) = sink.lock() {
                        *clicks += 1;
                    }
                }),
                BoxSpec::default(),
                || {},
            );
        })
    });
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Camera3d::default(), looking_ahead()));
    });
    support::start(&mut app);
    for _ in 0..4 {
        app.update();
    }
    // Presses at a point of the 1280 by 720 window and says whether a
    // Cranpose surface took the press.
    let mut press = |x: f32, y: f32| {
        app.world_mut().write_message(CursorMoved {
            window: Entity::PLACEHOLDER,
            position: Vec2::new(x, y),
            delta: None,
        });
        let button = |state| MouseButtonInput {
            button: MouseButton::Left,
            state,
            window: Entity::PLACEHOLDER,
        };
        app.world_mut().write_message(button(ButtonState::Pressed));
        app.update();
        let taken = app.world().resource::<CranposeInput>().pointer;
        app.world_mut().write_message(button(ButtonState::Released));
        app.update();
        assert!(!app.world().resource::<CranposeInput>().pointer);
        taken
    };
    let on = press(640.0, 360.0);
    let beside = press(40.0, 40.0);
    let clicks = *clicks.lock().expect("clicks");
    (on, beside, clicks)
}

#[test]
fn a_press_on_a_screen_clicks_it_and_a_press_beside_it_is_the_games() {
    assert_eq!(presses(f32::INFINITY), (true, false, 1));
}

#[test]
fn a_screen_out_of_reach_leaves_the_press_to_the_game() {
    assert_eq!(presses(1.5), (false, false, 0));
}
