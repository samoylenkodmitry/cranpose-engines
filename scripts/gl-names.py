#!/usr/bin/env python3
"""Writes crates/cranpose-gpu-interop/src/gl_names.rs: the WebGL entry points
an Emscripten module can take the address of, from Emscripten's own
system/lib/gl/webgl1.c and webgl2.c, for wgpu's GL loader.

Godot's web templates are built without Emscripten's
emscripten_webgl_get_proc_address, so the table resolves the names itself.
The buffer-mapping calls are left out: WebGL has no mapping, and the
templates do not export Emscripten's emulation of it.

    scripts/gl-names.py ~/emsdk/upstream/emscripten
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEFT_OUT = {"glFlushMappedBufferRange", "glGetBufferPointerv", "glMapBufferRange", "glUnmapBuffer"}


def main() -> None:
    emscripten = Path(sys.argv[1])
    version = (emscripten / "emscripten-version.txt").read_text().strip().strip('"')
    names = []
    for file in ("webgl1.c", "webgl2.c"):
        source = (emscripten / "system/lib/gl" / file).read_text()
        body = source[source.index("_get_proc_address(const char *name) {") :]
        for match in re.finditer(r"RETURN_FN(?:_WITH_SUFFIX)?\((\w+)(?:,\s*(\w+))?\)", body):
            name, suffix = match.group(1), match.group(2)
            symbol = name + (suffix or "")
            if symbol not in LEFT_OUT and (name, symbol) not in names:
                names.append((name, symbol))
    symbols = sorted({symbol for _, symbol in names})
    lines = [
        f"// Written by scripts/gl-names.py from Emscripten {version}.",
        "",
        "use std::ffi::c_void;",
        "",
        "// Taking a symbol's address makes Emscripten's dynamic linker resolve it",
        "// against the main module's exports; nothing calls these declarations.",
        'unsafe extern "C" {',
        *[f"    fn {symbol}();" for symbol in symbols],
        "}",
        "",
        "/// The address of the WebGL entry point Emscripten names `name`, or null.",
        "pub(crate) fn gl_proc_address(name: &str) -> *const c_void {",
        "    match name {",
        *[f'        "{name}" => {symbol} as *const c_void,' for name, symbol in names],
        "        _ => std::ptr::null(),",
        "    }",
        "}",
        "",
    ]
    out = ROOT / "crates/cranpose-gpu-interop/src/gl_names.rs"
    out.write_text("\n".join(lines))
    subprocess.run(["rustfmt", "--edition", "2024", str(out)], check=True)
    print(f"{out}: {len(names)} entry points")


if __name__ == "__main__":
    main()
