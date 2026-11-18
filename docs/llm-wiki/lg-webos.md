# HOME en LG webOS · OLED55C1PSA

Estudiado el 2026-11-16. Esta página reúne lo que hace falta para llevar HOME a la segunda tele del usuario, una LG OLED. **La tele está rooteada desde el 2026-11-16** (véase «Estado del root»). Desde ese mismo día hay un **port nativo en curso** (`crates/lume-webos`, paquete `app.home.tv`; véase «Port de HOME a webOS»). Aquí están los hechos, los límites, la ruta elegida y el estado del port.

Cada dato lleva su procedencia: **[comprobado]** lo medí desde el Mac, **[LG]** viene de la documentación oficial (`webostv.developer.lge.com`), **[comunidad]** viene de proyectos de terceros y hay que confirmarlo en esta tele antes de darlo por cierto.

## La tele

| Dato | Valor | Procedencia |
| --- | --- | --- |
| Modelo | LG OLED55C1PSA (serie C1, 2021), nombre de dispositivo «OLED» | usuario y anuncio AirPlay [comprobado] |
| Sistema | webOS TV 6.5.3, firmware **03.53.45** | [comprobado] con `ares-device -i` |
| Motor web | Chromium 79 | [LG] |
| Plataforma | OTA `HE_DTV_W21O_AFABATAA`, placa `O20N_ATSC_US`, núcleo 4.4.84 aarch64, 4 núcleos, 2,6 GB de RAM (unos 700 MB libres) | [comprobado] |
| Gráficos | `libEGL`, `libGLESv2` y `libSDL2 2.0.10` del sistema en `/usr/lib` | [comprobado] |
| Almacenamiento de apps | 2,7 GB, 557 MB libres | [comprobado] |
| Red | `LGwebOSTV.local` → `192.168.0.185` (puede cambiar; resuelve por nombre) | [comprobado] |
| Puertos abiertos | 3000 y 3001 (control por red SSAP), 7000 (AirPlay), 9922 (SSH del Modo Desarrollador) | [comprobado] |
| Acceso | Root por SSH con clave (puerto 22); el Modo Desarrollador de LG se usó solo para leer datos y ya está desinstalado | [comprobado] |

El número de serie y la MAC no se guardan aquí: el repositorio es público.

En el Mac están instalados el CLI de webOS (`ares` 3.2.6, vía npm), webOS Dev Manager y el simulador `webOS_TV_25_Simulator_1.4.4.app` en `/Applications`. La GPU es Mali (`libmali.so`, con `libwayland-egl`).

## Estado del root [comprobado]

El usuario aceptó el root el 2026-11-16. Se hizo con SlopBro (commit `f76b717`, posterior a la v0.1.0) desde el Mac:

```sh
python3 slopbro.py --debug --webos-version 6 --local-ip <IP del Mac> 192.168.0.185
```

La tele pidió aceptar el emparejamiento, cargó el exploit en `com.webos.app.adoverlayex` e instaló Homebrew Channel 0.7.3. Después se desinstaló la app Developer Mode y se reinició **desenchufando la tele**: apagar con el mando solo la deja en reposo, y Homebrew Channel no aparece hasta un arranque completo.

Estado que devuelve `luna://org.webosbrew.hbchannel.service/getConfiguration`: `root: true`, `sshdEnabled: true`, `blockUpdates: true`, `telnetDisabled: true`.

- **Entrar:** `ssh -i ~/.ssh/oled_root -o IdentitiesOnly=yes root@192.168.0.185`. La clave pública está en `/home/root/.ssh/authorized_keys` de la tele. Copiar archivos: `scp -O` (es dropbear; la subida por Wi-Fi es lenta, unos 8 MB por minuto).
- **`luna-send` por SSH necesita terminal:** sin `ssh -tt` no imprime nada. Usa `luna-send -w 4000 -n 1 -f <uri> '<json>'`.
- **Telnet sin contraseña cerrado** (`/var/luna/preferences/webosbrew_telnet_disabled`). Si el arranque de Homebrew Channel falla dos veces entra en modo a prueba de fallos y lo vuelve a abrir.
- **No instalar la app Developer Mode de LG** con la tele rooteada, y **no actualizar el firmware**: Homebrew Channel bloquea las actualizaciones, pero conviene dejar también desactivada la automática en Ajustes → Soporte.
- **Si se pierde el root** (por ejemplo tras un restablecimiento), se repite SlopBro mientras el firmware siga en 03.53.45.
- **Control por red:** SlopBro dejó la clave de emparejamiento SSAP en su carpeta (`192.168.0.185.key`); con ella `ssap://com.webos.applicationManager/listApps` devolvió 124 apps.

