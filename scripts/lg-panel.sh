#!/bin/zsh
# HOME's quick panel in LG's compositor (the Settings key opens it over any app).
#
#   scripts/lg-panel.sh install      copy crates/lume-webos/panel to the TV and register the filter
#   scripts/lg-panel.sh uninstall    unregister it and remove the files (LG's list as before)
#   scripts/lg-panel.sh test KEY...  replay keys through the filter and print the panel state
#                                    (keys: settings back exit up down left right ok volup home
#                                    cursorhide cursorshow;
#                                    LG_HOLD=800 holds the last key that many ms)
#   scripts/lg-panel.sh shot FILE    screen capture (JPG) of what the TV shows
#   scripts/lg-panel.sh night FROM TO [LEVEL]   night-mode schedule, e.g. night 22:00 07:00 0
#   scripts/lg-panel.sh night off    no schedule (the panel's button still works)
#
# The filter list lives in com.webos.service.config (no system partition is touched); the first
# install keeps LG's original list in /var/lib/home-os/keyFilters.orig.json.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TV="${LG_TV:-192.168.0.185}"
key=~/.ssh/oled_root; [ -f "$key" ] || key=~/.ssh/id_ed25519
DIR=/var/lib/home-os/panel
FILTER_FILE="$DIR/home-panel.js"
CONFIG=com.webos.surfacemanager.keyFilters

tv() { ssh -tt -o BatchMode=yes -o LogLevel=ERROR -o IdentitiesOnly=yes -i "$key" "root@$TV" "$@" 2>&1 | tr -d '\r' | { grep -v -E 'post-quantum|store now|upgraded' || true; }; }
luna() { tv "luna-send -w 5000 -n 1 $1 '$2'"; }

filters_json() {
  luna luna://com.webos.service.config/getConfigs "{\"configNames\":[\"$CONFIG\"]}"
}

# Prints the setConfigs payload with our filter first (install) or without it (uninstall).
edit_filters() {
  uv run python -c '
import json, sys
mode, ours, name = sys.argv[1], sys.argv[2], sys.argv[3]
reply = json.loads(sys.stdin.read())
filters = [f for f in reply["configs"][name] if not f.get("file", "").startswith("/var/lib/home-os/panel/")]
if mode == "install":
    filters.insert(0, {"handler": "homePanelKey", "file": ours})
print(json.dumps({"configs": {name: filters}}, separators=(",", ":")))
' "$1" "$FILTER_FILE" "$CONFIG"
}

# The first Sonos soundbar on the TV's /24 (answers on port 1400 with an Arc/Beam/Ray/Playbar).
find_sonos_soundbar() {
  local net="${TV%.*}" ip
  for ip in $(for i in $(seq 1 254); do (nc -z -w1 "$net.$i" 1400 2>/dev/null && echo "$net.$i") & done; wait); do
    # Read it whole first: grep -q closing the pipe early fails the pipeline under pipefail.
    local desc="$(curl -s -m 2 "http://$ip:1400/xml/device_description.xml" || true)"
    [[ "$desc" =~ '<modelName>Sonos (Arc|Beam|Ray|Playbar|Playbase)' ]] && { echo "$ip"; return; }
  done
}

case "${1:-}" in
  install)
    current="$(filters_json)"
    # A folder per version (see homePanelDir in home-panel.js): the old ones are removed.
    version="$(cat "$ROOT"/crates/lume-webos/panel/*.js "$ROOT"/crates/lume-webos/panel/*.qml | sha256sum | cut -c1-12)"
    VDIR="$DIR/$version"
    stage="$(mktemp -d)"
    cp "$ROOT"/crates/lume-webos/panel/*.qml "$stage/"
    # The Sonos soundbar the volume slider drives over ARC (LG_SONOS=ip to skip the search).
    sonos="${LG_SONOS:-$(find_sonos_soundbar)}"
    echo "Sonos soundbar: ${sonos:-none}"
    sed -e "s|^var homePanelDir = .*|var homePanelDir = '$VDIR';|" \
        -e "s|^var homePanelSonos = .*|var homePanelSonos = '$sonos';|" \
        "$ROOT/crates/lume-webos/panel/home-panel.js" > "$stage/home-panel.js"
    tv "mkdir -p $VDIR; [ -f /var/lib/home-os/keyFilters.orig.json ] || echo '$current' > /var/lib/home-os/keyFilters.orig.json" >/dev/null
    scp -O -q -o IdentitiesOnly=yes -i "$key" \
      "$stage"/* \
      "$ROOT"/crates/lume-core/assets/fonts/PlusJakartaSans-Medium.ttf \
      "$ROOT"/crates/lume-core/assets/fonts/PlusJakartaSans-Bold.ttf \
      "$ROOT"/crates/lume-core/assets/fonts/lucide-subset.ttf \
      "$ROOT"/crates/lume-core/assets/fonts/OFL.txt \
      "$ROOT"/crates/lume-core/assets/fonts/LUCIDE-LICENSE.txt \
      "root@$TV:$VDIR/"
    rm -rf "$stage"
    FILTER_FILE="$VDIR/home-panel.js"
    # The compositor reloads the filters only when the list changes: drop ours, then add it
    # first again, so a reinstall picks up the new files.
    luna luna://com.webos.service.config/setConfigs "$(print -r -- "$current" | edit_filters uninstall)" >/dev/null
    sleep 1
    luna luna://com.webos.service.config/setConfigs "$(print -r -- "$current" | edit_filters install)"
    echo
    tv "cd $DIR && for d in *; do [ \"\$d\" = $version ] || rm -rf \"\$d\"; done" >/dev/null
    ;;
  uninstall)
    payload="$(filters_json | edit_filters uninstall)"
    luna luna://com.webos.service.config/setConfigs "$payload"
    echo
    tv "rm -rf $DIR" >/dev/null
    ;;
  test)
    shift
    n=$(date +%s%N | cut -c1-13)
    keys="$(printf '"%s",' "$@")"
    luna luna://com.webos.service.config/setConfigs \
      "{\"configs\":{\"com.webos.surfacemanager.homePanelTest\":{\"n\":$n,\"hold\":${LG_HOLD:-0},\"keys\":[${keys%,}]}}}" >/dev/null
    sleep $(( ${LG_HOLD:-0} / 1000 + 1 ))
    luna luna://com.webos.service.config/getConfigs '{"configNames":["com.webos.surfacemanager.homePanelState"]}'
    echo
    ;;
  night)
    if [ "${2:-}" = off ]; then
      cfg='{"enabled":false}'
    else
      [[ "${2:-}" =~ '^[0-2][0-9]:[0-5][0-9]$' && "${3:-}" =~ '^[0-2][0-9]:[0-5][0-9]$' ]] || { echo "night HH:MM HH:MM [LEVEL] | night off" >&2; exit 1; }
      cfg="{\"enabled\":true,\"from\":\"$2\",\"to\":\"$3\",\"level\":${4:-0}}"
    fi
    luna luna://com.webos.service.config/setConfigs "{\"configs\":{\"com.webos.surfacemanager.homePanelSchedule\":$cfg}}"
    echo
    ;;
  shot)
    luna luna://com.webos.service.capture/executeOneShot \
      '{"path":"/tmp/home-panel.jpg","method":"DISPLAY","format":"JPG","width":1280,"height":720}' >/dev/null
    scp -O -q -o IdentitiesOnly=yes -i "$key" "root@$TV:/tmp/home-panel.jpg" "${2:?file}"
    ;;
  *)
    sed -n 2,14p "$0"
    exit 1
    ;;
esac
