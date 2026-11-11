#!/usr/bin/env bash
#
# Automation script for the Android TV emulator and Lume TV Launcher.
#
# Usage:
#   scripts/emulator.sh start [--avd NAME] [--no-window] [--gpu MODE]
#   scripts/emulator.sh stop
#   scripts/emulator.sh status
#   scripts/emulator.sh install [debug|release]
#   scripts/emulator.sh screenshot [FILE.png]
#   scripts/emulator.sh resolution [1080p|720p|WIDTHxHEIGHT] [DENSITY]
#   scripts/emulator.sh key [up|down|left|right|ok|back|home|menu]
#   scripts/emulator.sh create-720p
#

set -euo pipefail

DEFAULT_AVD="tv_api28"
DEFAULT_SERIAL="emulator-5554"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Ensure Android SDK tools are in PATH
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android}"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$ANDROID_HOME}"
export PATH="${PATH}:${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/emulator"

get_target_serial() {
    local serial="${ANDROID_SERIAL:-}"
    if [ -n "${serial}" ]; then
        echo "${serial}"
        return
    fi
    local first_emu
    first_emu="$(adb devices | awk '/emulator-[0-9]+/ {print $1; exit}')"
    if [ -n "${first_emu}" ]; then
        echo "${first_emu}"
    else
        echo "${DEFAULT_SERIAL}"
    fi
}

cmd_start() {
    local avd="${DEFAULT_AVD}"
    local gpu="host"
    local extra_args=()

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --avd)
                avd="$2"
                shift 2
                ;;
            --gpu)
                gpu="$2"
                shift 2
                ;;
            --no-window)
                extra_args+=("-no-window")
                shift
                ;;
            *)
                extra_args+=("$1")
                shift
                ;;
        esac
    done

    # Check if already running
    local serial
    serial="$(get_target_serial)"
    if adb -s "${serial}" get-state 2>/dev/null | grep -q "device"; then
        echo "Emulator already running on ${serial}."
        return 0
    fi

    echo "Starting Android TV emulator (AVD: ${avd}, GPU: ${gpu})..."
    nohup emulator -avd "${avd}" -gpu "${gpu}" "${extra_args[@]}" >/dev/null 2>&1 &
    local emu_pid=$!
    echo "Emulator process spawned (PID: ${emu_pid}). Waiting for device to boot..."

    adb wait-for-device
    serial="$(get_target_serial)"

    local count=0
    while [ "$(adb -s "${serial}" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" != "1" ]; do
        sleep 2
        count=$((count + 2))
        if [ ${count} -ge 120 ]; then
            echo "Timeout waiting for emulator boot (120s)." >&2
            exit 1
        fi
    done

    echo "Emulator ${serial} is fully booted and ready."
}

cmd_stop() {
    local serial
    serial="$(get_target_serial)"
    echo "Stopping emulator ${serial}..."
    if adb -s "${serial}" emu kill 2>/dev/null; then
        echo "Emulator stopped successfully."
    else
        echo "No running emulator found on ${serial}."
    fi
}

cmd_status() {
    local serial
    serial="$(get_target_serial)"
    if ! adb -s "${serial}" get-state 2>/dev/null | grep -q "device"; then
        echo "Emulator (${serial}) is not running."
        return 1
    fi

    echo "=== Emulator Status (${serial}) ==="
    echo "Model:      $(adb -s "${serial}" shell getprop ro.product.model | tr -d '\r')"
    echo "Android:    $(adb -s "${serial}" shell getprop ro.build.version.release | tr -d '\r') (API $(adb -s "${serial}" shell getprop ro.build.version.sdk | tr -d '\r'))"
    echo "ABI:        $(adb -s "${serial}" shell getprop ro.product.cpu.abilist | tr -d '\r')"
    echo "Resolution: $(adb -s "${serial}" shell wm size | tr -d '\r')"
    echo "Density:    $(adb -s "${serial}" shell wm density | tr -d '\r')"
    local current_home
    current_home="$(adb -s "${serial}" shell cmd package resolve-activity --brief -c android.intent.category.HOME . 2>/dev/null | tail -n 1 | tr -d '\r')"
    echo "Launcher:   ${current_home}"
}

