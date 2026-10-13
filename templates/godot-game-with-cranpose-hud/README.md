# Godot game with a Cranpose HUD

A Godot 4.7 project with a Rust extension that draws a Cranpose HUD over the
scene. The extension's `CranposeHud` node turns the node its `target` names at
the speed the HUD's slider sets, and the HUD counts the turns. macOS with
Godot's Metal driver.

- `rust/src/lib.rs` is the extension: the `CranposeHud` node, which owns a
  `cranpose_godot::Surface` on a `CanvasLayer`.
- `rust/src/hud.rs` is the HUD composition and the state it shares with the
  node.
- `project/` is the Godot project. `main.tscn` has the cube, a camera, a light
  and the `CranposeHud` node; `cranpose.gdextension` loads the built library.

```bash
cargo build --manifest-path rust/Cargo.toml
```

```bash
godot --path project
```

The project keeps `.godot/extension_list.cfg`, so Godot loads the extension
without an import first. Start with the scene in the Godot editor and the
panel in `rust/src/hud.rs`, and rename the crate in `rust/Cargo.toml` together
with the library name in `cranpose.gdextension`.
