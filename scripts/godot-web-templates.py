#!/usr/bin/env python3
"""Installs Godot's web export templates with extension support, without
threads, into Godot's template folder. They come out of the official export
templates archive by range requests, so only those entries download, about
24 MB of its 1.3 GB.

    scripts/godot-web-templates.py 4.7.2
"""

import os
import platform
import struct
import sys
import urllib.request
import zlib
from pathlib import Path

ENTRIES = ["version.txt", "web_dlink_nothreads_release.zip", "web_dlink_nothreads_debug.zip"]


def template_dir(version: str) -> Path:
    if platform.system() == "Darwin":
        base = Path.home() / "Library/Application Support/Godot"
    else:
        base = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share")) / "godot"
    return base / "export_templates" / f"{version}.stable"


def fetch(url: str, start: int, end: int) -> bytes:
    request = urllib.request.Request(url, headers={"Range": f"bytes={start}-{end}"})
    with urllib.request.urlopen(request) as response:
        return response.read()


def central_directory(url: str) -> dict[str, tuple[int, int, int]]:
    head = urllib.request.Request(url, method="HEAD")
    with urllib.request.urlopen(head) as response:
        size = int(response.headers["Content-Length"])
    tail = fetch(url, size - 65536, size - 1)
    end = tail.rfind(b"PK\x05\x06")
    length, offset = struct.unpack("<II", tail[end + 12 : end + 20])
    if offset == 0xFFFFFFFF:
        end64 = tail.rfind(b"PK\x06\x06")
        length, offset = struct.unpack("<QQ", tail[end64 + 40 : end64 + 56])
    directory = fetch(url, offset, offset + length - 1)
    entries = {}
    at = 0
    while directory[at : at + 4] == b"PK\x01\x02":
        (method,) = struct.unpack("<H", directory[at + 10 : at + 12])
        packed, unpacked = struct.unpack("<II", directory[at + 20 : at + 28])
        name_length, extra_length, comment_length = struct.unpack("<HHH", directory[at + 28 : at + 34])
        (local,) = struct.unpack("<I", directory[at + 42 : at + 46])
        name = directory[at + 46 : at + 46 + name_length].decode()
        extra = directory[at + 46 + name_length : at + 46 + name_length + extra_length]
        field = 0
        while field < len(extra):
            kind, field_length = struct.unpack("<HH", extra[field : field + 4])
            if kind == 1:
                values = extra[field + 4 : field + 4 + field_length]
                read = 0
                if unpacked == 0xFFFFFFFF:
                    read += 8
                if packed == 0xFFFFFFFF:
                    (packed,) = struct.unpack("<Q", values[read : read + 8])
                    read += 8
                if local == 0xFFFFFFFF:
                    (local,) = struct.unpack("<Q", values[read : read + 8])
            field += 4 + field_length
        entries[name] = (method, packed, local)
        at += 46 + name_length + extra_length + comment_length
    return entries


def main() -> None:
    version = sys.argv[1]
    url = f"https://github.com/godotengine/godot/releases/download/{version}-stable/Godot_v{version}-stable_export_templates.tpz"
    out = template_dir(version)
    if all((out / name).exists() for name in ENTRIES):
        print(f"{out}: already installed")
        return
    with urllib.request.urlopen(urllib.request.Request(url, method="HEAD")) as response:
        url = response.url
    entries = central_directory(url)
    out.mkdir(parents=True, exist_ok=True)
    for name in ENTRIES:
        method, packed, local = entries[f"templates/{name}"]
        header = fetch(url, local, local + 29)
        name_length, extra_length = struct.unpack("<HH", header[26:30])
        start = local + 30 + name_length + extra_length
        data = fetch(url, start, start + packed - 1)
        if method == 8:
            data = zlib.decompress(data, -15)
        (out / name).write_bytes(data)
        print(f"{out / name}: {len(data)} bytes")


if __name__ == "__main__":
    main()