cmd_install() {
    local variant="${1:-debug}"
    local serial
    serial="$(get_target_serial)"

    if ! adb -s "${serial}" get-state 2>/dev/null | grep -q "device"; then
        echo "Error: emulator ${serial} is not running. Run 'scripts/emulator.sh start' first." >&2
        exit 1
    fi

    local apk_path=""
    # Find newest matching APK in dist/
    if [ "${variant}" = "debug" ]; then
        apk_path="$(find "${REPO_ROOT}/dist" -name "*debug.apk" -type f -printf '%T@ %p\n' 2>/dev/null | sort -k1 -nr | head -n 1 | cut -d' ' -f2-)"
    else
        apk_path="$(find "${REPO_ROOT}/dist" -name "*release*.apk" -type f -printf '%T@ %p\n' 2>/dev/null | sort -k1 -nr | head -n 1 | cut -d' ' -f2-)"
    fi

    if [ -z "${apk_path}" ] || [ ! -f "${apk_path}" ]; then
        echo "APK not found in dist/. Building ${variant} APK..."
        "${REPO_ROOT}/scripts/container-build.sh" "${variant}"
        if [ "${variant}" = "debug" ]; then
            apk_path="$(find "${REPO_ROOT}/dist" -name "*debug.apk" -type f -printf '%T@ %p\n' 2>/dev/null | sort -k1 -nr | head -n 1 | cut -d' ' -f2-)"
        else
            apk_path="$(find "${REPO_ROOT}/dist" -name "*release*.apk" -type f -printf '%T@ %p\n' 2>/dev/null | sort -k1 -nr | head -n 1 | cut -d' ' -f2-)"
        fi
    fi

    echo "Installing ${apk_path} on ${serial}..."
    adb -s "${serial}" install -r "${apk_path}"
    echo "Setting Lume as default launcher..."
    adb -s "${serial}" shell cmd package set-home-activity app.lume.tv/.LumeActivity
    echo "Launching LumeActivity..."
    adb -s "${serial}" shell am start -n app.lume.tv/.LumeActivity
    echo "Done."
}

cmd_screenshot() {
    local out_file="${1:-emulator_screenshot.png}"
    local serial
    serial="$(get_target_serial)"
    echo "Capturing screenshot to ${out_file}..."
    adb -s "${serial}" exec-out screencap -p > "${out_file}"
    echo "Saved: ${out_file}"
}

cmd_resolution() {
    local res="${1:-reset}"
    local density="${2:-}"
    local serial
    serial="$(get_target_serial)"

    case "${res}" in
        reset)
            echo "Resetting resolution and density..."
            adb -s "${serial}" shell wm size reset
            adb -s "${serial}" shell wm density reset
            ;;
        1080p|1920x1080)
            echo "Setting resolution to 1920x1080..."
            adb -s "${serial}" shell wm size 1920x1080
            adb -s "${serial}" shell wm density "${density:-320}"
            ;;
        720p|1280x720)
            echo "Setting resolution to 1280x720 (AIWA profile)..."
            adb -s "${serial}" shell wm size 1280x720
            adb -s "${serial}" shell wm density "${density:-213}"
            ;;
        *)
            echo "Setting resolution to ${res}..."
            adb -s "${serial}" shell wm size "${res}"
            if [ -n "${density}" ]; then
                adb -s "${serial}" shell wm density "${density}"
            fi
            ;;
    esac
    echo "Current settings:"
    adb -s "${serial}" shell wm size
    adb -s "${serial}" shell wm density
}

cmd_key() {
    local key="${1:-}"
    local serial
    serial="$(get_target_serial)"
    local keycode=""

    case "${key}" in
        up)      keycode=19 ;; # DPAD_UP
        down)    keycode=20 ;; # DPAD_DOWN
        left)    keycode=21 ;; # DPAD_LEFT
        right)   keycode=22 ;; # DPAD_RIGHT
        ok|enter|select) keycode=23 ;; # DPAD_CENTER
        back)    keycode=4  ;; # BACK
        home)    keycode=3  ;; # HOME
        menu)    keycode=82 ;; # KEYCODE_MENU
        bookmark) keycode=174 ;; # KEYCODE_BOOKMARK (AIWA menu)
        *)
            keycode="${key}"
            ;;
    esac

    if [ -z "${keycode}" ]; then
        echo "Usage: scripts/emulator.sh key [up|down|left|right|ok|back|home|menu|KEYCODE]"
        exit 1
    fi

    adb -s "${serial}" shell input keyevent "${keycode}"
}

cmd_create_720p() {
    local name="tv_api28_720p"
    echo "Creating Android TV 720p AVD (${name})..."
    echo "no" | avdmanager create avd -n "${name}" -k "system-images;android-28;android-tv;x86" --device "tv_720p" --force
    echo "AVD ${name} created. Start it with: scripts/emulator.sh start --avd ${name}"
}

usage() {
    cat <<EOF
Lume Android TV Emulator Automation Tool

Commands:
  start [--avd NAME] [--gpu MODE] [--no-window]  Start emulator and wait for boot
  stop                                           Stop running emulator instance
  status                                         Show emulator status, API level, and resolution
  install [debug|release]                        Install APK and set Lume as home launcher
  screenshot [FILE.png]                          Capture PNG screenshot from emulator
  resolution [1080p|720p|reset|WxH] [DENSITY]    Change display resolution on-the-fly
  key [up|down|left|right|ok|back|home|menu]     Simulate remote control keypress
  create-720p                                    Create a 720p AVD (AIWA test profile)
EOF
}

case "${1:-help}" in
    start)
        shift
        cmd_start "$@"
        ;;
    stop)
        shift
        cmd_stop "$@"
        ;;
    status)
        shift
        cmd_status "$@"
        ;;
    install)
        shift
        cmd_install "$@"
        ;;
    screenshot)
        shift
        cmd_screenshot "$@"
        ;;
    resolution)
        shift
        cmd_resolution "$@"
        ;;
    key)
        shift
        cmd_key "$@"
        ;;
    create-720p)
        shift
        cmd_create_720p "$@"
        ;;
    *)
        usage
        ;;
esac
