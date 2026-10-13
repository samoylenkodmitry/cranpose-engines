# Cranpose with game engines

[Cranpose](https://github.com/samoylenkodmitry/Cranpose) and game engines on
one GPU device, in both directions: an engine's 3D scene inside a Cranpose
layout, and a Cranpose interface over an engine's scene. Frames pass between
them as GPU textures; no pixels go through the CPU.

| | Bevy 0.20 | Godot 4.7 |
| --- | --- | --- |
| The engine's scene inside Cranpose | `cranpose_bevy::BevyScene` | the `godot` example's `window.tscn` |
| A Cranpose HUD over the engine | `cranpose_bevy::hud::CranposeHudPlugin` | `cranpose_godot::Surface` |

![Bevy inside Cranpose](screenshots/bevy-in-cranpose.jpg)

![Cranpose HUD in Bevy](screenshots/cranpose-hud-bevy.jpg)

![Godot inside Cranpose](screenshots/godot-in-cranpose.jpg)

![Cranpose HUD in Godot](screenshots/cranpose-hud-godot.jpg)

## Try it

The two Bevy examples run in the browser at
[samoylenkodmitry.github.io/cranpose-engines](https://samoylenkodmitry.github.io/cranpose-engines/):
on WebGPU where the browser has it (Chrome, Edge, Safari 26), and on WebGL2
elsewhere, with a plain sky in place of the atmosphere, which needs compute
shaders. On a phone the controls go under the view, and a stick walks.

Each [release](https://github.com/samoylenkodmitry/cranpose-engines/releases)
has the two Bevy examples for macOS, Linux and Windows, and the Godot example
project for macOS. All four show the same valley: a winding river, trees,
flower meadows, grass and rocks, closed by mountains. Walk with WASD or the
arrow keys, run with Shift, and drag to look. Inside a Cranpose window, click
the scene first so it has the keyboard focus.

The macOS builds are not signed, so macOS asks before it opens one the first
time: allow it in System Settings, Privacy & Security. The Godot example needs
[Godot 4.7](https://godotengine.org/download) with its Metal driver; unzip it
and open `res://hud.tscn` (a HUD over Godot's scene) or `res://window.tscn`
(Godot's scene inside a Cranpose window):

```bash
godot --path cranpose-engines-godot-macos-arm64 res://hud.tscn
```

## Start a project

Each folder in [`templates`](templates) is a project to copy and rename:

| Template | Start from | You get |
| --- | --- | --- |
| [`cranpose-app-with-bevy`](templates/cranpose-app-with-bevy) | Cranpose | A desktop app whose layout shows a Bevy scene next to Cranpose controls. |
| [`bevy-game-with-cranpose-hud`](templates/bevy-game-with-cranpose-hud) | Bevy | A Bevy game with a Cranpose HUD that reads and changes game state. |
| [`godot-game-with-cranpose-hud`](templates/godot-game-with-cranpose-hud) | Godot | A Godot project with a Rust extension that draws a Cranpose HUD. macOS. |

```bash
git clone --depth 1 https://github.com/samoylenkodmitry/cranpose-engines
```

```bash
cp -R cranpose-engines/templates/cranpose-app-with-bevy my-app
```

```bash
cargo run --manifest-path my-app/Cargo.toml
```

Each template keeps the engine code and the interface code in separate files,
and its README says what to change first.

## Add to an existing project

The crates come from this repository by git tag:

```toml
[dependencies]
cranpose-bevy = { git = "https://github.com/samoylenkodmitry/cranpose-engines", tag = "v0.1.1" }

# Bevy requires crates.io's wgpu; this resolves it to the wgpu fork Cranpose
# uses, so Bevy and Cranpose share one device.
[patch.crates-io]
wgpu = { git = "https://github.com/samoylenkodmitry/cranpose-engines", tag = "v0.1.1" }
```

- **A Bevy scene in a Cranpose app.** Ask the window's device for Bevy's
  features with `AppLauncher::with_gpu_requirements(cranpose_bevy::gpu_requirements())`.
  `rememberBevyView(setup)` starts Bevy on its own thread with your `setup`,
  which adds the scene and gives its camera the render target Cranpose shows.
  `BevyScene(modifier, view)` shows it, and `view.send(|world| ...)` changes
  Bevy's world from the interface. If your build also turns on Bevy's
  `bevy_winit`, for a game window, turn on `cranpose-bevy`'s `bevy_winit`
  feature too: Bevy's thread then runs without the window plugin.
- **A Cranpose HUD in a Bevy game.** Add `CranposeHudPlugin::new(content)` and
  put `HudCamera` on the camera the HUD draws over. `content` gets Bevy's world
  once, to share state with the game, and returns the HUD's root composable.
  `HudInput` tells the game whether the HUD took the pointer, a finger or
  the keys.
- **A Cranpose HUD in Godot.** Depend on `cranpose-godot` (no patch needed),
  declare the extension with `cranpose_godot::godot_extension!(Extension)`, and
  give a node a `Surface` on a `CanvasLayer`. Call `draw` each frame and pass
  input events to `input` first.

## Build the examples

```bash
cargo run -p bevy-in-cranpose
```

```bash
cargo run -p cranpose-hud-bevy
```

```bash
cargo build -p cranpose-godot-demo
```

```bash
godot --path examples/godot/project res://window.tscn
```

```bash
godot --path examples/godot/project res://hud.tscn
```

The Godot projects keep
`.godot/extension_list.cfg`, so Godot loads the Rust extension without an
import first.

The web examples build with `scripts/build-web.sh` into `dist/web`, which any
static file server can serve. Each example has a WebGPU build in `pkg` and a
WebGL2 build in `pkg-webgl2`; the page loads the one the browser runs.

## Where it runs

- Bevy in both directions: macOS (Metal) is tested; the Linux and Windows
  builds use wgpu's Vulkan and D3D12 backends and are built by CI.
- Godot in both directions: macOS with Godot's Metal driver. The bridge imports
  Godot's Metal objects; Vulkan and D3D12 are not bridged yet.
- The web: the Bevy directions over WebGPU and WebGL2, tested in Chrome on
  Android and in WebKit. Bevy updates inside Cranpose's frame there, since a
  page has one thread. Godot's web export renders through its own WebGL
  context, which is not bridged.

Progress and the open work are tracked in
[Cranpose#1412](https://github.com/samoylenkodmitry/Cranpose/issues/1412).

## How the device is shared

Bevy 0.20 builds against wgpu 30, the release Cranpose's fork starts from.
[`wgpu-shim`](wgpu-shim) is a package named `wgpu` that re-exports the fork,
and a project patches crates.io's `wgpu` with it. Bevy and Cranpose then use one
`wgpu::Device` type and one device, so frames pass between them as textures,
with no copies and no unsafe code. Cargo ignores a `[patch]` entry that renames
a package, so the re-export package is needed.

- **Bevy inside Cranpose.** `cranpose_bevy::BevyView` (which
  `rememberBevyView` starts and `BevyScene` shows) runs Bevy on a thread of its
  own, with Bevy's renderer on Cranpose's device
  (`cranpose::gpu_context()`). The composition calls `BevyView::show` once per
  frame: Bevy may run one more update, and the composition gets the newest frame
  Bevy finished. Frames pass through `cranpose::ExternalFrames`, a ring of at
  most four textures: Bevy's render world publishes a frame once its work is
  submitted, and a texture goes back to Bevy once Cranpose shows a newer frame
  and holds no bitmap of the old one. Neither side waits for the other, so the
  interface keeps its frame rate when Bevy's frames take longer. The app asks
  the window's device for Bevy's features with
  `AppLauncher::with_gpu_requirements`. Bevy's thread is not the main thread,
  so it runs without Bevy's window plugin.
- **Cranpose HUD in Bevy.** `cranpose_bevy::hud::CranposeHudPlugin` creates a
  `GuestView` on Bevy's device and draws the HUD over the camera marked with
  `HudCamera`. Cranpose draws on Bevy's main thread into one of two textures,
  only when the composition changed, while the render world reads the other.
  A pass after Bevy's upscaling blends the newest HUD over the camera's final
  image with premultiplied alpha, at the target's size from the same frame.
  `HudInput` tells the game whether the HUD took the held press, which
  finger it took, and whether it took the last key. The HUD takes one finger
  at a time, the first that lands on something of the HUD's, so the game can
  follow the others.

## How Godot is bridged

Godot renders through its own Metal driver, so Cranpose cannot share its
`wgpu::Device`. `cranpose-gpu-interop`, the one crate here with unsafe code, builds
a wgpu device on Godot's `MTLDevice` from the handles
`RenderingDevice.get_driver_resource` returns.

Cranpose submits to a queue of its own. wgpu opens a Metal command buffer for
each pass and holds them until it submits, and Godot's queue allows 64
uncompleted command buffers, which Godot and a Cranpose frame with glass
together exhaust. Each frame, one command buffer on Godot's queue hands the
textures over:

- **Godot to Cranpose:** it waits for Cranpose's last frame, copies the
  `SubViewport` texture into a texture Cranpose samples, and signals an event
  that Cranpose's next submit waits for.
- **Cranpose to Godot:** it waits for the event Cranpose's frame signals and
  copies the frame into the texture a `TextureRect` shows. Godot's own command
  buffers that read it come later on the same queue, so Metal orders them
  after the copy.

The copies run on the GPU; no pixels pass through the CPU.

## Release

1. Point the templates at the new tag with `scripts/set-template-tag.sh vX.Y.Z`
   and commit.
2. Push the tag. `release.yml` builds the downloads and attaches them to the
   tag's release, and `pages.yml` publishes the web examples.