## Prueba de sustitución del Home con Luma [comprobado]

El 2026-11-16 se instaló Luma 3.4.0 (suma SHA-256 verificada) y se probó en esta tele:

- Arranca como app nativa, lista las apps y las entradas HDMI de la tele y dibuja a 1080p. Memoria residente: unos 70 MB.
- `sh home.sh enable` (en la carpeta de la app) montó su ejecutable sobre `/usr/bin/com.webos.app.home`. Con otra app en primer plano, **pulsar Home abrió Luma**: la app en primer plano pasó a ser `com.webos.app.home` y la captura mostró su interfaz.
- `sh /var/lib/luma-home/home.sh disable` devolvió el Home de LG (`mounted: false`, sin gancho de arranque). **La tele quedó con el Home de LG**; Luma sigue instalado como app normal.

Conclusión: en esta tele un binario nativo en Rust con GLES2 puede ser el Home. No se probó el arranque en frío con la sustitución activa.

Herramientas de prueba, equivalentes al mando virtual y la captura de `adb`:

```sh
scripts/lg-key.sh HOME            # pulsa botones: HOME BACK ENTER UP DOWN LEFT RIGHT
ssh -tt -i ~/.ssh/oled_root -o IdentitiesOnly=yes root@192.168.0.185 \
  'luna-send -w 8000 -n 1 luna://com.webos.service.capture/executeOneShot "{\"path\":\"/tmp/shot.jpg\",\"method\":\"DISPLAY\",\"format\":\"JPG\",\"width\":960,\"height\":540}"'
scp -O -i ~/.ssh/oled_root -o IdentitiesOnly=yes root@192.168.0.185:/tmp/shot.jpg .
```

Otros servicios usados con éxito como root: `com.webos.applicationManager/launch`, `getForegroundAppInfo`, `getAppInfo`; `com.webos.appInstallService/dev/install` (`{"id":…,"ipkUrl":"/tmp/x.ipk","subscribe":true}`, termina con `"state":"installed"`); `com.webos.service.networkinput/getPointerInputSocket` (devuelve un socket Unix que acepta `type:button\nname:HOME\n\n`). El emparejamiento SSAP de SlopBro no tiene permiso de entrada (error 401), por eso el mando virtual va por SSH.

## Port de HOME a webOS (`app.home.tv`) [comprobado, a medias]

Empezado el 2026-11-16 por la ruta B. Decisión del usuario: **un paquete aparte de la TCL**, no un APK. El APK `app.lume.tv` no cambia por esto.

