#!/bin/zsh
# Builds HOME for the LG webOS TV and packages the .ipk.
#
# Needs: docker (image lume-toolchain:latest), ares-package (webOS CLI), and the webOS NDK
# unpacked at ~/Projects/.clean-tv-os-private/webos-ndk/arm-webos-linux-gnueabi_sdk-buildroot
# (linux-aarch64 build of webosbrew/native-toolchain, release webos-d7ed7ee.6).
#
# The SDL-webOS runtime (the TV needs SDL's Wayland backend; the NDK's SDL is a link stub)
# is downloaded once and verified against Luma's pinned SHA-256 hashes.
#
# Output: dist/app.home.tv_<version>_arm.ipk
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PRIVATE="$HOME/Projects/.clean-tv-os-private"
NDK="$PRIVATE/webos-ndk/arm-webos-linux-gnueabi_sdk-buildroot"
SDL_DIR="$PRIVATE/webos-sdl"
IMAGE=lume-webos-toolchain:latest
TARGET=armv7-unknown-linux-gnueabi

SDL_RELEASE=release-2.30.12-webos.5
SDL_ARCHIVE_SHA=4ad566453d113bdd9ee96878176b97d28d9fa70503a62d83d55a351545abb334
SDL_LIB_SHA=f564f7da624cec47168300d017f3ec822f41b5096f0ce1c96873eecd8614d446

[ -d "$NDK" ] || { echo "webOS NDK not found at $NDK" >&2; exit 1; }

# --- SDL-webOS runtime (bundled in the ipk as lib/libSDL2-2.0.so.0) ---
mkdir -p "$SDL_DIR"
if [ ! -f "$SDL_DIR/libSDL2-2.0.so.0" ] \
   || [ "$(shasum -a 256 "$SDL_DIR/libSDL2-2.0.so.0" | cut -d' ' -f1)" != "$SDL_LIB_SHA" ]; then
  archive="$SDL_DIR/SDL2-2.30.12-webos.tar.gz"
  if [ ! -f "$archive" ] \
     || [ "$(shasum -a 256 "$archive" | cut -d' ' -f1)" != "$SDL_ARCHIVE_SHA" ]; then
    echo "Downloading SDL-webOS $SDL_RELEASE"
    curl -fL -o "$archive" \
      "https://github.com/webosbrew/SDL-webOS/releases/download/$SDL_RELEASE/SDL2-2.30.12-webos.tar.gz"
  fi
  echo "$SDL_ARCHIVE_SHA  $archive" | shasum -a 256 -c - >/dev/null
  tar -xzf "$archive" -C "$SDL_DIR" --include '*/libSDL2-2.0.so.0.*' 2>/dev/null || true
  found="$(find "$SDL_DIR/lib" -name 'libSDL2-2.0.so.0.*' -type f | head -1)"
  [ -n "$found" ] || { echo "libSDL2-2.0.so.0.* not in the archive" >&2; exit 1; }
  cp "$found" "$SDL_DIR/libSDL2-2.0.so.0"
  echo "$SDL_LIB_SHA  $SDL_DIR/libSDL2-2.0.so.0" | shasum -a 256 -c - >/dev/null
fi

# --- Toolchain image (lume-toolchain + the ARM gnueabi Rust target) ---
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  docker build -t "$IMAGE" -f "$ROOT/docker/Dockerfile.webos" "$ROOT"
fi

# --- Relocate the NDK to its path inside the container (idempotent) ---
docker run --rm -v "$PRIVATE/webos-ndk:/opt/webos-ndk" "$IMAGE" \
  sh -c '/opt/webos-ndk/arm-webos-linux-gnueabi_sdk-buildroot/relocate-sdk.sh'

# --- Cross-build (the target directory stays in a volume: bind mounts are slow) ---
mkdir -p "$ROOT/dist/webos"
docker run --rm \
  -v "$ROOT:/src" -w /src \
  -v lume-cargo-registry:/opt/cargo/registry \
  -v lume-webos-target:/target \
  -v "$PRIVATE/webos-ndk:/opt/webos-ndk" \
  "$IMAGE" sh -lc '
set -e
NDK=/opt/webos-ndk/arm-webos-linux-gnueabi_sdk-buildroot
CC="$NDK/bin/arm-webos-linux-gnueabi-gcc"
SYSROOT="$NDK/arm-webos-linux-gnueabi/sysroot"
export CARGO_TARGET_DIR=/target
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_LINKER="$CC"
# Soft-float calling convention with the NDK Cortex-A9/VFP baseline, as Luma builds
# (see lg-luma tools/build-webos.sh); NEON off for portable scalar Rust.
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C target-cpu=cortex-a9 -C target-feature=-neon,-soft-float -C link-arg=--sysroot=$SYSROOT -C link-arg=-Wl,-rpath,\$ORIGIN/lib"
export CC_armv7_unknown_linux_gnueabi="$CC"
export AR_armv7_unknown_linux_gnueabi="$NDK/bin/arm-webos-linux-gnueabi-ar"
export CFLAGS_armv7_unknown_linux_gnueabi="--sysroot=$SYSROOT"
cargo build --release --target armv7-unknown-linux-gnueabi -p lume-webos
cp /target/armv7-unknown-linux-gnueabi/release/home /src/dist/webos/home
'

# --- Stage and package the ipk ---
STAGE="$ROOT/dist/webos/app.home.tv"
rm -rf "$STAGE"
mkdir -p "$STAGE/lib" "$STAGE/licenses"
cp "$ROOT/crates/lume-webos/webos/appinfo.json" "$STAGE/"
cp "$ROOT/crates/lume-webos/webos/app-icon.png" "$STAGE/"
cp "$ROOT/crates/lume-webos/webos/app-icon-large.png" "$STAGE/"
cp "$ROOT/dist/webos/home" "$STAGE/home"
chmod +x "$STAGE/home"
cp "$SDL_DIR/libSDL2-2.0.so.0" "$STAGE/lib/"
cat > "$STAGE/licenses/README.md" <<'EOF'
- lib/libSDL2-2.0.so.0: SDL-webOS (github.com/webosbrew/SDL-webOS, zlib license),
  prebuilt release-2.30.12-webos.5, bundled unmodified.
- home: HOME (this repository), GPL-3.0-or-later.
EOF
ares-package --no-minify -o "$ROOT/dist" "$STAGE"
ls -la "$ROOT"/dist/*.ipk
