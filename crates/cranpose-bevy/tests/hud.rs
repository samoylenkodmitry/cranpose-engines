use std::sync::{Arc, Mutex};

use bevy::{
    app::PluginsState,
    camera::RenderTarget,
    core_pipeline::tonemapping::Tonemapping,
    input::{
        ButtonState,
        keyboard::{Key, KeyCode, KeyboardInput},
    },
    prelude::{
        App, Assets, Camera3d, ClearColor, Color as BevyColor, Commands, DefaultPlugins, Entity,
        Image, On, PluginGroup, ResMut, Startup, WindowPlugin, World, default,
    },
    render::{
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{TextureFormat, TextureUsages},
    },
    tasks::tick_global_task_pools_on_main_thread,
    window::ExitCondition,
};
use cranpose::prelude::*;
use cranpose_bevy::hud::{CranposeHudPlugin, HudCamera, HudInput};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;
const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

type Pixels = Arc<Mutex<Option<Vec<u8>>>>;

/// A headless Bevy app whose camera renders a blue image with `hud` over it.
/// The image is read back into the returned cell every frame.
fn hud_app(hud: impl Fn(&mut World) -> Box<dyn FnMut()> + Send + Sync + 'static) -> (App, Pixels) {
    let read = Pixels::default();
    let sink = Arc::clone(&read);
    let plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: None,
        exit_condition: ExitCondition::DontExit,
        close_when_requested: false,
        ..default()
    });
    // Tests run off the main thread, where Bevy's window plugin cannot start.
    #[cfg(feature = "bevy_winit")]
    let plugins = plugins.disable::<bevy::winit::WinitPlugin>();
    let mut app = App::new();
    app.add_plugins(plugins)
        .add_plugins(CranposeHudPlugin::new(hud))
        .insert_resource(ClearColor(BevyColor::srgb(0.0, 0.0, 1.0)))
        .add_systems(
            Startup,
            move |mut commands: Commands, mut images: ResMut<Assets<Image>>| {
                let mut image =
                    Image::new_target_texture(WIDTH, HEIGHT, TextureFormat::Rgba8UnormSrgb, None);
                image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
                let image = images.add(image);
                commands.spawn((
                    Camera3d::default(),
                    Tonemapping::None,
                    RenderTarget::Image(image.clone().into()),
                    HudCamera,
                ));
                let sink = Arc::clone(&sink);
                commands.spawn(Readback::texture(image)).observe(
                    move |event: On<ReadbackComplete>| {
                        if let Ok(mut read) = sink.lock() {
                            *read = Some(event.data.clone());
                        }
                    },
                );
            },
        );
    while app.plugins_state() == PluginsState::Adding {
        tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    (app, read)
}

fn pixel(pixels: &[u8], x: u32, y: u32) -> [u8; 4] {
    let start = ((y * WIDTH + x) * 4) as usize;
    [
        pixels[start],
        pixels[start + 1],
        pixels[start + 2],
        pixels[start + 3],
    ]
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
    let mut last = None;
    for _ in 0..600 {
        app.update();
        let pixels = read.lock().expect("readback").take();
        if let Some(pixels) = pixels {
            let seen = (pixel(&pixels, 8, 8), pixel(&pixels, WIDTH - 8, HEIGHT - 8));
            if seen == (RED, BLUE) {
                return;
            }
            last = Some(seen);
        }
    }
    panic!("the HUD never showed over the camera's image; last read {last:?}");
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
    assert!(app.world().resource::<HudInput>().keyboard);
}