- **Separación.** El código LG vive en `crates/lume-webos` (binario `home`, unas 1 700 líneas); el Android sigue en `crates/lume-app`. Ninguno depende del otro; solo comparten `lume-core` (diseño, foco, animaciones, dibujo). Cualquier cambio en el núcleo debe seguir pasando sus pruebas para no romper la TCL (494 pasan el 2026-11-17).
- **Módulos.** `sdl.rs`: una docena de llamadas SDL2 por FFI directo, sin el crate `sdl2`, porque SDL-webOS entrega códigos de tecla fuera de su enum (Atrás 482, Home 384, Salir 505). `luna.rs`: una ida y vuelta al bus Luna con `libluna-service2` dentro del proceso, porque la jaula de la app **no deja ejecutar `luna-send`**. `bridge.rs`: apps, lanzar, niveles, reloj y estadísticas por Luna o `/proc`. `icons.rs`: decodifica los PNG de icono de cada app y los escala a la celda del atlas. `webos_compat.rs`: la libc del NDK no tiene `getauxval`; se lee `/proc/self/auxv`. `main.rs`: bucle recortado del de Android, sin aparcar/reanudar y sin redibujado parcial todavía (medir antes en esta GPU). `luna.rs` y `webos_compat.rs` siguen la técnica de Luma (MIT, con atribución en el código).
- **Paquete.** `crates/lume-webos/webos/appinfo.json`: `"type": "native"`, `main: home`, 1920×1080, `requiredACG` de launcher y consulta. Aún sin `appCategory: home`: hoy es una app normal.
- **Compilar.** `scripts/webos-build.sh` → `dist/app.home.tv_<versión>_arm.ipk` (`dist/` está fuera de Git). Usa la imagen `lume-webos-toolchain:latest` (`docker/Dockerfile.webos`: `lume-toolchain` más el destino `armv7-unknown-linux-gnueabi` y `file`), el NDK de webosbrew en `~/Projects/.clean-tv-os-private/webos-ndk/` (versión linux-aarch64, release `webos-d7ed7ee.6`) y descarga una vez SDL-webOS `release-2.30.12-webos.5`, comprobado con las sumas SHA-256 que fija Luma, que se empaqueta en `lib/`.
- **Instalar.** Copiar el `.ipk` a `/tmp` por `scp -O` e instalar con `com.webos.appInstallService/dev/install` (ver «Prueba de sustitución del Home»).

Estado en la tele el 2026-11-16 (versión 0.1.0, comprobado con captura): arranca como app nativa, dibuja la fila de apps con foco y la cápsula de la hora, lista **las apps reales** de la tele y ocupa unos 40 MB de memoria residente. Defectos vistos:

- Fondo negro: el registro dice `cannot load the wallpaper cinematic`.
- Las apps salen como rectángulos con el nombre, sin icono.
- La hora se corta en la cápsula («10:02…»).
- Sin empezar: entradas HDMI en Inicio, fondos de Luma, saludo con fecha, Centro de control para LG.

Se quedó así porque el agente que hacía el port lo detuvieron los filtros de seguridad del modelo mientras probaba pulsaciones de mando por un método de bajo nivel. Para probar, usa `scripts/lg-key.sh` y la captura de arriba; no hace falta nada más bajo. El Home de LG sigue siendo el Home (sin montaje ni ganchos de arranque) y Luma sigue instalado.

## Diseño híbrido HOME × Luma

Petición del usuario: combinar HOME con lo que le gusta de Luma (fondos, píldoras de entradas, widgets). Diseño en Figma, página **«06 · LG webOS · híbrido»** (`node-id=29-2`) del archivo de HOME (`2NhgcRgKySPuCRAOaEs2v7`); las páginas 00–05 son de la TCL y no se tocaron. Once pantallas a 1920×1080: Inicio plegado (píldoras HDMI arriba a la izquierda, hora a la derecha, saludo con fecha y clima, fila de seis apps), Inicio desplegado, Entradas con foco, Centro de control para LG (Reposo, Volumen, Entradas, Temporizador, AirPlay, Imagen, Sonido; sin Wi-Fi ni Bluetooth), Ajustes (raíz, Fondo con los seis fondos de Luma, Entradas, Widgets), Apps recientes (Atrás mantenido), modo ambiente y una página de notas. Componente nuevo: `Entrada/Píldora`.

Pendiente de que el usuario confirme:

- Píldoras y hora también visibles en el desplegado (en la TCL la hora se desvanece).
- «Brillo OLED · por comprobar» en el Centro de control: se quita si la LG no deja cambiarlo.
- Clima: está en el diseño, fuera del código por ahora (hace falta un servicio y una ciudad).
- Legibilidad de fecha y ciudad sobre fondos claros: oscurecer la parte superior.

Fondos de Luma (`ecklf/lg-luma/assets.md`): cinco fotos de Unsplash (licencia Unsplash) y **Yosemite, que exige crédito a su autor, David Iliff**. Los fondos SVG Midnight, Slate y Prism son de Luma (MIT). Si se incorporan, anotar el crédito en el repositorio.

## Diferencias de fondo con la TCL

