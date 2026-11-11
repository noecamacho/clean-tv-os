#!/usr/bin/env bash
# Builds lume inside a container, so no Rust, NDK or Android SDK is needed on the host.
#
# Usage: scripts/container-build.sh [debug|release]
#
# Runtime selection: $LUME_CONTAINER (a command) always wins; otherwise the order is the priority:
# the first one found of wslc (WSL), docker and podman, in that order, is used.
# Results land in dist/ (APK, and AAB for release).
set -euo pipefail

variant="${1:-debug}"
case "$variant" in
  debug | release) ;;
  *) echo "usage: $0 [debug|release]" >&2; exit 2 ;;
esac

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"
mkdir -p dist
tarball="dist/artifacts-$variant.tar"

# The build version (platform-compatibility spec). The container has no .git, so it is resolved
# here: LUME_VERSION_NAME without its leading "v"s (a release tag), else the short commit
# (+ "-dirty" with uncommitted changes to tracked files), else "dev". LUME_VERSION_CODE is the
# commit count, used for anything that is not a release.
version="$(printf '%s' "${LUME_VERSION_NAME:-}" | sed 's/^v*//')"
version_code="${LUME_VERSION_CODE:-}"
if [[ -z "$version" ]]; then
  if hash="$(git rev-parse --short HEAD 2>/dev/null)"; then
    version="$hash"
    if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then version="$hash-dirty"; fi
    [[ -n "$version_code" ]] || version_code="$(git rev-list --count HEAD)"
  else
    version="dev"
  fi
fi
echo "Version: $version"

# Limits so the fat-LTO build fits in the ~7 GB a default WSL VM has.
jobs="${CARGO_BUILD_JOBS:-4}"
heap="${GRADLE_HEAP:-1536m}"

# Sets `runtime_cmd`, the command (as separate words, so a path with spaces survives) that runs
# "build". The order is the priority: wslc, docker, podman.
find_runtime() {
  if [[ -n "${LUME_CONTAINER:-}" ]]; then
    # shellcheck disable=SC2206 # a command with arguments, split on spaces on purpose
    runtime_cmd=($LUME_CONTAINER)
  elif command -v wslc.exe >/dev/null 2>&1; then
    runtime_cmd=("wslc.exe")
  elif [[ -x "/mnt/c/Program Files/WSL/wslc.exe" ]]; then
    runtime_cmd=("/mnt/c/Program Files/WSL/wslc.exe")
  elif command -v docker >/dev/null 2>&1; then
    runtime_cmd=("docker" "buildx")
  elif command -v podman >/dev/null 2>&1; then
    runtime_cmd=("podman")
  else
    cat >&2 <<'MSG'
error: no container runtime found (looked for wslc, docker and podman, in that order).
Install one of them, or pick another way to build:
  - Windows: update WSL (wsl --update) to get wslc, or install Docker Desktop.
  - Linux/macOS: install Docker Engine or Podman (podman 5 or newer, for --output).
  - Any other OCI runtime with a docker-compatible "build" command: set LUME_CONTAINER.
  - No containers at all: install Rust, the Android NDK/SDK and cargo-ndk and follow
    docker/Dockerfile step by step on your machine.
MSG
    exit 1
  fi
}

find_runtime
runtime="${runtime_cmd[*]}"
echo "Using container runtime: $runtime"

# wslc.exe is a Windows program: it needs Windows paths.
path_for_runtime() {
  if [[ "${runtime_cmd[0]}" == *wslc.exe ]]; then wslpath -w "$1"; else echo "$1"; fi
}

# Release signing: pass the keystore and its passwords as build secrets, so they are never
# written into an image layer. Set LUME_KEYSTORE (a file) and LUME_KEYSTORE_PASSWORD,
# LUME_KEY_ALIAS and LUME_KEY_PASSWORD in the environment.
secrets=()
signed=false
if [[ "$variant" == "release" && -n "${LUME_KEYSTORE:-}" ]]; then
  : "${LUME_KEYSTORE_PASSWORD:?set LUME_KEYSTORE_PASSWORD}"
  : "${LUME_KEY_ALIAS:?set LUME_KEY_ALIAS}"
  : "${LUME_KEY_PASSWORD:?set LUME_KEY_PASSWORD}"
  [[ -f "$LUME_KEYSTORE" ]] || { echo "error: $LUME_KEYSTORE does not exist" >&2; exit 1; }
  # Passwords go through short-lived files rather than environment variables: the Windows side
  # of wslc cannot see the environment of this shell.
  secret_dir="$(mktemp -d)"
  chmod 700 "$secret_dir"
  trap 'rm -rf "$secret_dir"' EXIT
  printf '%s' "$LUME_KEYSTORE_PASSWORD" > "$secret_dir/store_password"
  printf '%s' "$LUME_KEY_ALIAS" > "$secret_dir/key_alias"
  printf '%s' "$LUME_KEY_PASSWORD" > "$secret_dir/key_password"
  secrets=(
    --secret "id=lume_keystore,src=$(path_for_runtime "$(realpath "$LUME_KEYSTORE")")"
    --secret "id=lume_store_password,src=$(path_for_runtime "$secret_dir/store_password")"
    --secret "id=lume_key_alias,src=$(path_for_runtime "$secret_dir/key_alias")"
    --secret "id=lume_key_password,src=$(path_for_runtime "$secret_dir/key_password")"
  )
  signed=true
elif [[ "$variant" == "release" ]]; then
  echo "note: LUME_KEYSTORE is not set, so the release is signed with the debug key (do not publish it)"
fi

if [[ -n "${LUME_KEYS_FILE:-}" ]]; then
  [[ -f "$LUME_KEYS_FILE" ]] || { echo "error: $LUME_KEYS_FILE does not exist" >&2; exit 1; }
  secrets+=(
    --secret "id=lume_keys_file,src=$(path_for_runtime "$(realpath "$LUME_KEYS_FILE")")"
  )
fi

# wslc cannot export a directory, only a tar file; the same works for docker buildx and podman.
"${runtime_cmd[@]}" build \
  --progress plain \
  --build-arg "VARIANT=$variant" \
  --build-arg "CARGO_BUILD_JOBS=$jobs" \
  --build-arg "GRADLE_HEAP=$heap" \
  --build-arg "LUME_VERSION_NAME=$version" \
  --build-arg "LUME_VERSION_CODE=$version_code" \
  ${secrets[@]+"${secrets[@]}"} \
  --target artifacts \
  --output "type=tar,dest=$(path_for_runtime "$repo/$tarball")" \
  --file "$(path_for_runtime "$repo/docker/Dockerfile")" \
  "$(path_for_runtime "$repo")"

# The artifacts stage only contains the build outputs.
tar -xmf "$tarball" -C dist
rm -f "$tarball"

for f in dist/app-*.apk dist/app-*.aab; do
  [[ -e "$f" ]] || continue
  extension="${f##*.}"
  suffix=""
  if [[ "$variant" == "release" && "$signed" == false ]]; then suffix="-debugsigned"; fi
  target="dist/lume-$version-$variant$suffix.$extension"
  mv "$f" "$target"
  echo "Built $target"
done
ls -l dist
