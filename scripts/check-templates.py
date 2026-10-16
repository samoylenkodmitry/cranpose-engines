#!/usr/bin/env python3
"""Checks every template against this checkout.

Each template is copied to a scratch folder, as a developer would copy it,
and its dependencies on this repository's crates are pointed at the local ones.
Then cargo runs clippy (the default) or the given cargo command in it.

    scripts/check-templates.py            # clippy, warnings are errors
    scripts/check-templates.py build      # cargo build

Set CRANPOSE_PATH to a Cranpose checkout to build against its crates instead
of the released ones.
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCAL = {
    "cranpose-bevy": ROOT / "crates" / "cranpose-bevy",
    "cranpose-godot": ROOT / "crates" / "cranpose-godot",
}
ENGINE_DEPENDENCY = re.compile(
    r'^(?P<name>' + "|".join(map(re.escape, LOCAL)) + r') = "[^"]+"$',
    re.MULTILINE,
)


CRANPOSE_DEPENDENCY = re.compile(r'^cranpose = \{ version = "[^"]+",', re.MULTILINE)


def local_manifest(text: str) -> str:
    def to_path(match: re.Match) -> str:
        name = match.group("name")
        return f'{name} = {{ path = "{LOCAL[name]}" }}'

    text = ENGINE_DEPENDENCY.sub(to_path, text)
    cranpose = os.environ.get("CRANPOSE_PATH")
    if cranpose:
        local = Path(cranpose) / "crates" / "cranpose"
        text = CRANPOSE_DEPENDENCY.sub(f'cranpose = {{ path = "{local}",', text)
    return text


def cargo_command(mode: str) -> list[str]:
    if mode == "clippy":
        command = ["cargo", "clippy", "--all-targets"]
        tail = ["--", "-D", "warnings"]
    else:
        command = ["cargo", mode]
        tail = []
    cranpose = os.environ.get("CRANPOSE_PATH")
    if cranpose:
        for name, path in (
            ("cranpose", "crates/cranpose"),
            ("cranpose-wgpu", "forks/wgpu/wgpu"),
            ("cranpose-wgpu-hal", "forks/wgpu/wgpu-hal"),
        ):
            command += ["--config", f'patch.crates-io.{name}.path="{Path(cranpose) / path}"']
    return command + tail


def main() -> None:
    mode = sys.argv[1] if len(sys.argv) > 1 else "clippy"
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target" / "templates"))
    with tempfile.TemporaryDirectory(prefix="cranpose-templates-") as scratch:
        for template in sorted((ROOT / "templates").iterdir()):
            if not template.is_dir():
                continue
            if "godot" in template.name and sys.platform != "darwin":
                # The Godot bridge runs on Godot's Metal driver only.
                print(f"== {template.name}: skipped, macOS only", flush=True)
                continue
            copy = Path(scratch) / template.name
            shutil.copytree(template, copy, ignore=shutil.ignore_patterns("target", ".godot"))
            for manifest in copy.rglob("Cargo.toml"):
                manifest.write_text(local_manifest(manifest.read_text()))
                print(f"== {template.name}: {manifest.relative_to(copy)}", flush=True)
                subprocess.run(cargo_command(mode), cwd=manifest.parent, env=env, check=True)


if __name__ == "__main__":
    main()