| Tema | TCL 55Q5K (Android 14) | LG C1 (webOS 6) |
| --- | --- | --- |
| Tipo de app oficial | APK nativo | Solo app web (`"type": "web"`) [LG] |
| Sustituir el Home | Servicio de accesibilidad, sin root | Imposible sin root |
| Listar apps instaladas | API pública | No hay API pública; solo `launch` por id y `getAppLoadStatus` [LG] |
| Brillo del panel | Librería de TCL por reflexión | Sin API pública; con root, por el bus Luna |
| Volumen | Fijar valor exacto | Solo subir 1, bajar 1 y silenciar [LG] |
| Instalar en pruebas | `adb install -r`, permanente | Modo Desarrollador con sesión que caduca; al caducar **se desinstalan las apps** [LG] |
| Pruebas automáticas | `adb shell uinput` | SSAP por red (puertos 3000/3001) o `ares` |

## Dos caminos

### A. App web oficial, sin root

Lo que LG documenta. Se instala con el Modo Desarrollador y aparece como una app más.

Sirve para: mostrar la interfaz, lanzar apps cuyo id ya conozcamos (`luna://com.webos.applicationManager` método `launch`), subir, bajar y silenciar el volumen (`luna://com.webos.audio`), leer idioma, país y reloj (`luna://com.webos.settingsservice/getSystemSettings`), leer modelo y firmware (`luna://com.webos.service.tv.systemproperty/getSystemInfo`), saber si hay red (Connection Manager).

No sirve para: abrirse con el botón Home ni al encender, listar las apps instaladas, cambiar el brillo o el modo de imagen, cambiar de entrada HDMI de forma documentada, apagar la pantalla, ni Wi-Fi o Bluetooth (BLE GATT existe solo desde webOS 24).

Además exigiría reescribir la interfaz en HTML y JavaScript, o compilar `lume-core` a WebAssembly con WebGL (no probado), y la sesión del Modo Desarrollador hay que renovarla (cada 1000 horas según la comunidad; LG no publica la cifra). Con este camino HOME sería un prototipo, no el Home de la tele.

### B. App nativa en Rust sobre tele con root (recomendado)

Es la misma arquitectura que ya tiene HOME. `crates/lume-core` es independiente de la plataforma y dibuja con `glow` sobre OpenGL ES 2; `crates/lume-app` contiene solo la parte Android. El trabajo sería un crate nuevo de plataforma para webOS (ventana y entrada con SDL-webOS, el mismo `Gles2Backend`) y sustituir el puente JNI por llamadas al bus Luna.

Existe una prueba de que funciona: **Luma** (`ecklf/lg-luma`, MIT), un launcher nativo en Rust, SDL2 y GLES2 inspirado en Apple TV, que sustituye el Home de LG. Su tabla de compatibilidad marca la serie C1 con webOS 6 como compatible con la sustitución del Home (modelo `OLED48C1KNB`) [comunidad]. De su documentación:

- Paquete `.ipk` con `appinfo.json` de `"type": "native"`, `"appCategory": ["home"]` y permisos `requiredACG`: `application.launcher`, `application.query`, `lunabus.query`.
- Destino de compilación `armv7-unknown-linux-gnueabi` (coma flotante por software, **no** `gnueabihf`) con el NDK de webosbrew (`arm-webos-linux-gnueabi_sdk-buildroot`). El SDL del NDK es un stub; se empaqueta SDL-webOS (`release-2.30.12-webos.5`) junto al binario.
- Necesita Homebrew Channel **elevado**. Las llamadas con privilegios van por el servicio `exec` de Homebrew Channel, que ejecuta `luna-send -n 1` como root.
- Sustitución del Home en webOS 5 y 6: monta su ejecutable encima de `/usr/bin/com.webos.app.home` (montaje temporal, un reinicio lo deshace; el Home de LG no se borra) y añade un filtro de teclas en el compositor para abrirse antes en el arranque en frío. Alternativa suave: arranque automático unos 25 s después de encender, sin tocar el botón Home.
- Redibujado parcial con `EGL_EXT_buffer_age` y `EGL_KHR_partial_update`, igual que hace HOME en la TCL.
- Servicios que usa: `com.webos.service.applicationmanager/launch`, `com.webos.service.recent/listRecents`, `com.webos.appInstallService`.

