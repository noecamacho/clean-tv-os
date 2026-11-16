#!/usr/bin/env bash
# Compiles the Vulkan shaders (crates/lume-app/shaders/*.vert|frag) to the SPIR-V files next to
# them, which the app embeds. Run it after editing a shader; the .spv files are committed.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
docker run --rm -v "$PWD:/src" -w /src/crates/lume-app/shaders lume-toolchain:latest sh -c '
  glslc="$ANDROID_NDK_HOME/shader-tools/linux-x86_64/glslc"
  for f in *.vert *.frag; do "$glslc" --target-env=vulkan1.1 -O "$f" -o "$f.spv" || exit 1; done
  ls -l *.spv'
