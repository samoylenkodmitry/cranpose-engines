use std::cell::Cell;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::{
    sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel},
    thread::JoinHandle,
};

use bevy::{
    app::PluginsState,
    camera::{ManualTextureViewHandle, RenderTarget},
    math::UVec2,
    prelude::*,
    render::{
        Render, RenderApp, RenderPlugin, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        texture::{ManualTextureView, ManualTextureViews},
    },
    window::ExitCondition,
};
use cranpose::{ExternalFrames, FrameWriter, GpuContext, prelude::ImageBitmap};

const TARGET: ManualTextureViewHandle = ManualTextureViewHandle(0);
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

type Command = Box<dyn FnOnce(&mut World) + Send>;

/// A Bevy app that renders with Cranpose's device, for a composition to show.
///
/// Bevy runs on a thread of its own. The composition calls [`Self::show`] once
/// per frame, or shows the view with [`crate::BevyScene`], which does. Each
/// call lets Bevy run one update and returns the newest frame Bevy finished,
/// so the composition never waits for Bevy, and Bevy never waits for the
/// composition. A web page has one thread, so there Bevy runs its update
/// inside the call. Two views are equal only when they are the same view.
///
/// Bevy runs without its window plugin, since windows belong to Cranpose and
/// Bevy's thread is not the main thread. A build that also turns on Bevy's
/// `bevy_winit` feature, such as one with a Bevy game window, turns on this
/// crate's `bevy_winit` feature too, which leaves the plugin out.
pub struct BevyView {
    frames: ExternalFrames,
    runner: Runner,
    shown: Cell<u64>,
    max_density: Cell<f32>,
}

#[cfg(not(target_arch = "wasm32"))]
struct Runner {
    ticks: Option<SyncSender<()>>,
    commands: Sender<Command>,
    thread: Option<JoinHandle<()>>,
}

#[cfg(target_arch = "wasm32")]
struct Runner {
    app: RefCell<App>,
    writer: FrameWriter,
}

#[derive(Resource, Clone, Copy, ExtractResource)]
#[extract_app(RenderApp)]
struct FrameNumber(u64);

#[derive(Resource)]
struct Publisher(FrameWriter);

impl BevyView {
    /// Starts Bevy. `setup` adds the scene and gets the render target its
    /// camera draws into.
    pub fn new(
        gpu: &GpuContext,
        setup: impl FnOnce(&mut App, RenderTarget) + Send + 'static,
    ) -> std::io::Result<Self> {
        let frames = ExternalFrames::new(gpu, FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT, true);
        let runner = Runner::start(gpu, frames.writer(), setup)?;
        Ok(Self {
            frames,
            runner,
            shown: Cell::new(0),
            max_density: Cell::new(f32::INFINITY),
        })
    }

    /// Lets [`crate::BevyScene`] ask Bevy for at most `density` pixels per
    /// dp, and scale Bevy's frames up to the view. A phone's screen has two
    /// to three pixels per dp; rendering 3D at 1.5 costs half or less.
    pub fn set_max_density(&self, density: f32) {
        self.max_density.set(density);
    }

    /// The most pixels per dp [`crate::BevyScene`] asks Bevy for.
    pub fn max_density(&self) -> f32 {
        self.max_density.get()
    }

    /// Asks Bevy for frames of `width` by `height` pixels, lets it run one
    /// more update, and returns the newest frame it finished since the last
    /// call.
    pub fn show(&self, width: u32, height: u32) -> Option<ImageBitmap> {
        self.frames.set_size(width, height);
        self.runner.tick();
        let frame = self.frames.next_frame();
        if frame.is_some() {
            self.shown.set(self.shown.get() + 1);
        }
        frame
    }

    /// How many of Bevy's frames [`Self::show`] returned so far.
    pub fn shown_frames(&self) -> u64 {
        self.shown.get()
    }

    /// Runs `command` on Bevy's world before its next update.
    pub fn send(&self, command: impl FnOnce(&mut World) + Send + 'static) {
        self.runner.send(Box::new(command));
    }
}

