# Bevy game with a Cranpose HUD

A Bevy game with a Cranpose HUD over its camera: a lit cube turns at the speed
the HUD's slider sets, and the HUD counts its turns.

- `src/main.rs` is the game. It adds `CranposeHudPlugin::new(hud::content)` and
  puts `HudCamera` on the camera the HUD draws over.
- `src/hud.rs` is the HUD. `content` shares a `Hud` handle with the game as a
  non-send resource; the game's systems read and write the HUD's state through
  it.

```bash
cargo run
```

Start with the game in `src/main.rs` and the panel in `src/hud.rs`, and rename
the package in `Cargo.toml`. `cranpose_bevy::hud::HudInput` tells the game's
systems whether the HUD took the pointer or the keys, so they can leave that
input alone.
