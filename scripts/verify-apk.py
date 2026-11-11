#!/usr/bin/env python3
"""Checks a lume APK against the release requirements.

  scripts/verify-apk.py dist/lume-0.1.0-release.apk [--expect-abis armeabi-v7a,arm64-v8a]
                                                   [--max-size-mb 6] [--version 0.1.0]

Checks, using only the Python standard library:
  - the file size is within the limit;
  - the native libraries are exactly the expected ABIs;
  - every native library is stored uncompressed at a 16 KB boundary and its ELF segments are
    aligned to 16 KB (Android 15+ on 16 KB page devices);
  - every wallpaper is in assets/wallpaper/ exactly once, stored, and none is inside a native library
    (the wallpapers are read from the assets: a copy in each liblume.so would repeat them per ABI);
  - the manifest does not ask for QUERY_ALL_PACKAGES;
  - the manifest declares leanback as a feature and the version name, when given.

Exit status 0 when everything passes, 1 otherwise.
"""

import argparse
import struct
import sys
import zipfile

PAGE = 16 * 1024
MAX_LIBRARY_MB = 5
FORBIDDEN_PERMISSIONS = ["android.permission.QUERY_ALL_PACKAGES"]


def data_offset(info: zipfile.ZipInfo, raw: bytes) -> int:
    """Offset of an entry's data inside the file (after its local header)."""
    name_length, extra_length = struct.unpack_from("<HH", raw, info.header_offset + 26)
    return info.header_offset + 30 + name_length + extra_length


def elf_segment_alignments(library: bytes) -> list[int]:
    """p_align of the PT_LOAD segments of an ELF file."""
    if library[:4] != b"\x7fELF":
        raise ValueError("not an ELF file")
    is_64 = library[4] == 2
    if is_64:
        (phoff,) = struct.unpack_from("<Q", library, 32)
        phentsize, phnum = struct.unpack_from("<HH", library, 54)
    else:
        (phoff,) = struct.unpack_from("<I", library, 28)
        phentsize, phnum = struct.unpack_from("<HH", library, 42)
    alignments = []
    for index in range(phnum):
        offset = phoff + index * phentsize
        (kind,) = struct.unpack_from("<I", library, offset)
        if kind == 1:  # PT_LOAD
            alignments.append(struct.unpack_from("<Q", library, offset + 48)[0] if is_64 else struct.unpack_from("<I", library, offset + 28)[0])
    return alignments


def manifest_has(manifest: bytes, text: str) -> bool:
    """The binary manifest keeps its strings as UTF-8 or UTF-16 in the string pool."""
    return text.encode("utf-8") in manifest or text.encode("utf-16-le") in manifest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("apk")
    parser.add_argument("--expect-abis", default="armeabi-v7a,arm64-v8a")
    parser.add_argument("--max-size-mb", type=float, default=40.0)
    parser.add_argument("--version")
    args = parser.parse_args()

    problems: list[str] = []
    raw = open(args.apk, "rb").read()
    size_mb = len(raw) / (1024 * 1024)
    print(f"size: {size_mb:.2f} MB (limit {args.max_size_mb} MB)")
    if size_mb > args.max_size_mb:
        problems.append(f"the APK is {size_mb:.2f} MB, over the {args.max_size_mb} MB limit")

    with zipfile.ZipFile(args.apk) as apk:
        libraries = {i.filename: i for i in apk.infolist() if i.filename.startswith("lib/") and i.filename.endswith(".so")}
        abis = sorted({name.split("/")[1] for name in libraries})
        expected = sorted(a for a in args.expect_abis.split(",") if a)
        print(f"ABIs: {', '.join(abis) or 'none'}")
        if abis != expected:
            problems.append(f"expected the ABIs {expected}, found {abis}")
        for name, info in sorted(libraries.items()):
            if info.compress_type != zipfile.ZIP_STORED:
                problems.append(f"{name} is compressed (it must be stored so it can be mapped)")
                continue
            offset = data_offset(info, raw)
            if offset % PAGE:
                problems.append(f"{name} starts at {offset}, not on a 16 KB boundary")
            try:
                alignments = elf_segment_alignments(apk.read(name))
            except ValueError as error:
                problems.append(f"{name}: {error}")
                continue
            if not alignments or min(alignments) < PAGE:
                problems.append(f"{name} has segments aligned to {sorted(set(alignments))}, below 16 KB")
            else:
                print(f"{name}: stored, aligned (segments {sorted(set(hex(a) for a in alignments))})")

        wallpapers = [i for i in apk.infolist() if i.filename.startswith("assets/wallpaper/")]
        names = [i.filename for i in wallpapers]
        print(f"wallpapers: {len(wallpapers)} in assets/wallpaper/")
        if not wallpapers:
            problems.append("no wallpaper in assets/wallpaper/")
        for name in sorted({n for n in names if names.count(n) > 1}):
            problems.append(f"{name} appears more than once in the APK")
        for info in wallpapers:
            if info.compress_type != zipfile.ZIP_STORED:
                problems.append(f"{info.filename} is compressed (it is already QOI: store it)")
        # QOI starts with "qoif": a copy inside a native library would repeat it for every ABI.
        for name in sorted(libraries):
            # A 1920x1080 QOI header: "qoif", width 1920, height 1080.
            if b"qoif\x00\x00\x07\x80\x00\x00\x04\x38" in apk.read(name):
                problems.append(f"{name} embeds a wallpaper image")
            # An image in any other format (deflate, raw) would show up as a much bigger library.
            if libraries[name].file_size > MAX_LIBRARY_MB * 1024 * 1024:
                problems.append(f"{name} is {libraries[name].file_size / 1048576:.1f} MB, over {MAX_LIBRARY_MB} MB: it must not embed wallpapers")

        manifest = apk.read("AndroidManifest.xml")
        for permission in FORBIDDEN_PERMISSIONS:
            if manifest_has(manifest, permission):
                problems.append(f"the manifest asks for {permission}")
        if not manifest_has(manifest, "android.software.leanback"):
            problems.append("the manifest does not declare the leanback feature")
        if args.version and not manifest_has(manifest, args.version):
            problems.append(f"the manifest does not contain the version {args.version}")

    if problems:
        print("\nFAILED:")
        for problem in problems:
            print(f"  - {problem}")
        return 1
    print("\nall checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
