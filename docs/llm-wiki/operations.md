# TCL 55Q5K · compilación y operaciones

## Equipo probado

TCL 55Q5K, Google TV con Android 14/API 34 desde la actualización del 2026-11-15 (antes Android 12/API 31), ABI `armeabi-v7a`, SoC Realtek rtd6748, GPU PowerVR BXE-4-32 (OpenGL ES 3.2, Vulkan 1.1, `EGL_KHR_partial_update`), pantalla 4K con UI a 1920×1080, plataforma G10, firmware `V8-0008T01-LF1V521`, LAN `192.168.0.229`. La IP puede cambiar. El launcher del sistema es `com.google.android.apps.tv.launcherx/.home.HomeActivity`; HOME es `app.lume.tv/.LumeActivity`.

## Datos privados fuera del repositorio

`~/Projects/.clean-tv-os-private/signing/credentials.env` contiene variables de firma y señala el JKS privado en la misma carpeta. `~/Projects/.clean-tv-os-private/baseline/base.apk` es el APK anterior a esta variante. No añadas ninguno a Git ni pegues sus contenidos en una respuesta. La firma permite `adb install -r` sin borrar datos; instalar un APK firmado con otra clave sobre `app.lume.tv` falla o requiere desinstalar y perder preferencias.

## Construir y verificar

```sh
cd ~/Projects/clean-tv-os
docker run --rm -v "$PWD:/src" -w /src lume-toolchain:latest sh -lc 'cargo fmt --all && cargo test -q -p lume-core'
set -a; source ~/Projects/.clean-tv-os-private/signing/credentials.env; set +a
LUME_VERSION_NAME=v0.9.7-tvos scripts/container-build.sh release
uv run python scripts/verify-apk.py dist/lume-0.9.7-tvos-release.apk
```

Para un cambio nuevo, aumenta la versión por encima de 907 en `LUME_VERSION_NAME`. El script empaqueta ARMv7 y ARM64 y genera APK/AAB en `dist/`. Comprueba que muestra firma de release; sin `LUME_KEYSTORE` produce un APK firmado con la clave de debug, que no actualiza la instalación actual. `dist/` está ignorado por Git: el APK final se copió localmente, pero otros agentes deben reconstruirlo si clonan el repositorio.

## ADB y pruebas manuales

```sh
adb connect 192.168.0.229:5555
adb -s 192.168.0.229:5555 devices
adb -s 192.168.0.229:5555 install -r dist/lume-0.9.7-tvos-release.apk
adb -s 192.168.0.229:5555 shell am start -n app.lume.tv/.LumeActivity
adb -s 192.168.0.229:5555 shell cmd package resolve-activity --brief -a android.intent.action.MAIN -c android.intent.category.HOME
adb -s 192.168.0.229:5555 exec-out screencap -p > /tmp/home-check.png
```

En HOME, pulsa Arriba para mostrar la barra, desplázate hasta el icono de Centro de control y pulsa OK. Comprueba navegación izquierda/derecha/arriba/abajo, Back, Wi-Fi, Bluetooth, Imagen/Sonido, Entradas y Ajustes. El botón Home del control debe abrir Google TV. En la prueba anterior, Wi-Fi abrió `NetworkActivity`, Bluetooth `AddAccessoryActivity`, Imagen/Sonido `DisplaySoundActivity`, Modo ambiente `DaydreamActivity` y Entradas el panel TCL. No había fuente HDMI activa; un cambio real de HDMI sigue sin probarse.

## Botón Home (HOME como launcher)

`cmd package set-home-activity app.lume.tv/.LumeActivity` devuelve Success pero esta TCL lo ignora: Home sigue abriendo Google TV. Desde 0.3.10, `HomeKeyService` (servicio de accesibilidad que solo filtra teclas) abre HOME con el botón Home, al encender la pantalla y tras el arranque. TCL bloquea el enlace del servicio sin el app-op de autoarranque.

```sh
# Activar HOME como inicio
adb -s 192.168.0.229:5555 shell appops set app.lume.tv AUTO_START allow   # APP_AUTO_START en el firmware Android 12
adb -s 192.168.0.229:5555 shell settings put secure enabled_accessibility_services app.lume.tv/app.lume.tv.HomeKeyService
adb -s 192.168.0.229:5555 shell settings put secure accessibility_enabled 1
# Volver a Google TV
adb -s 192.168.0.229:5555 shell settings put secure enabled_accessibility_services null
```

También se puede apagar en Ajustes > Sistema > Accesibilidad > HOME. `adb shell input keyevent KEYCODE_HOME` **no** pasa por el filtro de accesibilidad y siempre abre Google TV; para probar sin el mando usa `adb shell uinput` con un dispositivo virtual (KEY_HOMEPAGE = 172). La actualización de firmware a Android 14 (`UTT2.250416.001`, 2026-11-15) borró la autorización ADB de la Mac y dejó el app-op, ahora llamado `AUTO_START`, en `ignore`; hubo que aceptar de nuevo la depuración en la tele y repetir el primer comando. Apps deshabilitadas, DNS privado y app-ops de `com.tcl.ttvs` sobrevivieron. Comprobado el 2026-11-11 y el 2026-11-15 con uinput; encendido desde reposo y reinicio en frío sin comprobar, y el firmware puede desactivar el servicio por su cuenta.

## Brillo, suspensión y animaciones del sistema (2026-11-15)

A petición del usuario, para el Centro de control y la fluidez general:

