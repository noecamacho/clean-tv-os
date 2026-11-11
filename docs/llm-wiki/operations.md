# TCL 55Q5K · compilación y operaciones

## Equipo probado

TCL 55Q5K, Google TV con Android 12/API 31, ABI `armeabi-v7a`, SoC Realtek rtd6748, GPU PowerVR BXE-4-32 (OpenGL ES 3.2, Vulkan 1.1, `EGL_KHR_partial_update`), pantalla 4K con UI a 1920×1080, plataforma G10, firmware `V8-0008T01-LF1V521`, LAN `192.168.0.229`. La IP puede cambiar. El launcher del sistema es `com.google.android.apps.tv.launcherx/.home.HomeActivity`; HOME es `app.lume.tv/.LumeActivity`.

## Datos privados fuera del repositorio

`~/Projects/.clean-tv-os-private/signing/credentials.env` contiene variables de firma y señala el JKS privado en la misma carpeta. `~/Projects/.clean-tv-os-private/baseline/base.apk` es el APK anterior a esta variante. No añadas ninguno a Git ni pegues sus contenidos en una respuesta. La firma permite `adb install -r` sin borrar datos; instalar un APK firmado con otra clave sobre `app.lume.tv` falla o requiere desinstalar y perder preferencias.

## Construir y verificar

```sh
cd ~/Projects/clean-tv-os
docker run --rm -v "$PWD:/src" -w /src lume-toolchain:latest sh -lc 'cargo fmt --all && cargo test -q -p lume-core'
set -a; source ~/Projects/.clean-tv-os-private/signing/credentials.env; set +a
LUME_VERSION_NAME=v0.4.0-tvos scripts/container-build.sh release
uv run python scripts/verify-apk.py dist/lume-0.4.0-tvos-release.apk
```

Para un cambio nuevo, aumenta la versión por encima de 400 en `LUME_VERSION_NAME`. El script empaqueta ARMv7 y ARM64 y genera APK/AAB en `dist/`. Comprueba que muestra firma de release; sin `LUME_KEYSTORE` produce un APK firmado con la clave de debug, que no actualiza la instalación actual. `dist/` está ignorado por Git: el APK final se copió localmente, pero otros agentes deben reconstruirlo si clonan el repositorio.

## ADB y pruebas manuales

```sh
adb connect 192.168.0.229:5555
adb -s 192.168.0.229:5555 devices
adb -s 192.168.0.229:5555 install -r dist/lume-0.4.0-tvos-release.apk
adb -s 192.168.0.229:5555 shell am start -n app.lume.tv/.LumeActivity
adb -s 192.168.0.229:5555 shell cmd package resolve-activity --brief -a android.intent.action.MAIN -c android.intent.category.HOME
adb -s 192.168.0.229:5555 exec-out screencap -p > /tmp/home-check.png
```

En HOME, pulsa Arriba para mostrar la barra, desplázate hasta el icono de Centro de control y pulsa OK. Comprueba navegación izquierda/derecha/arriba/abajo, Back, Wi-Fi, Bluetooth, Imagen/Sonido, Entradas y Ajustes. El botón Home del control debe abrir Google TV. En la prueba anterior, Wi-Fi abrió `NetworkActivity`, Bluetooth `AddAccessoryActivity`, Imagen/Sonido `DisplaySoundActivity`, Modo ambiente `DaydreamActivity` y Entradas el panel TCL. No había fuente HDMI activa; un cambio real de HDMI sigue sin probarse.

## Botón Home (HOME como launcher)

`cmd package set-home-activity app.lume.tv/.LumeActivity` devuelve Success pero esta TCL lo ignora: Home sigue abriendo Google TV. Desde 0.3.10, `HomeKeyService` (servicio de accesibilidad que solo filtra teclas) abre HOME con el botón Home, al encender la pantalla y tras el arranque. TCL bloquea el enlace del servicio sin el app-op de autoarranque.

```sh
# Activar HOME como inicio
adb -s 192.168.0.229:5555 shell appops set app.lume.tv APP_AUTO_START allow
adb -s 192.168.0.229:5555 shell settings put secure enabled_accessibility_services app.lume.tv/app.lume.tv.HomeKeyService
adb -s 192.168.0.229:5555 shell settings put secure accessibility_enabled 1
# Volver a Google TV
adb -s 192.168.0.229:5555 shell settings put secure enabled_accessibility_services null
```

También se puede apagar en Ajustes > Sistema > Accesibilidad > HOME. `adb shell input keyevent KEYCODE_HOME` **no** pasa por el filtro de accesibilidad y siempre abre Google TV; para probar sin el mando usa `adb shell uinput` con un dispositivo virtual (KEY_HOMEPAGE = 172). Comprobado el 2026-11-11 con uinput; encendido desde reposo y reinicio en frío sin comprobar, y el firmware puede desactivar el servicio por su cuenta.

## Rollback

La acción menos invasiva es pulsar Home: Google TV ya es el launcher predeterminado. Para retirar HOME, usa `adb -s 192.168.0.229:5555 uninstall app.lume.tv`; no hay que reactivar paquetes. El APK anterior está en `~/Projects/.clean-tv-os-private/baseline/base.apk`, pero tiene otra firma: para instalarlo hay que desinstalar HOME y se pierden sus preferencias. No deshabilites Google TV ni componentes TCL.
