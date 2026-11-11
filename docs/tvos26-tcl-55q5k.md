# Personal tvOS 26-inspired build on TCL 55Q5K

This variant is an original Rust/GLES2 implementation inspired by Apple's public tvOS 26 image. The three beach scenes and HOME icon are generated for this project. It does not contain Apple UI artwork or system code. The glass effect uses one wallpaper tap for edge-weighted refraction and translucent layers; it is not Apple's private Liquid Glass renderer.

## Device and artifact

- TCL 55Q5K, Android 12 / API 31, `armeabi-v7a`, Realtek rtd6748.
- Release APK: `dist/lume-0.3.9-home-controls-release.apk` (31.08 MB).
- User locale on the tested television: `es-ES`.
- The signed release uses a local key kept outside this repository in `~/Projects/.clean-tv-os-private/signing/lume-tvos-local.jks`. Keep that key and its `credentials.env` file for future APK updates. Never commit them.

## Build and install

From the repository root:

```sh
set -a
source ~/Projects/.clean-tv-os-private/signing/credentials.env
set +a
LUME_VERSION_NAME=v0.3.9-home-controls scripts/container-build.sh release
uv run python scripts/verify-apk.py dist/lume-0.3.9-home-controls-release.apk
adb connect 192.168.0.229:5555
adb install -r dist/lume-0.3.9-home-controls-release.apk
adb shell am start -n app.lume.tv/.LumeActivity
```

The build uses Docker Buildx and requires the local signing files. `adb install -r` works for subsequent builds signed with that key. An earlier debug-signed install cannot update in place; its APK was backed up at `~/Projects/.clean-tv-os-private/baseline/base.apk` before this release was installed.

## Checks on 2026-11-11

- `cargo test -p lume-core`: 508 tests run: 506 passed, 2 ignored; final Android release build passed.
- APK verification: 9 QOI wallpapers; ARMv7 and ARM64 libraries; 31.08 MB.
- First frame captured at `docs/llm-wiki/images/home-tcl.png`: Beach Cove, six app banners, refractive glass shelf and compact clock.
- Up reveals navigation. Android Settings opens from the gear.
- System Home still resolves to `com.google.android.apps.tv.launcherx/.home.HomeActivity` and the Home key returns to Google TV.
- Inputs opens TCL's source panel. The panel listed Google TV Home, AirPlay and TV during this check; no HDMI source was available to exercise an HDMI switch.
- The glass top bar includes a Control Center button. Its right-side panel has Wi-Fi, Bluetooth, Inputs, Picture, Sound, Ambient mode, Apps and Settings cards. Wi-Fi opens Network, Bluetooth opens Add Accessory, Picture opens Display and Sound, Ambient opens the TV's ambient-mode screen, and Inputs opens the TCL source panel. Sound uses the same Display and Sound destination on this TCL firmware.
- Final panel screenshot: `docs/llm-wiki/images/control-center-tcl.png`. The Spanish Ambient mode label fits without ellipsis.
- Panel focus navigation on the television measured 16.6 ms median and 18.0 ms 95th percentile among active SurfaceFlinger frame intervals during eight right/left cycles.
- Foreground memory after loading and navigating: approximately 21.5–27.8 MB total PSS.

## Rollback

To remove only this personal Lume build:

```sh
adb uninstall app.lume.tv
```

Google TV is still the configured Home launcher, so no system package needs to be re-enabled. To reinstall the previous Lume version, run `adb install ~/Projects/.clean-tv-os-private/baseline/base.apk` after uninstalling HOME. Reinstalling across signing keys resets Lume's private preferences.

## Focus fluidity on TCL 55Q5K

The 1920×1080 build presented rapid focus animation frames at a median interval of about 37 ms. The final HOME build renders its 1280×720 logical canvas directly, uses one wallpaper sample in the glass shader, and moves focus over 180 ms with a decelerating curve. On the same television and a repeated left/right input sequence, SurfaceFlinger reported a 16.8 ms median and 19.5 ms 95th percentile among active frame intervals. This trades some static sharpness for much smoother movement; the source beach assets remain 1920×1080 in the APK. The TV display setting was not changed.
