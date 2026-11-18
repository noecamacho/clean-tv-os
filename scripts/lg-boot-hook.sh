#!/bin/sh
# HOME OS boot tuning for the LG OLED55C1PSA (webOS 6.5.3, rooted).
# Runs as a Homebrew Channel init.d hook, about 36 s into a cold boot.
#
# Install: scp -O scripts/lg-boot-hook.sh root@TV:/var/lib/webosbrew/init.d/60-home-os
#          ssh root@TV chmod 755 /var/lib/webosbrew/init.d/60-home-os
# Undo:    rm /var/lib/webosbrew/init.d/60-home-os, then reboot.
# Details and the one-off settings that go with it: docs/llm-wiki/lg-webos.md, «Ajuste de arranque».
(
    # The hook can be invoked again in the same boot; act once.
    [ -f /tmp/home-os-boot-done ] && exit 0
    touch /tmp/home-os-boot-done
    echo "start $(cut -d' ' -f1 /proc/uptime)"
    LS="luna-send -n 1 -w 5000"
    # Quick Start+ reboots the TV in the background after power-off (reason quickStart) to
    # reach its standby; a screen-less boot is left to LG. One line per boot, kept short.
    reason=$($LS luna://com.webos.service.tvpower/power/getPowerOnReason '{}' | sed -n 's/.*"reason": *"\([^"]*\)".*/\1/p')
    state=$($LS luna://com.webos.service.tvpower/power/getPowerState '{}' | sed -n 's/.*"state": *"\([^"]*\)".*/\1/p')
    H=/var/lib/home-os/boot-history.log
    mkdir -p /var/lib/home-os; tail -n 40 $H > $H.tmp 2>/dev/null; mv $H.tmp $H
    echo "$(date '+%F %T') up=$(cut -d' ' -f1 /proc/uptime) reason=$reason state=$state" >> $H
    echo "reason=$reason state=$state"
    [ "$state" = Active ] || exit 0

    # No antenna: broadcast stack and EPG, plus LG helpers nobody uses here.
    # --no-block because some of them take long to stop.
    # utp stays: tvpowerd asks utp.broadcast/getWarmOffStatus at power-off (Quick Start+).
    systemctl stop --no-block legacy-broadcast.service iepg.service \
        contentminer.service uploadd.service nudge.service

    # LG preloads these 60-80 s after boot; preloadmanager forgets the policy on reboot,
    # so it is set again each time. They still open on demand.
    # preloadmanager may not answer yet when the hook starts: wait up to ~60 s.
    i=0
    until $LS luna://com.webos.service.preloadmanager/getPreloadPolicy '{}' | grep -q '"returnValue":true'; do
        i=$((i+1)); [ $i -gt 30 ] && break; sleep 2
    done
    echo "preloadmanager ready $(cut -d' ' -f1 /proc/uptime)"
    for id in com.webos.app.lgchannels com.webos.app.browser com.webos.app.voice \
              google.assistant amazon.alexapr amazon \
              com.webos.app.livetv com.webos.app.livemenu com.webos.app.miracast; do
        echo "$id $($LS luna://com.webos.service.preloadmanager/setPreloadPolicy \
            "{\"application\":{\"id\":\"$id\",\"isEnabled\":false}}")"
        # If it was preloaded before the policy changed, free it now.
        $LS luna://com.webos.applicationManager/closeByAppId "{\"id\":\"$id\"}" >/dev/null
    done

    echo "done $(cut -d' ' -f1 /proc/uptime)"
) </dev/null >/tmp/home-os-boot.log 2>&1 &

