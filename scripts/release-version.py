#!/usr/bin/env python3
"""Sets the published crates' version from a release tag.

    scripts/release-version.py v0.1.8

The workspace version, the workspace's dependencies on the published crates,
the templates and the README take the tag's version. The README's git lines
take the tag itself.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATES = r"cranpose-(?:bevy|godot|gpu-interop)"


def rewrite(path: Path, version: str, tag: str) -> None:
    text = path.read_text()
    new = re.sub(
        r'(\[workspace\.package\]\n(?:#.*\n)*)version = "[^"]+"',
        rf'\g<1>version = "{version}"',
        text,
    )
    new = re.sub(
        rf'^({CRATES} = \{{ path = "[^"]+", version = )"[^"]+"',
        rf'\g<1>"{version}"',
        new,
        flags=re.MULTILINE,
    )
    new = re.sub(rf'({CRATES} = )"[^"]+"', rf'\g<1>"{version}"', new)
    new = re.sub(r'(cranpose-engines", tag = )"[^"]+"', rf'\g<1>"{tag}"', new)
    if new != text:
        path.write_text(new)
        print(f"{path.relative_to(ROOT)}: {version}")


def main() -> None:
    if len(sys.argv) != 2 or not re.fullmatch(r"v\d+\.\d+\.\d+", sys.argv[1]):
        sys.exit("usage: release-version.py v<major>.<minor>.<patch>")
    tag = sys.argv[1]
    version = tag[1:]
    paths = [ROOT / "Cargo.toml", ROOT / "README.md"]
    paths += sorted((ROOT / "templates").glob("**/Cargo.toml"))
    for path in paths:
        rewrite(path, version, tag)


if __name__ == "__main__":
    main()
