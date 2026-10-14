use std::{
    cell::{Cell, OnceCell},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use bevy::{
    camera::RenderTarget,
    core_pipeline::tonemapping::Tonemapping,
    math::{Vec2, Vec3},
    prelude::{
        App, Assets, Camera3d, ClearColor, Color as BevyColor, Commands, Mesh, Mesh3d, Rectangle,
        ResMut, Startup, Transform,
    },
};
use cranpose::{AppLauncher, FrameEffect, embedded_view::EmbeddedView, gpu_context, prelude::*};
use cranpose_bevy::{
    BevyScene, BevyView, rememberBevyView,
    screen::{CranposeScreen, CranposeScreenPlugin},
};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;
const RED: [u8; 4] = [255, 0, 0, 255];

fn clear_to(color: BevyColor) -> impl FnOnce(&mut App, RenderTarget) + Send + 'static {
    move |app, target| {
        app.insert_resource(ClearColor(color));
        app.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn((Camera3d::default(), Tonemapping::None, target.clone()));
        });
    }
}

// Draws frames until `done` says so, for a minute at most.
fn draw_until(view: &mut EmbeddedView, mut done: impl FnMut(&[u8]) -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut pixels = Vec::new();
    while Instant::now() < deadline {
        view.draw(&mut pixels).expect("frame");
        if done(&pixels) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

// Draws frames until the pixel in the middle is `expected`.
fn wait_for(view: &mut EmbeddedView, expected: [u8; 4]) -> bool {
    let start = ((HEIGHT / 2 * WIDTH + WIDTH / 2) * 4) as usize;
    draw_until(view, |pixels| {
        pixels.get(start..start + 4) == Some(&expected[..])
    })
}

#[test]
fn a_bevy_app_on_its_own_thread_shows_in_a_composition() {
    let bevy: Rc<OnceCell<Rc<BevyView>>> = Rc::new(OnceCell::new());
    let composed = Rc::clone(&bevy);
    let mut view = AppLauncher::new()
        .with_gpu_requirements(cranpose_bevy::gpu_requirements())
        .create_embedded_view(WIDTH, HEIGHT, 1.0, move || {
            let Some(gpu) = gpu_context() else {
                return;
            };
            let running = composed
                .get_or_init(|| {
                    let red = clear_to(BevyColor::srgb(1.0, 0.0, 0.0));
                    Rc::new(BevyView::new(&gpu, red).expect("Bevy thread"))
                })
                .clone();
            let frame = rememberMutableStateOf(|| None::<ImageBitmap>);
            FrameEffect((), true, move |_| {
                if let Some(shown) = running.show(WIDTH, HEIGHT) {
                    frame.set(Some(shown));
                }
            });
            Box(
                Modifier::empty().fill_max_size().draw_behind(move |scope| {
                    if let Some(frame) = frame.value() {
                        scope.draw_image_at(Rect::from_size(scope.size()), frame, 1.0, None);
                    }
                }),
                BoxSpec::default(),
                || {},
            );
        })
        .expect("GPU embedded component");
    assert!(wait_for(&mut view, [255, 0, 0, 255]), "Bevy's red frame");

    let bevy = bevy.get().expect("composed");
    bevy.send(|world| world.insert_resource(ClearColor(BevyColor::srgb(0.0, 0.0, 1.0))));
    assert!(
        wait_for(&mut view, [0, 0, 255, 255]),
        "the command's blue frame"
    );
}

// A camera at the origin, and a one metre red screen two metres ahead of it
// that counts its clicks.
fn screen_ahead(clicks: Arc<AtomicU32>) -> impl FnOnce(&mut App, RenderTarget) + Send + 'static {
    move |app, target| {
        app.insert_resource(ClearColor(BevyColor::srgb(0.0, 0.0, 1.0)))
            .add_plugins(CranposeScreenPlugin);
        app.add_systems(
            Startup,
            move |mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>| {
                commands.spawn((
                    Camera3d::default(),
                    Tonemapping::None,
                    target.clone(),
                    Transform::default().looking_at(Vec3::NEG_Z, Vec3::Y),
                ));
                let clicks = Arc::clone(&clicks);
                commands.spawn((
                    Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))),
                    Transform::from_xyz(0.0, 0.0, -2.0),
                    CranposeScreen::new(Vec2::splat(40.0), 1.0, move |_| {
                        let clicks = Arc::clone(&clicks);
                        Box::new(move || {
                            let clicks = Arc::clone(&clicks);
                            Box(
                                Modifier::empty()
                                    .fill_max_size()
                                    .background(Color::from_rgb_u8(255, 0, 0))
                                    .clickable(move |_| {
                                        clicks.fetch_add(1, Ordering::Relaxed);
                                    }),
                                BoxSpec::default(),
                                || {},
                            );
                        })
                    }),
                ));
            },
        );
    }
}