```sh
# Respaldo para otras marcas. En esta TCL ese ajuste NO mueve el panel: HOME usa TvBacklightManager de com.tcl.tv.display (véase tcl-apis.md)
adb -s 192.168.0.229:5555 shell appops set app.lume.tv WRITE_SETTINGS allow
# Transiciones de Android a la mitad. No pongas animator_duration_scale a 0: HOME lo lee como «reducir movimiento»
adb -s 192.168.0.229:5555 shell settings put global window_animation_scale 0.5
adb -s 192.168.0.229:5555 shell settings put global transition_animation_scale 0.5
adb -s 192.168.0.229:5555 shell settings put global animator_duration_scale 0.5
# Deshacer
adb -s 192.168.0.229:5555 shell appops set app.lume.tv WRITE_SETTINGS default
for k in window_animation_scale transition_animation_scale animator_duration_scale; do adb -s 192.168.0.229:5555 shell settings put global $k 1.0; done
```

«Suspender» y el temporizador usan `performGlobalAction(GLOBAL_ACTION_LOCK_SCREEN)` de `HomeKeyService`: solo funcionan con ese servicio activado. Comprobado: la tele pasa a `Asleep` y `KEYCODE_WAKEUP` devuelve HOME. Pantallas que abre el Centro de control: AirPlay (`com.realtek.media.airplay2.LAUNCH_SETTINGS`), Miracast (`com.tcl.miracast.test.MAINMENU`), imagen (`android.settings.DISPLAY_SETTINGS` → `com.tcl.settings.PictureActivity`), sonido (`DisplaySoundActivity`). No son escribibles por una app: modo de imagen y de sonido (claves TCL), encender/apagar Wi‑Fi, conectar auriculares Bluetooth (API privilegiada). La tele no expone sensor de luz a Android: el brillo automático solo existe en el panel de imagen de TCL.

Para medir: `scratchpad/measure.sh` descarta intervalos de más de 100 ms (las pausas entre animaciones no son cuadros lentos).

## Apps de fábrica deshabilitadas (2026-11-11)

A petición del usuario se deshabilitaron para el usuario 0, con `pm disable-user --user 0`, veinte paquetes (el usuario volvió a habilitar después Prime Video; quedan 19) de promoción, demostración o preinstalados. Nada se borra de la partición del sistema. Comprobado después: Ajustes, Imagen/Sonido, panel de Entradas y HOME abren sin cierres en el registro. **Sin comprobar:** reinicio de la tele (el puerto de ADB por red no es persistente, `persist.adb.tcp.port` está vacío, así que no se reinició por ADB) y una fuente HDMI real.

```sh
# Volver a habilitar todo
for p in com.tcl.waterfall.overseas com.tcl.channelplus com.tcl.dashboard com.tcl.messagebox \
  com.tcl.usercenter com.tcl.tv.tclhome_passive com.tcl.interactive com.tcl.exhibit \
  com.tcl.esticker com.tcl.hotelmenu com.tcl.repairguide com.tcl.ocean.instructions \
  com.tcl.logkit com.tcl.smartalexa com.tcl.t_solo com.tcl.hearaid \
  com.amazon.amazonvideo.livingroom com.google.android.youtube.tvmusic \
  com.google.android.videos com.google.android.play.games; do
  adb -s 192.168.0.229:5555 shell pm enable $p
done
```

No tocar: `com.tcl.tvinput`, `com.tcl.tv`, `com.tcl.systemserver`, `com.tcl.providers.config`, `com.realtek.*`, `com.tcl.autopair`, `com.tcl.UpdatePeripheral`, `com.tcl.inputmethod.international`, `com.tcl.ttvs`, `com.tcl.guard`, `com.tcl.suspension`, `com.tcl.android.webview`. Si se deshabilita `com.google.android.apps.tv.launcherx`, el inicio pasaría a `com.google.android.tungsten.setupwraith/.RecoveryActivity` (prioridad 1) y no a HOME (prioridad 0): probarlo con el comando de vuelta preparado.

## Privacidad (2026-11-11)

A petición del usuario:

- **DNS privado con filtro de anuncios y rastreo** (AdGuard). Comprobado: `google.com` resuelve y `doubleclick.net` queda bloqueado.
- **`com.tcl.ttvs` sin estadísticas de uso, ubicación ni micrófono** (app-ops en `ignore`). Ese paquete mezcla telemetría (`BIService`, `TrackDataService`, `UsagestService`, Samba TV) con actualización del sistema, envío de pantalla y ajustes del menú, así que no se deshabilita entero. `pm disable` de sus componentes falla («Shell cannot change component state»). **No hagas `am force-stop com.tcl.ttvs`**: no vuelve a arrancar solo; se recupera con `am startservice -n com.tcl.ttvs/.CoreService`.

```sh
# Deshacer
adb -s 192.168.0.229:5555 shell settings put global private_dns_mode opportunistic
adb -s 192.168.0.229:5555 shell settings delete global private_dns_specifier
for op in GET_USAGE_STATS FINE_LOCATION COARSE_LOCATION RECORD_AUDIO; do
  adb -s 192.168.0.229:5555 shell appops set com.tcl.ttvs $op default
done
```

## Rollback

La acción menos invasiva es pulsar Home: Google TV ya es el launcher predeterminado. Para retirar HOME, usa `adb -s 192.168.0.229:5555 uninstall app.lume.tv`; no hay que reactivar paquetes. El APK anterior está en `~/Projects/.clean-tv-os-private/baseline/base.apk`, pero tiene otra firma: para instalarlo hay que desinstalar HOME y se pierden sus preferencias. No deshabilites Google TV ni componentes TCL.
