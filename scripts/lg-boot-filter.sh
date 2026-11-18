#!/bin/zsh
# HOME's early-boot compositor filter on the rooted LG (opens HOME at minimal-boot-done, ~18 s).
#
#   scripts/lg-boot-filter.sh install     copy crates/lume-webos/boot/home-boot.js and register it
#   scripts/lg-boot-filter.sh uninstall   unregister it and remove the file
#
# The filter goes right after homePanelKey (the quick panel stays first; see scripts/lg-panel.sh),
# or first if the panel is not installed. Only com.webos.surfacemanager.keyFilters changes; no
# system partition is touched. Details: docs/llm-wiki/lg-webos.md, «Arranque más rápido».
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TV="${LG_TV:-192.168.0.185}"
key=~/.ssh/oled_root; [ -f "$key" ] || key=~/.ssh/id_ed25519
DIR=/var/lib/home-os/boot
CONFIG=com.webos.surfacemanager.keyFilters

tv() { ssh -tt -o BatchMode=yes -o LogLevel=ERROR -o IdentitiesOnly=yes -i "$key" "root@$TV" "$@" 2>&1 | tr -d '\r' | { grep -v -E 'post-quantum|store now|upgraded' || true; }; }
luna() { tv "luna-send -w 5000 -n 1 $1 '$2'"; }

edit_filters() {
  uv run python -c '
import json, sys
mode, ours, name = sys.argv[1], sys.argv[2], sys.argv[3]
reply = json.loads(sys.stdin.read())
filters = [f for f in reply["configs"][name] if not f.get("file", "").startswith("/var/lib/home-os/boot/")]
if mode == "install":
    at = next((i + 1 for i, f in enumerate(filters) if f.get("handler") == "homePanelKey"), 0)
    filters.insert(at, {"handler": "homeBootKey", "file": ours})
print(json.dumps({"configs": {name: filters}}, separators=(",", ":")))
' "$1" "$DIR/home-boot.js" "$CONFIG"
}

current() { luna luna://com.webos.service.config/getConfigs "{\"configNames\":[\"$CONFIG\"]}"; }

case "${1:-}" in
  install)
    tv "mkdir -p $DIR" >/dev/null
    scp -O -q -o IdentitiesOnly=yes -i "$key" "$ROOT/crates/lume-webos/boot/home-boot.js" "root@$TV:$DIR/"
    luna luna://com.webos.service.config/setConfigs "$(current | edit_filters install)"; echo
    ;;
  uninstall)
    luna luna://com.webos.service.config/setConfigs "$(current | edit_filters uninstall)"; echo
    tv "rm -rf $DIR" >/dev/null
    ;;
  *)
    sed -n 2,9p "$0"; exit 1
    ;;
esac
