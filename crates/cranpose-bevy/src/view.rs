#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::{
    cell::Cell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel},
    thread::JoinHandle,
};

use bevy::{
    app::PluginsState,
    camera::{ManualTextureViewHandle, RenderTarget},
    input::{
        ButtonState,
        mouse::MouseButtonInput,
        touch::{TouchInput, TouchPhase},
    },
    math::UVec2,
    prelude::*,
    render::{
        Render, RenderApp, RenderPlugin, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        texture::{ManualTextureView, ManualTextureViews},
    },
    window::{CursorMoved, ExitCondition},
};
use cranpose::{ExternalFrames, FrameWriter, GpuContext, prelude::ImageBitmap};

use crate::{CranposeInput, input::PointerTarget};

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
/// crate's `bevy_winit` feature too, which leaves the plugin out. The pointer
/// reaches Bevy's world through [`Self::pointer`], as a window's would.
pub struct BevyView {
    frames: ExternalFrames,
    runner: Runner,
    shown: Cell<u64>,
    max_density: Cell<f32>,
    holds_press: Arc<AtomicBool>,
}

/// A pointer sample on a [`BevyView`], in the pixels of Bevy's frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewPointer {
    /// The mouse moved here.
    Moved(Vec2),
    /// The left button went down here.
    Pressed(Vec2),
    /// The left button went up here.
    Released(Vec2),
    /// A finger touched, moved or lifted here.
    Touch {
        finger: u64,
        phase: TouchPhase,
        at: Vec2,
    },
}

#[cfg(not(target_arch = "wasm32"))]
struct Runner {
    ticks: Option<SyncSender<()>>,
    commands: Sender<Command>,
    pointers: Sender<ViewPointer>,
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

// Tells the composition whether a Cranpose surface in Bevy's world holds the
// press.
#[derive(Resource)]
struct PressReport(Arc<AtomicBool>);

impl BevyView {
    /// Starts Bevy. `setup` adds the scene and gets the render target its
    /// camera draws into.
    pub fn new(
        gpu: &GpuContext,
        setup: impl FnOnce(&mut App, RenderTarget) + Send + 'static,
    ) -> std::io::Result<Self> {
        let frames = ExternalFrames::new(gpu, FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT, true);
        let holds_press = Arc::new(AtomicBool::new(false));
        let runner = Runner::start(gpu, frames.writer(), Arc::clone(&holds_press), setup)?;
        Ok(Self {
            frames,
            runner,
            shown: Cell::new(0),
            max_density: Cell::new(f32::INFINITY),
            holds_press,
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

    /// Hands Bevy's world a pointer sample before its next update, as the
    /// window messages a Bevy game gets: `CursorMoved`, `MouseButtonInput`
    /// and `TouchInput`. [`crate::BevyScene`] sends the pointer that lands
    /// on it.
    pub fn pointer(&self, sample: ViewPointer) {
        self.runner.pointer(sample);
    }

    /// Whether a Cranpose screen in Bevy's world held the press when Bevy
    /// last updated, for the composition around the view to leave it alone.
    pub fn holds_press(&self) -> bool {
        self.holds_press.load(Ordering::Relaxed)
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
        holds_press: Arc<AtomicBool>,
        setup: impl FnOnce(&mut App, RenderTarget) + Send + 'static,
    ) -> std::io::Result<Self> {
        let (ticks, tick_receiver) = sync_channel(1);
        let (commands, command_receiver) = channel();
        let (pointers, pointer_receiver) = channel();
        let gpu = gpu.clone();
        let thread = std::thread::Builder::new()
            .name("bevy".into())
            .spawn(move || {
                let mut app = start(&gpu, writer.clone(), holds_press, setup);
                let inputs = Inputs {
                    ticks: tick_receiver,
                    commands: command_receiver,
                    pointers: pointer_receiver,
                };
                run(&mut app, &writer, &inputs);
            })?;
        Ok(Self {
            ticks: Some(ticks),
            commands,
            pointers,
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

    fn pointer(&self, sample: ViewPointer) {
        let _ = self.pointers.send(sample);
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
        holds_press: Arc<AtomicBool>,
        setup: impl FnOnce(&mut App, RenderTarget) + Send + 'static,
    ) -> std::io::Result<Self> {
        Ok(Self {
            app: RefCell::new(start(gpu, writer.clone(), holds_press, setup)),
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

    fn pointer(&self, sample: ViewPointer) {
        if let Ok(mut app) = self.app.try_borrow_mut() {
            write_pointer(app.world_mut(), sample);
        }
    }
}

fn start(
    gpu: &GpuContext,
    writer: FrameWriter,
    holds_press: Arc<AtomicBool>,
    setup: impl FnOnce(&mut App, RenderTarget),
) -> App {
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
        .add_plugins(ExtractResourcePlugin::<FrameNumber>::default())
        .insert_resource(PressReport(holds_press))
        .insert_resource(PointerTarget::TextureView(TARGET))
        .add_systems(Update, report_press);
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

// What Bevy's thread receives from the composition.
#[cfg(not(target_arch = "wasm32"))]
struct Inputs {
    ticks: Receiver<()>,
    commands: Receiver<Command>,
    pointers: Receiver<ViewPointer>,
}

#[cfg(not(target_arch = "wasm32"))]
fn run(app: &mut App, writer: &FrameWriter, inputs: &Inputs) {
    while inputs.ticks.recv().is_ok() {
        for command in inputs.commands.try_iter() {
            command(app.world_mut());
        }
        for sample in inputs.pointers.try_iter() {
            write_pointer(app.world_mut(), sample);
        }
        step(app, writer);
    }
}

fn write_pointer(world: &mut World, sample: ViewPointer) {
    let window = Entity::PLACEHOLDER;
    let (at, button) = match sample {
        ViewPointer::Moved(at) => (at, None),
        ViewPointer::Pressed(at) => (at, Some(ButtonState::Pressed)),
        ViewPointer::Released(at) => (at, Some(ButtonState::Released)),
        ViewPointer::Touch { finger, phase, at } => {
            world.write_message(TouchInput {
                phase,
                position: at,
                window,
                force: None,
                id: finger,
            });
            return;
        }
    };
    world.write_message(CursorMoved {
        window,
        position: at,
        delta: None,
    });
    if let Some(state) = button {
        world.write_message(MouseButtonInput {
            button: MouseButton::Left,
            state,
            window,
        });
    }
}

fn report_press(input: Option<Res<CranposeInput>>, report: Res<PressReport>) {
    let holds = input.is_some_and(|input| input.holds_press());
    report.0.store(holds, Ordering::Relaxed);
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