Es referencia de técnica, no código incorporado. Su licencia MIT es compatible con GPL-3.0 si algún día se copia algo, con atribución.

**El root en esta tele.** [comunidad], sin comprobar aquí:

| Método | Estado en C1 con firmware reciente |
| --- | --- |
| RootMy.TV | Cerrado desde 2022 |
| crashd | Cerrado en esta plataforma desde 03.40.85 |
| faultmanager | Cerrado en el último firmware de webOS 6 (nota del autor, 2025-08-24) |
| DejaVuln | Reportado como fallido en 03.53.45 |
| **SlopBro** (`throwaway96/slopbro`) | Un diario de campo de una OLED77C1PUB con firmware 03.53.45 y webOS **6.5.3-47** (**mismo firmware, misma plataforma OTA y mismo webOS que esta tele**; cambia el tamaño y la región) reporta éxito con la v0.1.0 el 2026-10-29; dejó Homebrew Channel 0.7.3 y root por telnet |

SlopBro se ejecuta desde el Mac con Python 3.8 o superior, se conecta por SSAP (la tele muestra un aviso de emparejamiento) y no necesita el Modo Desarrollador. Su autor dice que un intento fallido no daña la tele. LG ya cerró el fallo en webOS 10 y 11 en octubre de 2026, así que una actualización de firmware puede cerrarlo también aquí: **no actualizar el firmware** hasta decidir. `cani.rootmy.tv` es el comprobador por modelo y firmware; no pude leerlo desde aquí (la página carga los datos con JavaScript).

Riesgos del root: puede afectar a la garantía, cambios imprudentes en el sistema pueden inutilizar la tele, y con root la comunidad recomienda quitar la app Developer Mode y no reinstalarla.

El resultado en esta tele está en «Estado del root».

## Modo Desarrollador (sirve para el camino A y para probar antes del root)

Pasos de LG:

1. Cuenta en el sitio de desarrolladores de LG.
2. En la tele: LG Content Store → buscar «Developer Mode» → instalar → iniciar sesión → activar **Dev Mode Status** (la tele se reinicia).
3. En el Mac:

```sh
ares-setup-device --add oled -i "host=LGwebOSTV.local" -i "port=9922" -i "username=prisoner"
# En la app Developer Mode de la tele, activar «Key Server» y después:
ares-novacom --device oled --getkey     # descarga la clave; exige «Key Server» encendido (puerto 9991)
ares-setup-device --modify oled -i "privatekey=oled_webos" -i "passphrase=<frase de la tele>"
ares-device --system-info --device oled # modelo, firmware, SDK, OTA ID
```

La sesión caduca; se alarga con el botón **EXTEND** de la app mientras la tele tenga red. Si caduca y la tele se reinicia, o se reinicia diez veces sin red, el modo se desactiva y las apps instaladas se borran.

## Comandos del CLI

```sh
ares-generate -t basic miapp            # plantilla de app web
ares-package --no-minify miapp          # crea el .ipk (sin minificar para depurar)
ares-install --device oled ./com.ejemplo.app_1.0.0_all.ipk
ares-install --device oled --list       # apps instaladas en modo desarrollador
ares-launch --device oled com.ejemplo.app
ares-launch --device oled --close com.ejemplo.app
ares-launch --device oled --hosted miapp   # sin instalar, recarga al guardar
ares-inspect --device oled --app com.ejemplo.app --open   # inspector web
ares-device --device oled -r -t 1 -s recursos.csv         # CPU y memoria
ares-launch -s 25 -sp /Applications miapp  # simulador de webOS 25
```

El último comando no se ha probado; `-sp` es la carpeta que contiene el simulador.

## El simulador no es esta tele

