# Cranpose app with Bevy

A Cranpose desktop app whose layout shows a Bevy scene: a lit cube that turns
over a floor, next to a slider that sets its speed.

- `src/main.rs` is the interface. `rememberBevyView(scene::build)` starts Bevy
  on a thread of its own, `BevyScene` shows its frames, and `bevy.send(...)`
  changes Bevy's world when the slider moves.
- `src/scene.rs` is Bevy's side. `build` adds the scene, and its camera draws
  into the render target Cranpose shows.

```bash
cargo run
```

Start with the scene in `src/scene.rs` and the controls in `src/main.rs`, and
rename the package in `Cargo.toml`. Bevy runs on its own thread and the window
belongs to Cranpose, so this project keeps Bevy's `bevy_winit` feature off. If
you add it for something else, turn on `cranpose-bevy`'s `bevy_winit` feature
too.