impl PartialEq for BevyView {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Runner {
    fn start(
        gpu: &GpuContext,
        writer: FrameWriter,
        setup: impl FnOnce(&mut App, RenderTarget) + Send + 'static,
    ) -> std::io::Result<Self> {
        let (ticks, tick_receiver) = sync_channel(1);
        let (commands, command_receiver) = channel();
        let gpu = gpu.clone();
        let thread = std::thread::Builder::new()
            .name("bevy".into())
            .spawn(move || {
                let mut app = start(&gpu, writer.clone(), setup);
                run(&mut app, &writer, &tick_receiver, &command_receiver);
            })?;
        Ok(Self {
            ticks: Some(ticks),
            commands,
            thread: Some(thread),
        })
    }

    fn tick(&self) {
        if let Some(ticks) = self.ticks.as_ref() {
            // A full channel means Bevy has not started the last update yet.
            let _ = ticks.try_send(());
        }
    }

    fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for Runner {
    fn drop(&mut self) {
        self.ticks = None;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            error!("Bevy's thread panicked");
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Runner {
    fn start(
        gpu: &GpuContext,
        writer: FrameWriter,
        setup: impl FnOnce(&mut App, RenderTarget) + Send + 'static,
    ) -> std::io::Result<Self> {
        Ok(Self {
            app: RefCell::new(start(gpu, writer.clone(), setup)),
            writer,
        })
    }

    fn tick(&self) {
        if let Ok(mut app) = self.app.try_borrow_mut()
            && ready(&mut app)
        {
            step(&mut app, &self.writer);
        }
    }

    fn send(&self, command: Command) {
        if let Ok(mut app) = self.app.try_borrow_mut() {
            command(app.world_mut());
        }
    }
}

fn start(gpu: &GpuContext, writer: FrameWriter, setup: impl FnOnce(&mut App, RenderTarget)) -> App {
    let mut app = App::new();
    let plugins = DefaultPlugins
        .set(RenderPlugin {
            render_creation: crate::render_creation(gpu),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: ExitCondition::DontExit,
            close_when_requested: false,
            ..default()
        });
    // A build that has Bevy's window plugin for a game window leaves it out
    // here: windows belong to Cranpose, and this Bevy runs off the main thread.
    #[cfg(feature = "bevy_winit")]
    let plugins = plugins.disable::<bevy::winit::WinitPlugin>();
    app.add_plugins(plugins)
        .add_plugins(ExtractResourcePlugin::<FrameNumber>::default());
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render
            .insert_resource(Publisher(writer))
            .add_systems(Render, publish.in_set(RenderSystems::Cleanup));
    }
    setup(&mut app, RenderTarget::TextureView(TARGET));
    #[cfg(not(target_arch = "wasm32"))]
    while !ready(&mut app) {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app
}

// Finishes the app's plugins once they are ready, and says whether they are.
fn ready(app: &mut App) -> bool {
    match app.plugins_state() {
        PluginsState::Adding => false,
        PluginsState::Ready => {
            app.finish();
            app.cleanup();
            true
        }
        PluginsState::Finished => {
            app.cleanup();
            true
        }
        PluginsState::Cleaned => true,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run(app: &mut App, writer: &FrameWriter, ticks: &Receiver<()>, commands: &Receiver<Command>) {
    while ticks.recv().is_ok() {
        for command in commands.try_iter() {
            command(app.world_mut());
        }
        step(app, writer);
    }
}

// Renders one frame into the next free texture. Every texture is in use while
// the composition has not drawn the frames published so far, and then Bevy
// waits for the next call.
fn step(app: &mut App, writer: &FrameWriter) {
    let Some(target) = writer.begin() else {
        return;
    };
    let texture = target.texture();
    let world = app.world_mut();
    world.resource_mut::<ManualTextureViews>().insert(
        TARGET,
        ManualTextureView {
            texture_view: target.view().clone().into(),
            size: UVec2::new(texture.width(), texture.height()),
            view_format: FORMAT,
        },
    );
    world.insert_resource(FrameNumber(target.number()));
    app.update();
}

// Runs in the render world after it submitted the frame's work.
fn publish(frame: Option<Res<FrameNumber>>, publisher: Res<Publisher>) {
    if let Some(frame) = frame {
        publisher.0.publish(frame.0);
    }
}