El simulador instalado es de webOS 25 y usa Chromium 120. La C1 usa **Chromium 79**. Una app web que funcione en el simulador puede fallar en la tele. En Chromium 79 faltan, entre otros: encadenamiento opcional `?.` y `??` (llegan en 80), `gap` en flexbox (84), `aspect-ratio` (88), `inset` (87), `:has()` y consultas de contenedor, `structuredClone`. Sí hay `backdrop-filter`, WebGL 2, WebAssembly y módulos ES. El simulador tampoco ejecuta apps nativas ni todos los servicios Luna (el de audio no), así que el camino B solo se prueba en la tele.

## Datos de plataforma para una app web [LG]

- `appinfo.json`: obligatorios `id`, `title`, `type` (`web`), `main`, `icon` (PNG 80×80), `version` (tres números). El `id` no puede empezar por `com.webos`, `com.lge`, `com.palm`.
- `resolution`: `1920x1080` (por defecto) o `1280x720`. No hay interfaz 4K para apps web.
- `disableBackHistoryAPI: true` para recibir la tecla Atrás; `handlesRelaunch: true` para preparar la pantalla antes de volver al frente (`webOSSystem.activate()`); `splashBackground` (PNG 1920×1080); `transparent`.
- Ciclo de vida: eventos `webOSLaunch` (con parámetros en `detail`), `webOSRelaunch` y `visibilitychange`; una app oculta queda suspendida.
- Teclas: izquierda 37, arriba 38, derecha 39, abajo 40, OK 13, **Atrás 461**, rojo 403, verde 404, amarillo 405, azul 406, reproducir 415, pausa 19, parar 413. Encendido, volumen y Home son del sistema y la app no los recibe. En webOS 6, Atrás sin historial pregunta si se sale de la app; Atrás mantenido va al Home.
- El Magic Remote alterna entre puntero y cruceta; la app debe funcionar entera con cruceta (evento `cursorStateChange` para saber si el cursor está visible).
- Las llamadas Luna se hacen con la librería `webOSTV.js` (`webOS.service.request`).

## Control por red (SSAP)

Los puertos 3000 y 3001 están abiertos [comprobado]. Es el protocolo que usan la app móvil de LG y SlopBro: un WebSocket con emparejamiento que la tele pide aceptar una vez. Según la comunidad permite listar y lanzar apps, fijar el volumen, cambiar de entrada, apagar y enviar pulsaciones del mando. Sería el equivalente al mando virtual de `adb` para automatizar pruebas. No se ha emparejado ni probado.

## Sin comprobar

- El arranque en frío con la sustitución del Home activa, y que SSH arranque solo tras reiniciar.
- Rendimiento de `lume-core` en la GPU de la C1 y si el compositor expone la edad del búfer.
- Qué servicios Luna dan brillo del panel OLED, modo de imagen, entradas y temporizador con root.
- El lanzamiento en el simulador y la captura de pantalla con `ares`. El usuario preguntó si se puede usar el simulador del Mac: para el port nativo no sirve (solo ejecuta apps web).
- Si `lume-webos` compila tras los últimos cambios del núcleo; no hay pruebas propias del crate.

## Siguiente paso propuesto

1. Hecho: root con SlopBro.
2. Hecho: Luma sustituyó el Home en la prueba.
3. Hecho a medias: `crates/lume-webos` corre en la tele como `app.home.tv` 0.1.0.
4. Siguiente: fondo, iconos y hora (defectos de arriba); después entradas HDMI, fondos de Luma y saludo según el Figma una vez aprobado; después Centro de control con lo que el bus Luna permita; al final, `appCategory: home` y la sustitución del Home como hizo Luma.

## Fuentes

- LG: `develop/getting-started`, `developer-mode-app`, `web-app-types`, `app-lifecycle`, `guides/app-lifecycle-management`, `guides/magic-remote`, `references/appinfo-json`, `references/luna-service-introduction`, `references/audio`, `references/settings-service`, `references/application-manager`, `references/tv-device-information`, `specifications/web-api-and-web-engine`, `tools/cli-dev-guide`, `tools/simulator-introduction`.
- Comunidad: `webosbrew.org/rooting`, `github.com/throwaway96/slopbro`, `github.com/ecklf/lg-luma` (readme y `development.md`), `github.com/thiagopanda/lg-oled77c1pub-field-notes`, `github.com/throwaway96/faultmanager-autoroot`.
