//! A headless Bevy app for the tests, and the image its camera renders.

use std::sync::{Arc, Mutex};

use bevy::{
    app::PluginsState,
    camera::RenderTarget,
    core_pipeline::tonemapping::Tonemapping,
    ecs::bundle::Bundle,
    prelude::{
        App, Assets, Camera3d, Color, Commands, DefaultPlugins, Image, On, PluginGroup, ResMut,
        Startup, Window, WindowPlugin, default,
    },
    render::{
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{TextureFormat, TextureUsages},
    },
    tasks::tick_global_task_pools_on_main_thread,
    window::ExitCondition,
};

pub const WIDTH: u32 = 64;
pub const HEIGHT: u32 = 48;
pub const RED: [u8; 4] = [255, 0, 0, 255];
pub const BLUE: [u8; 4] = [0, 0, 255, 255];

pub type Pixels = Arc<Mutex<Option<Vec<u8>>>>;

/// A Bevy app with the default plugins but no window system. With
/// `primary_window` it has a primary window entity, which no window backs.
pub fn app(primary_window: bool) -> App {
    let plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: primary_window.then(Window::default),
        exit_condition: ExitCondition::DontExit,
        close_when_requested: false,
        ..default()
    });
    // Tests run off the main thread, where Bevy's window plugin cannot start.
    #[cfg(feature = "bevy_winit")]
    let plugins = plugins.disable::<bevy::winit::WinitPlugin>();
    let mut app = App::new();
    app.add_plugins(plugins)
        .insert_resource(bevy::prelude::ClearColor(Color::srgb(0.0, 0.0, 1.0)));
    app
}

/// Adds a camera with `extra` that renders a blue image of `WIDTH` by
/// `HEIGHT`, which is read back into the returned cell every frame.
pub fn image_camera(app: &mut App, extra: impl Bundle + Clone) -> Pixels {
    let read = Pixels::default();
    let sink = Arc::clone(&read);
    app.add_systems(
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
                extra.clone(),
            ));
            let sink = Arc::clone(&sink);
            commands
                .spawn(Readback::texture(image))
                .observe(move |event: On<ReadbackComplete>| {
                    if let Ok(mut read) = sink.lock() {
                        *read = Some(event.data.clone());
                    }
                });
        },
    );
    read
}

/// Waits for the plugins and finishes the app, ready for `update`.
pub fn start(app: &mut App) {
    while app.plugins_state() == PluginsState::Adding {
        tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
}

/// Runs frames until an image read back has each color at its point, for
/// at most 600 frames: read back images arrive frames late, and later on a
/// software GPU.
pub fn shows(app: &mut App, read: &Pixels, expected: &[((u32, u32), [u8; 4])]) -> bool {
    (0..600).any(|_| {
        app.update();
        let pixels = read.lock().map(|mut read| read.take()).ok().flatten();
        pixels.is_some_and(|pixels| {
            expected
                .iter()
                .all(|&((x, y), color)| pixel(&pixels, x, y) == color)
        })
    })
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