#[test]
fn a_press_on_a_screen_in_bevys_view_clicks_it_and_keeps_its_drag() {
    let clicks = Arc::new(AtomicU32::new(0));
    let counted = Arc::clone(&clicks);
    let bevy: Rc<OnceCell<Rc<BevyView>>> = Rc::new(OnceCell::new());
    let composed = Rc::clone(&bevy);
    // The moves that reached the window.
    let moves: Rc<Cell<u32>> = Rc::default();
    let seen = Rc::clone(&moves);
    let mut view = AppLauncher::new()
        .with_gpu_requirements(cranpose_bevy::gpu_requirements())
        .create_embedded_view(WIDTH, HEIGHT, 1.0, move || {
            let Some(running) = rememberBevyView(screen_ahead(Arc::clone(&counted))) else {
                return;
            };
            composed.get_or_init(|| Rc::clone(&running));
            let seen = Rc::clone(&seen);
            Box(
                Modifier::empty()
                    .fill_max_size()
                    .pointer_input((), move |scope| {
                        let seen = Rc::clone(&seen);
                        async move {
                            scope
                                .await_pointer_event_scope(|events| async move {
                                    loop {
                                        let event = events.await_pointer_event().await;
                                        if event.kind == PointerEventKind::Move {
                                            seen.set(seen.get() + 1);
                                        }
                                    }
                                })
                                .await;
                        }
                    }),
                BoxSpec::default(),
                move || BevyScene(Modifier::empty().fill_max_size(), Rc::clone(&running)),
            );
        })
        .expect("GPU embedded component");
    assert!(wait_for(&mut view, RED), "the screen in Bevy's frame");
    let bevy = Rc::clone(bevy.get().expect("composed"));

    view.shell().set_cursor(32.0, 24.0);
    view.shell().pointer_pressed();
    assert!(
        draw_until(&mut view, |_| bevy.holds_press()),
        "the screen took the press"
    );
    let before = moves.get();
    view.shell().set_cursor(34.0, 25.0);
    assert_eq!(
        moves.get(),
        before,
        "the screen kept its drag from the window"
    );
    view.shell().pointer_released();
    assert!(
        draw_until(&mut view, |_| clicks.load(Ordering::Relaxed) == 1),
        "the screen's click"
    );

    // Beside the screen the press stays the window's.
    assert!(draw_until(&mut view, |_| !bevy.holds_press()));
    view.shell().set_cursor(2.0, 2.0);
    view.shell().pointer_pressed();
    let pressed = bevy.shown_frames();
    assert!(draw_until(&mut view, |_| bevy.shown_frames() >= pressed + 3));
    assert!(!bevy.holds_press(), "a press beside the screen");
    let before = moves.get();
    view.shell().set_cursor(4.0, 3.0);
    assert_eq!(moves.get(), before + 1, "the window's drag");
    view.shell().pointer_released();
    assert_eq!(clicks.load(Ordering::Relaxed), 1);
}
