#!/bin/sh
# Virtual remote for the rooted LG: lgkey.sh HOME LEFT ENTER ...  (names: HOME BACK ENTER UP DOWN LEFT RIGHT ...)
# Asks the TV for its input socket as root and writes button events with the TV's own node.
keys="$*"
# The Mac has the TV's root key; other machines use their own key, added to the TV.
key=~/.ssh/oled_root; [ -f "$key" ] || key=~/.ssh/id_ed25519
ssh -tt -i "$key" -o IdentitiesOnly=yes root@192.168.0.185 "
# network-input-service accepts one client per spawn; respawn it so this connect succeeds.
killall network-input-service 2>/dev/null; sleep 1
p=\$(luna-send -w 4000 -n 1 luna://com.webos.service.networkinput/getPointerInputSocket '{}' | sed -n 's/.*\"socketPath\":\"\([^\"]*\)\".*/\1/p')
node -e '
const s=require(\"net\").connect(process.argv[1]);const k=process.argv.slice(2);let i=0;
s.on(\"error\",e=>{console.log(\"ERR\",e.message);process.exit(1)});
(function n(){if(i>=k.length){setTimeout(()=>process.exit(0),300);return}s.write(\"type:button\nname:\"+k[i++]+\"\n\n\");setTimeout(n,600)})()' \"\$p\" $keys
" 2>&1 | tr -d '\r' | grep -v "^Connection"
