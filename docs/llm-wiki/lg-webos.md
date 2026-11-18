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
- **Desde el PC** (2026-11-17) también entra la clave `~/.ssh/id_ed25519`, añadida a `authorized_keys` desde el Mac; la clave `oled_root` sigue solo en el Mac.
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

**Versión 0.1.1 (2026-11-17, comprobado con captura y con `home.log`).** Defectos corregidos y su causa:

- **Fondo negro.** El paquete no llevaba los fondos. `scripts/webos-build.sh` copia ahora `crates/lume-core/assets/wallpaper/*` (19 QOI, 28 MB) a `assets/wallpaper` del ipk. El fondo se descodifica en 144 ms la primera vez y en 10–36 ms desde la caché.
- **Apps sin icono.** HOME corre enjaulado (uid 6715) y no ve `/usr/palm/applications` ni `/media/cryptofs/apps`, solo `/media/developer/apps`. `Bridge::mirror_unreadable_icons` escribe una lista en `/media/developer/app.home.tv/icons/wanted.txt`, y una sola llamada a `luna://org.webosbrew.hbchannel.service/exec` copia como root los iconos a esa carpeta, que la jaula sí ve. La primera vez fueron 35 iconos en 1,3 s; después solo se copian los nuevos.
- **Centro de control congelado.** Tres causas:
  1. Cada llamada Luna registraba y desregistraba una conexión al bus. Ahora hay una conexión por hilo, que se descarta si una llamada caduca.
  2. Los niveles se leían en el bucle de dibujo cada 300 ms. Ahora los lee y escribe el hilo `levels` (`src/levels.rs`), y las órdenes del deslizador se agrupan para enviar solo la última.
  3. El audio no responde a HOME: `master/getVolume` devuelve `"Message status unknown"` o caduca a los 6,5 s. webOS da a una app de desarrollo solo el grupo ACG `public` (`/var/luna-service2-dev/client-permissions.d/app.home.tv.app.json`) aunque `requiredACG` pida más. El volumen se lee y se cambia como root con `luna-send` a través del `exec` de Homebrew Channel (`call_as_root`), en unos 110 ms. El resto de llamadas de HOME sí son públicas (apps, ajustes, `backlight`, red, Bluetooth, `turnOffScreen`, información del sistema).
- **Primer fotograma:** 0,57 s; antes, 7 s.
- **Wi-Fi y Bluetooth del Centro de control** abren los ajustes de LG con `params.target`: `ExternalDevice` para Bluetooth y `network` para Wi-Fi.
- **Registro:** `/media/developer/app.home.tv/home.log`, nuevo en cada arranque. SAM manda el stderr de las apps nativas a `/dev/null`.

**Incidente:** el 2026-11-17 a las 21:01 `audiod` murió por `SIGABRT` y systemd no lo reinició (`audiod.service` en `failed`). Mientras estuvo caído, ninguna llamada de audio respondía, ni siquiera como root. Se arrancó de nuevo con `systemctl start audiod.service`. Un chorro de 30 `setVolume` simultáneos no lo reprodujo, así que **la causa no se conoce**. Si el volumen deja de responder, mira `systemctl is-active audiod.service`.

**`scripts/lg-key.sh` dejó de funcionar** el 2026-11-17: el socket de `networkinput` acepta la conexión y la cierra sin entregar la tecla, ni siquiera `HOME`, con los servicios del gancho de arranque parados o en marcha. Las pruebas de mando quedan en manos del usuario con el mando físico.

Compilar en un PC Linux x86_64: el NDK es `openlgtv/buildroot-nc4` `webos-d7ed7ee` (`arm-webos-linux-gnueabi_sdk-buildroot.tar.gz`, misma versión que el de webosbrew); la imagen base se construye con `docker build --target toolchain -t lume-toolchain:latest -f docker/Dockerfile .`, y `ares-package` se ejecuta con `npx` si no está instalado (el script ya hace las dos cosas). Para probar cambios sin reinstalar: cierra la app (`closeByAppId`), espera a que no haya proceso `home`, copia `dist/webos/home` encima del binario instalado y relanza.

Se quedó así porque el agente que hacía el port lo detuvieron los filtros de seguridad del modelo mientras probaba pulsaciones de mando por un método de bajo nivel. Para probar, usa `scripts/lg-key.sh` y la captura de arriba; no hace falta nada más bajo. El Home de LG sigue siendo el Home (sin montaje ni ganchos de arranque) y Luma sigue instalado.

## API del bus Luna comprobada con root (2026-11-17) [comprobado]

Probado por SSH como root con `luna-send -w 4000 -n 1 -f`. Cada escritura se hizo con el valor actual o con ida y vuelta, y se leyó de nuevo; la tele quedó como estaba. Hay 138 servicios en `/usr/share/luna-service2/services.d/`.

| Función | Llamada | Resultado |
| --- | --- | --- |
| Entradas y su estado | `com.webos.service.eim/getAllInputStatus {}` | Comprobado: `devices[]` con `id` (`HDMI_1`…), `label`, `appId` (`com.webos.app.hdmi1`…), `connected`, `activate`, dispositivos CEC en `subList` y `arcDevice` |
| Cambiar de entrada | `applicationManager/launch {"id":"com.webos.app.hdmiN"}` | Comprobado que existen `hdmi1`–`hdmi4` en `listApps`; abrirlas es el mismo `launch` ya probado |
| Volumen exacto | `com.webos.service.audio/master/getVolume` y `setVolume {"volume":N}` | Comprobado: `volume`, `maxVolume` 100, `muteStatus`, `soundOutput` |
| Salida de sonido | `com.webos.service.apiadapter/audio/getSoundOutput` y `changeSoundOutput {"output":…}` | Comprobado (hoy `external_arc`) |
| Brillo OLED | `settingsservice` categoría `picture`, clave `backlight` (0–100) | Comprobado. `oledLight` **no existe** en esta tele (`errorKey`); el brillo OLED se llama `backlight` |
| Modo de imagen | `picture` / `pictureMode` | Comprobado lectura y escritura (`vivid`) |
| Modo de sonido | `sound` / `soundMode` | Comprobado lectura y escritura (`aiSoundPlus`, `standard`…) |
| Temporizador de apagado | `time` / `sleepTimer` | Comprobado (`off`) |
| Apagar y encender la pantalla | `com.webos.service.tvpower/power/turnOffScreen` y `turnOnScreen` | Comprobado: el estado pasa a `Screen Off` y vuelve a `Active` |
| Estado de energía | `tvpower/power/getPowerState` | Comprobado |
| Bluetooth | `com.webos.service.bluetooth2/adapter/getStatus` | Comprobado: `powered`, `discovering`, nombre. Cambiar redes o emparejar sigue fuera del diseño |
| Estado de la red | `com.webos.service.connectionmanager/getStatus` | Comprobado: `isInternetConnectionAvailable`, estado de Wi-Fi y cable |
| Recientes | `com.webos.service.recent/listRecents` | Comprobado: `recentsAppList[]` con `appId`, `title`, `splashBackground` |
| Cerrar una app | `applicationManager/closeByAppId {"id":…}` | Comprobado con `app.home.tv`: se cierra y sale de `running` |
| App en primer plano y residentes | `getForegroundAppInfo`, `running` | Comprobado |
| Aviso del sistema | `com.webos.notification/createToast {"sourceId":…,"message":…}` | Comprobado: devuelve `toastId` |
| Desinstalar | `com.webos.appInstallService/remove {"id":…}` | Existe y responde (`returnValue: true` con un id inexistente); no se desinstaló nada real |
| Gancho de arranque | `/var/lib/webosbrew/init.d/` | Comprobado (2026-11-17): un script ejecutable de prueba corrió en el arranque, unos 7 s antes de que SSH aceptara conexiones; después se borró y la carpeta quedó vacía |
| Apagar la tele | `tvpower/power/shutDown {"reason":"remoteKey"}` | Comprobado: la tele se apaga como con el mando. `power/turnOff` **no existe** (`Unknown method`). Al encenderla con el mando arrancó en frío (`getPowerOnReason` → `remoteKey`) aunque Quick Start+ estaba activo; SSH tarda unos 15 s más |

Figma (`HOME OS · LG webOS`) recoge estas insignias en la página 04 y en las notas de cada pantalla; la pantalla nueva es **08.10 · Rendimiento**, a la que se llega desde 08.8 Sistema.

Queda por medir con el mando físico: puntero, rueda y teclas numéricas (qué eventos SDL llegan al port).

## Rendimiento y procesos en segundo plano [comprobado]

Medido el 2026-11-17 con el Home de LG en primer plano, 26 minutos después de encender.

- **La carga media de ~13,7 no es CPU.** La CPU está libre en un 92 %. La carga la suman 13 hilos de drivers de LG que viven en estado D (`hdmi21_rx_engin`, `AUD-KDRV-TASK`, `LGDTV-PMDRV-TAS`, `de_thread`…). No se pueden tocar ni hace falta.
- **El cuello es la memoria.** De 2,6 GB quedan unos 550 MB disponibles, 60 MB libres de verdad y unos 150 MB en swap.
- **El mayor consumidor de CPU es `nn-proxy`** (~4,5 % de un núcleo), el proxy de la red neuronal del chip O20. No depende del modo de sonido (igual con `aiSoundPlus` que con `standard`).
- `cpulimit -p <update> -l 30` limita el servicio `update` al 30 %.

**Apps residentes y precarga.** `com.webos.service.preloadmanager` mantiene unas 20 apps precargadas mientras el Home de LG está en primer plano (`allowedApps` en `/etc/palm/webos-preload-manager-conf.json`). Si se cierran con `closeByAppId` se liberan unos 190 MB, pero en menos de 30 s vuelven YouTube y LG Channels, y además se abren el salvapantallas y la voz. El interruptor real es la política de precarga de cada app:

```sh
luna-send -n 1 luna://com.webos.service.preloadmanager/getPreloadPolicy '{}'
luna-send -n 1 luna://com.webos.service.preloadmanager/setPreloadPolicy '{"application":{"id":"youtube.leanback.v4","isEnabled":false}}'
```

Comprobado con YouTube: con `isEnabled:false` no volvió en 45 s; con `true` quedó restaurado. **No sobrevive a un reinicio**: tras apagar y encender volvió a `isEnabled:true`. HOME tendrá que reaplicar sus interruptores de precarga al arrancar (por ejemplo, desde su gancho `init.d`).

| Precargada | Memoria residente medida | Memoria que pide la política |
| --- | --- | --- |
| `com.webos.app.home` (Home de LG) | 177 MB | 40 MB |
| `youtube.leanback.v4` | 115 MB | 135 MB |
| `com.webos.app.browser` | ~87 MB (3 procesos) | 72 MB |
| `airplay` (web) | 65 MB | 50 MB (precarga desactivada; corre igual) |
| `com.webos.app.lgchannels` | 59 MB | 100 MB |
| `com.webos.app.inputcommon` (HDMI, TV en vivo) | 58 + 23 MB | 40 MB por entrada |
| `com.webos.app.voice`, `google.assistant`, `amazon.alexapr`, `netflix`, `amazon` | no estaban abiertas en la medida | 42–100 MB cada una |

**Ajustes que apagan funciones de fondo.** Todos son `com.webos.settingsservice/setSystemSettings` con valores `"on"` u `"off"`, comprobados con ida y vuelta. No se midió si el proceso asociado sale al cambiarlos.

| Ajuste | Categoría / clave | Valor actual | Qué hace |
| --- | --- | --- | --- |
| Live Plus (ACR, reconocimiento de contenido) | `option` / `livePlus` | `off` | Ya apagado |
| Promociones en Home | `general` / `homePromotion` | `off` | Ya apagado |
| Promociones en TV en vivo | `option` / `livePromotion` | `on` | Anuncios en el canal y la guía |
| Vídeo promocional de webOS | `option` / `webOSPromotionVideo` | `on` | Vídeo de demostración |
| Avisos emergentes | `option` / `enableToastPopup` | `on` | Avisos del sistema (los de HOME también usan este canal) |
| Alertas deportivas | `general` / `sportsAlarm` | `on` | Proceso `sportsalarm` (4 MB) |
| Anuncio por Bluetooth LE | `network` / `bleAdvertisingOnOff` | `on` | Hace visible la tele a la app móvil |
| Salvapantallas | `general` / `screenSaverEnabled` | `on` | Abre `com.webos.app.screensaver` |
| Encendido rápido (Quick Start+) | `option` / `quickStartMode` | `on` | Deja la tele en reposo caliente; apagarlo alarga el arranque |

**Servicios residentes sin interruptor.** Se pararían solo con un `kill` desde `init.d`, lo que sobrevive al reinicio y requiere permiso del usuario. Memoria residente medida:
- `homeconnect` (ThinQ): 26 MB.
- `amazon-alexa-adapter`: 13 MB.
- `airplay`, `homekit` y `airplay-adaptor`: 23 MB.
- `iot-client`: 9 MB.
- `sdx`: 9 MB.
- `nlpmanager`, `voiceinput` y `voiceconductor`: 14 MB.
- `miracast` y `miracast-tx`: 7,5 MB.
- `uploadd`: 4,6 MB.
- `contentminer`: 4,4 MB.
- `nudge`: 3 MB.
- `cis`: 3 MB.
- `appcasting`: 3 MB.
- `lg.thinqai.adapter`: 2,6 MB.

En total suman unos 120 MB, menos de lo que libera desactivar la precarga de YouTube y el navegador.

## Arranque y encendido [comprobado]

Medido el 2026-11-17 con `systemd-analyze` y `/var/log/bootd.log` tras un arranque.

- **Arranque por snapshot.** El núcleo arranca con `snapshot resume=/dev/mmcblk0p51` (imagen de 150 MB): restaura una imagen de memoria guardada en el eMMC. Quick Start+ (`option/quickStartMode`) está activo.
- **Cronología desde el núcleo:**

  | Momento | Tiempo |
  | --- | --- |
  | Núcleo | 0,85 s |
  | `snapshot-boot-on` (ruta crítica) | 12,3 s |
  | `sam-ready` | 13,0 s |
  | Primera app («Last Input App», hoy `com.webos.app.livetv`) | 13,9 s |
  | `tv-ready` | 16,6 s |
  | Mando listo (`mrcu-ready`) | 17,6 s |
  | `boot-complete` | ~49 s |
  | Gancho `init.d` | ~36 s |
  | Precargas de `preloadmanager` (LG Channels, HDMI 1, navegador, voz) | 60–82 s |

  El tiempo del cargador de arranque, antes del núcleo, no se ve desde dentro.
- **`/etc` es de solo lectura** (squashfs con overlay `ro`). No se pueden enmascarar unidades systemd sin escribir en particiones del sistema, y eso está prohibido. Solo se puede actuar con ajustes Luna, con el gancho `init.d` (corre antes de las precargas) y con `systemctl stop` en tiempo de ejecución.
- `bootd` abre al encender la última entrada (`Last Input App`), o el Home si `general/homeAutoLaunch` está en `on` (hoy está en `off`).
- La red en reposo depende de `network/wolwowlOnOff` («TV encendida con móvil», hoy `true`). Tras `power/shutDown` la tele no respondía al ping.

### Ajuste de arranque aplicado (2026-11-17) [comprobado]

Pedido por el usuario: no usa antena y quiere HOME al encender. El gancho `scripts/lg-boot-hook.sh` está instalado en la tele como `/var/lib/webosbrew/init.d/60-home-os`, que corre a unos 32 s del arranque. Deja su registro en `/tmp/home-os-boot.log` y hace tres cosas:

1. Abre HOME (`app.home.tv`) a los ~32 s.
2. Para con `systemctl stop --no-block` los servicios `legacy-broadcast` (emisión ATSC, 24 MB), `iepg`, `contentminer`, `uploadd` y `nudge`. (Hasta el 2026-11-18 también paraba `utp`; eso rompía Quick Start+, véase «Arranque más rápido».) Se probó en marcha más de 10 minutos, abriendo HDMI 3, HOME y el Home de LG, sin ningún reinicio del vigilante. Sin `--no-block`, `stop nudge` se queda colgado hasta el tiempo límite de systemd. `nudge` vuelve a arrancar más tarde; son 3 MB y se deja así.
3. Espera a que `preloadmanager` responda (hacia los 48 s) y apaga la precarga de `lgchannels`, `browser`, `voice`, `google.assistant`, `amazon.alexapr`, `amazon`, `livetv`, `livemenu` y `miracast`. Si alguna ya estaba abierta, la cierra. Se mantienen la del Home de LG, las de HDMI, Netflix y YouTube.

Ajustes cambiados una sola vez (persisten solos; entre paréntesis, el valor anterior):
- `general/homeAutoLaunch` → `on` (`off`);
- `option/livePromotion` → `off` (`on`);
- `option/webOSPromotionVideo` → `off` (`on`);
- `general/sportsAlarm` → `off` (`on`).

| Medida (tras `reboot`) | Antes | Con el gancho |
| --- | --- | --- |
| `tv-ready` | 16,6 s | 13,7 s |
| `boot-complete` | ~49 s | 44 s |
| HOME en pantalla | no se abría | 32 s |
| Precargas a los 60–82 s | 4 apps | ninguna |
| Memoria disponible a los 105 s | ~550 MB | 910 MB |

Con `homeAutoLaunch` en `on`, bootd sigue abriendo la última entrada y pone encima el Home de LG («Cover App»). Entre los ~18 s y los 32 s se ve el Home de LG y después aparece HOME. Para que HOME salga antes hay que usar la vía temprana de Luma: un servicio LS2 que se activa hacia los 18 s y un filtro de teclas en el compositor (véase `com.ecklf.luma/home.sh`, orden `early`). Eso exige que HOME sea el Home, y el port todavía tiene defectos visuales.

**Deshacer:** `rm /var/lib/webosbrew/init.d/60-home-os` y reiniciar. Los servicios y las precargas vuelven solos. Los cuatro ajustes se devuelven a su valor anterior con `setSystemSettings`.

Falta medir el encendido desde el mando con cronómetro: el tiempo antes del núcleo y si el reposo del mando es suspensión o arranque completo.

**Encender con el mando** (2026-11-17, medido): tras apagar y encender con el mando, la tele arrancó de cero, con el tiempo encendido de nuevo en 0 y `getPowerOnReason` → `remoteKey`. En parte se debía a que el gancho paraba `utp`. Pasados unos minutos, Quick Start+ apaga del todo igualmente: véase «Arranque más rápido».

## Arranque más rápido (2026-11-18) [comprobado]

**Qué es Quick Start+ en esta C1.** No es un reposo caliente que dure. Al apagar con el mando:
1. La tele pasa a `Active Standby` (pantalla apagada y todo en memoria).
2. Entre 20 s y unos 6 minutos después, `tvpowerd` pasa a `Prepare Suspend` y reinicia la tele (`NL_POWER_STATE` → `Reboot`, `PowerOffReason: quickStart`).
3. Ese arranque sin pantalla queda en `Active Standby` con `getPowerOnReason` → `quickStart`, renueva la imagen de snapshot y se apaga del todo.
4. El siguiente encendido es el arranque por snapshot (núcleo y restauración en ~12 s).

Solo un encendido dentro de esa ventana es instantáneo. Se comprobó con `power/powerOff {"reason":"remoteKey"}` y encendido por WoL.

**Por qué antes no había ni esa ventana.** El gancho paraba `utp.service`. Al apagar, `tvpowerd` llama a `luna://com.webos.service.utp.broadcast/getWarmOffStatus`; sin `utp` no hay respuesta. Con `utp` en marcha responde `{"warmOwner":[]}` y existe `Active Standby`. Esto no se repitió sin `utp` (A/B).

**Qué abre bootd al despertar de `Active Standby`.** Reevalúa `/etc/palm/action-condition.json` y abre la última entrada (`Input App`, hoy HDMI 3). No usa `firstAppByPowerOnReason`. bootd sí lee `/var/luna/preferences/bootd.json`, pero no se instaló porque no cambiaría esta ruta. `homeAutoLaunch` no pone el Home de LG.

**Cambios:**
1. `scripts/lg-boot-hook.sh`:
   - ya no para `utp`;
   - apunta cada arranque en `/var/lib/home-os/boot-history.log` (motivo y estado; 40 líneas como máximo);
   - en un arranque sin pantalla (`quickStart`, estado distinto de `Active`) no abre apps ni para servicios;
   - deja un vigilante de `tvpower/power/getPowerState`. Al ver `processing: "LastInput Ready"` (solo ocurre al despertar) espera 0,2 s y abre HOME, y vuelve a comprobar a los 3 s. Registro: `/tmp/home-os-wake.log`.
2. `crates/lume-webos/boot/home-boot.js` es un filtro del compositor, instalado con `scripts/lg-boot-filter.sh install` en `/var/lib/home-os/boot/`. Va en `com.webos.surfacemanager.keyFilters` justo detrás de `homePanelKey`. Se carga hacia los 16 s. En un arranque con pantalla (`Active`), al llegar `minimal-boot-done` abre HOME y lo reintenta una vez a los 6 s. Las teclas pasan sin cambios. La técnica viene de `home-filter.js` de Luma (MIT), escrito de nuevo.

Los dos solo abren HOME si delante hay una entrada (`livetv`, `hdmiN`, `externalinput.*`), el Home de LG o nada. Encender con la tecla de Netflix sigue abriendo Netflix.

**Medidas** (`/var/log/bootd.log`):

| Encendido | Antes | Ahora |
| --- | --- | --- |
| Dentro de la ventana de `Active Standby` | no existía (arranque completo) | pantalla a 2,2–3,1 s, HOME a 4,5–4,9 s |
| Arranque por snapshot (tras `quickStart`, o `reboot`) | HOME a 31–34 s | HOME a 26,2 s (`minimal-boot-done` a 22,7 s) |

Se ve HDMI 3 unos 1,5 s antes de HOME al despertar, y desde los ~19,5 s en el arranque por snapshot. `audiod` siguió activo tras cada arranque.

**Deshacer:**
- Filtro: `scripts/lg-boot-filter.sh uninstall`.
- Gancho: volver a la versión anterior (con `utp.service` en el `systemctl stop` y sin vigilante ni historial) y reinstalar con `scp -O`; o `rm /var/lib/webosbrew/init.d/60-home-os`; después reiniciar. `rm /var/lib/home-os/boot-history.log` borra el historial.

**Pendiente:**
- Comprobarlo con el mando real; aquí se encendió por WoL (`wakeOnWiFi`).
- Medir cuánto se ve la entrada antes de HOME con cronómetro.
- No se sabe qué decide el tiempo en `Active Standby` (20 s frente a 6 min). `tvpowerd` tiene un temporizador de apagado forzado cuando `warmOwner` está vacío.

## Luma como Home (2026-11-18) [comprobado]

Decisión del usuario: en la LG se deja HOME (`app.home.tv`) y se usa **Luma**, porque es más rápido y más completo. El panel ⚙ sigue, porque vive en el compositor y no en la app HOME.

- `echo y | sh /media/developer/apps/usr/palm/applications/com.ecklf.luma/home.sh enable` (método `executable` para webOS 6):
  - monta el ejecutable de Luma sobre `/usr/bin/com.webos.app.home`, así que el botón Home abre Luma;
  - crea `/var/lib/luma-home`;
  - instala el gancho `init.d/64-luma-home`;
  - registra el servicio temprano `com.webos.service.lumahome` (`luma-early`) y el filtro de teclas de Luma, para que un arranque en frío abra Luma hacia los 20 s.
- `sh /var/lib/luma-home/home.sh startup-home on`: al encender se abre el Home (Luma) en lugar de la última entrada. Guarda la política de entrada anterior.
- Estado comprobado (`home.sh status`): `enabled`, `mounted`, `bootHook`, `earlyStart` y `startupHome` a `true`. El filtro del panel ⚙ sigue el primero en `keyFilters`.
- **Deshacer:** `sh /var/lib/luma-home/home.sh startup-home off` y `sh /var/lib/luma-home/home.sh disable`.
- El gancho de HOME (`60-home-os`) ya no debe abrir `app.home.tv`; conserva el recorte de servicios y precargas.

**Cambio del 2026-11-18 de madrugada.** El usuario pasó a Luma como Home. El gancho ya no abre `app.home.tv` y se quitó el vigilante de despertar. Se desinstaló `home-boot.js` (`lg-boot-filter.sh uninstall`; el código sigue en el repo). El gancho conserva el recorte de servicios (sin `utp`), las precargas y el historial de arranques.

**Always Ready (`general/alwaysOn`).** Estaba en `off`; ahora está en `on` por decisión del usuario, que nota el encendido rápido con él. `tvpowerd` se salta el apagado forzado de `Active Standby` cuando está activo («Forced Power Off Timer skip because alwaysOn is on»). Fuera de horas lo desactiva `alwaysOnDisableStartHour`/`EndHour`, de 1:00 a 6:00 por defecto (se dejaron así). En la única prueba propia (franja movida a 4–5, ya restaurada) la tele hizo igualmente el reinicio `quickStart` a los ~50 s. No se midió el consumo en reposo; mantener el sistema en `Active Standby` gasta más que el apagado completo. Deshacer: `setSystemSettings {"category":"general","settings":{"alwaysOn":"off"}}`.

### Pendiente al cerrar la sesión (2026-11-18)

- **Encendido rápido:** comprobar con el mando real que, con Always Ready, la tele vuelve al instante tras 2–3 min apagada (`/var/lib/home-os/boot-history.log` dice si hubo reinicio `quickStart`). Si no basta, probar a quitar la franja de 1:00 a 6:00 o a registrar una petición de `Active Standby` (`registerActiveStandbyRequest`), con más consumo en reposo.
- **Luma:** comprobar que el botón Home abre Luma y que Luma es lo primero tras encender, tanto desde el reposo como en un arranque en frío.
- **Cursor ⚙:** ver arriba; si la política 1 lo quita, dejarla solo en ⚙ y devolver los demás botones a su valor.
- **Vidrio esmerilado** del panel sobre Luma (`frost` 1): comprobar que la captura no sale negra y que la animación va fluida.
- `app.home.tv` sigue instalada, pero ya no se usa en la LG.

## Panel rápido con la tecla ⚙ (2026-11-17) [comprobado]

Petición del usuario: que ⚙ abra algo como el Centro de control de HOME sobre cualquier app (bajar el brillo viendo una serie), no los ajustes de LG.

- **Por qué no es una app.** Una app nativa no puede ser una superposición: con `"defaultWindowType": "overlay"` en `appinfo.json`, el sistema sigue registrando HOME como `_WEBOS_WINDOW_TYPE_CARD` y tapa la app de debajo. El panel de ajustes rápidos de LG tampoco es una app: lo dibuja el compositor.
- **Cómo funciona.** Es un filtro de teclas del compositor (`surface-manager`), la misma vía que usa Luma. `crates/lume-webos/panel/home-panel.js` (ES5) queda primero en `com.webos.service.config` → `com.webos.surfacemanager.keyFilters`. El compositor se suscribe a esa lista y recarga los filtros al momento, sin reiniciarse, y la lista se mantiene tras reiniciar (comprobado). El filtro crea `HomePanel.qml` (más `PanelSlider.qml` y `PanelButton.qml`) como hijo de `compositorWindow.contentItem`, así que dibuja sobre todo, Netflix incluido (comprobado con captura). Los objetos que el cargador da a los filtros (`volume`, `overlayView`…) son proxies sin `parent`; el árbol visual se alcanza por `compositorWindow`.
- **Comportamiento:**
  - ⚙ corto abre o cierra el panel; mantenido 0,6 s abre los ajustes de LG (`com.palm.app.settings`, se superpone como antes).
  - Con el panel abierto: las flechas mueven el foco; Izquierda/Derecha cambian brillo o volumen de 5 en 5; OK activa «Apagar pantalla» o «Más ajustes»; Atrás/Salir cierran.
  - Las teclas de volumen pasan a LG y el deslizador se actualiza. Cualquier otra tecla cierra el panel y sigue su camino.
  - Se cierra solo a los 10 s.
  - El puntero también vale: clic, arrastrar en los deslizadores y clic fuera para cerrar.
- **Datos:**
  - Los niveles se leen al abrir con `lunaCall`, que tiene la identidad del compositor (el audio sí le responde).
  - Las escrituras se agrupan en una cada 80 ms.
  - Una lectura que llega después de una escritura se descarta.
  - Un nivel aún desconocido no se mueve, para no partir de 0.
  - El bus del compositor devuelve números como texto (`"40"`), así que se leen con `parseInt`.
- **Diseño:** el del Centro de control de HOME: unidades de 1280×720 escaladas, Plus Jakarta Sans y Lucide cargadas con `FontLoader`, y animaciones de 240/160 ms. Sobre vídeo el vidrio es más oscuro (`#b80e141e`). Cerrado no se dibuja (`visible: false`).
- **Coste medido:** la memoria residente de `surface-manager` no cambia (96,5 MB abierto o cerrado). La CPU se queda en la base del compositor (3–5 ticks en 5 s) con el panel abierto y quieto.
- **Seguridad:** cada entrada del filtro captura sus errores y devuelve `NextPolicy`. Si el QML no carga, ⚙ vuelve a ser de LG. La primera instalación guarda la lista original en `/var/lib/home-os/keyFilters.orig.json`.
- **Órdenes:** `scripts/lg-panel.sh install | uninstall | test TECLAS… | shot ARCHIVO`. `install` quita y vuelve a añadir el filtro, porque el compositor solo recarga si la lista cambia. `test` reproduce teclas por el manejador real a través de la configuración `com.webos.surfacemanager.homePanelTest`, y lee el estado en `…homePanelState`. Es la forma de probar sin inyectar teclas.
- **Pruebas hechas el 2026-11-17:**
  - abrir, navegar los cuatro controles y cerrar con Atrás;
  - brillo 40 → 45 → 40 comprobado en la tele, y volumen ±5;
  - 12 pulsaciones seguidas acaban en el valor exacto (0 y 60);
  - cierre automático a los 10 s;
  - ⚙ mantenido abre los ajustes de LG;
  - persiste tras `reboot`;
  - se dibuja sobre HOME, el Home de LG y Netflix.

  Comprobado después sobre Netflix reproduciendo, con el mando físico del usuario.

**Arreglos tras probarlo con el mando (2026-11-17):**

- **`CursorShow`/`CursorHide`.** El Magic Remote los manda como teclas al pasar de puntero a botones, justo después de ⚙. Cerraban el panel al instante; ahora se ignoran. Con el panel abierto, la tecla nunca llega a la app aunque el filtro falle (un OK pausaba Netflix).
- **Caché de QML.** El motor del compositor guarda los componentes por ruta, así que reinstalar en el mismo sitio seguía usando el QML viejo. `lg-panel.sh install` instala cada versión en `/var/lib/home-os/panel/<hash>/` y borra las anteriores.
- **Volumen con la barra Sonos por HDMI ARC.**
  - Las teclas de volumen del mando **no pasan por el compositor** ni por el volumen de la tele: no aparecen en los filtros ni en `mrcu/getButtonHistory`, y la tele no tiene códigos IR universales configurados (`ir/getCurrentSavedCodeSetId` → `0xff`).
  - Ni `audio/master/setVolume`, ni `master/volumeUp` más `volumeKeeping` (lo que hace la tecla en el compositor), ni `com.webos.audio/volumeUp` mueven la Sonos. El usuario lo comprobó de oído; solo cambiaba el contador de la tele.
  - Solución: el panel lee y fija el volumen de la Sonos por su API local UPnP (`http://<ip>:1400/MediaRenderer/RenderingControl/Control`, `GetVolume`/`SetVolume`) con `XMLHttpRequest`, que funciona en el QML del compositor. El paso es de 2 en 2.
  - Mientras el panel está abierto pregunta a la Sonos cada segundo, para seguir las teclas del mando. `install` busca la barra en la red (puerto 1400 con `modelName` Arc/Beam/Ray/Playbar) y escribe su IP en el filtro. Hoy: Arc en `192.168.0.169`, Sub en `.175`.
  - Si la salida no es `external_arc` se usa el volumen de la tele, de 5 en 5.
- **Una pulsación antes de saber el nivel** se guarda y se aplica cuando llega la lectura, en vez de perderse.
- **Apagar pantalla:** actúa 300 ms después de soltar OK. Antes, al soltar OK la pantalla volvía a encenderse.
- **OK sobre Brillo** alterna `picture/energySaving` entre `auto` y `off` (se ve «Auto · N %»). Mover el brillo a mano lo pone en `off`.
  - Hay sensor de luz: `pqcontroller/getEyeqSensorData` → `{"backlightValue":100,"Eyeqsensorcheck":true}`.
  - Pero el usuario no vio cambio con `auto` partiendo de 100 % en una sala a oscuras. Queda pendiente averiguar qué mide `backlightValue` y, si sirve, hacer el automático desde el panel.
- **Cursor:** al abrir, el panel pone `compositorWindow.cursorVisible = false`.
- **Silencio.** OK sobre Volumen silencia o devuelve el sonido. Con la Sonos usa `RenderingControl` `SetMute`/`GetMute`; con la tele, `audio/master/muteVolume {"mute":…}`. Mover el volumen quita el silencio.
- **Tarjeta «Ahora»** (`PanelFormats.qml`), permanente arriba del panel. Insignias de formato al estilo de la app Apple TV: los formatos premium (Dolby Vision, Atmos) van rellenos.
  - **Imagen:** `com.webos.service.videooutput/getStatus`, entrada `MAIN` → `videoInfo.hdrType` (`DolbyVision`, `HDR10`, `HLG`, si no SDR), `height` (2160 → 4K) y `frameRate` (23.976 → 24p). El modo sale de `picture/pictureMode` sin el prefijo `hdr`/`dolbyHdr`, con nombres en español.
  - **Sonido:** con la Sonos, `DeviceProperties#GetZoneInfo` → `HTAudioIn`, traducido con la tabla de la librería SoCo (por ejemplo, `84934714` es Dolby Digital Plus 5.1). Con otra salida se muestra cuál es (altavoces, Bluetooth, óptico, ARC).
  - Se lee una vez al abrir.
- **Brillo, lo que hay (2026-11-17):**
  - Los ajustes de imagen se guardan aparte por entrada y por tipo de contenido (`dimension`: `input`, `dynamicRange` sdr/hdr/dolbyHdr, `pictureMode`). El panel cambia el del contenido actual.
  - En Dolby Vision, `energySaving: auto` no atenúa.
  - No hay ajuste controlable de brillo por sensor: `eyeq` es solo un indicador de «modificado», y `ambientLightCompensation` no existe en la C1. `getEyeqSensorData` siempre devuelve `backlightValue: 100`.
  - Palancas disponibles: `pictureMode` (21 modos), `peakBrightness` y `dynamicContrast` (off/low/medium/high), `eyeComfortMode`, y `energySaving` (auto/off/min/med/max).
- **Modo de imagen** (fila con ‹ ›). Recorre la lista de su tipo de contenido y salta los modos que la tele rechaza.
  - SDR acepta `vivid normal eco cinema sports game filmMaker expert1 expert2`; `photo` se rechaza (comprobado).
  - Los modos `dolbyHdr*` solo se aceptan con contenido Dolby Vision en pantalla.
- **Modo noche** (botón con disco azul y horario debajo):
  - Guarda los valores del usuario de `backlight`, `peakBrightness`, `dynamicContrast` y `eyeComfortMode`, y aplica brillo mín(actual, nivel de noche; 0 por petición del usuario), pico off, contraste dinámico off y confort visual on.
  - Al apagarlo devuelve los valores (comprobado: low/off vuelven exactos).
  - La tele **solo escribe el modo que está en pantalla**: `setSystemSettings` con `dimension` explícita devuelve `errorKey`. Por eso se trabaja por modo, con la clave `input|dynamicRange|pictureMode`, cada vez que ese modo aparece (al abrir el panel, al cambiar de modo o cada minuto).
  - El estado y los valores guardados persisten en `com.webos.surfacemanager.homePanelNight`.
- **Horario:** `com.webos.surfacemanager.homePanelSchedule` (`lg-panel.sh night 22:00 07:00 0`, o `night off`).
  - Se enciende y apaga al cruzar el inicio o el final. Un cambio manual se mantiene hasta el siguiente cruce.
  - El primer chequeo tras cargar actúa si toca: el 2026-11-17 a las 22:54 se encendió solo, como debía.
  - La comprobación de cada minuto no llama a nada si el modo noche está apagado y no hay valores guardados.
- **Diseño aplicado (2026-11-17)**, de `docs/design/panel-rapido-lg.md`.
  - Orden: tarjeta «Ahora» (sin el modo), Brillo, Modo noche (ancho completo), Volumen, Modo de imagen, y Apagar pantalla | Más ajustes.
  - Foco por tabla de filas, sin dar la vuelta y recordando la mitad usada. El foco lleva halo de 3 u y escala 1,03.
  - Encendido es un disco relleno, nunca la cápsula blanca. El vidrio está al 78 % (`#c70e141e`).
  - Modo de imagen: el nombre cambia al momento y el modo se escribe 600 ms después de la última pulsación, con puntos de posición. No actúa hasta conocer el modo, y una lectura no deshace un cambio en curso.
  - El usuario quitó el botón «Horario» (el horario sigue solo, con `lg-panel.sh night`) y la velada oscura lateral.
  - Un cambio manual del modo noche se guarda (`manual: true`) y dura hasta el siguiente cruce del horario, también tras reiniciar.
- **Material Liquid Glass (2026-11-17)**, diseño de Figma, página «10 · Panel rápido (⚙)».
  - `PanelGlass.qml` es un `ShaderEffect` por control. Dibuja la forma con SDF, el tinte, la banda de lente, el filo especular iluminado desde arriba (al 55 % con el modo noche), una sombra interior y la de profundidad. Los colores se pasan premultiplicados a mano (`vector4d`).
  - **Sobre gráficos** (`frost` 1) usa una instantánea compartida de lo que hay detrás de la columna. Es un `ShaderEffectSource` de la raíz de vistas, que no contiene el panel, a media resolución, con `live: false`. Se toma al abrir y cada 1 s, y pasa por `FastBlur` con radio 32.
  - **Sobre vídeo** (`frost` 0, cuando `videooutput` tiene una salida `MAIN` conectada) no hay desenfoque, porque el vídeo es un plano de hardware: tinte de 70 a 80 %, filo y sombras.
  - **La línea de botones redondos** sustituye a la fila grande del modo noche, que el usuario pidió más abajo y más pequeña.
  - Coste medido con el panel abierto sobre vídeo: `surface-manager` pasa de unos 8 a unos 30 ticks en 5 s (≈ 6 % de un núcleo), con las lecturas de la Sonos incluidas, y sin cambio de memoria.
  - **Falta comprobar el modo esmerilado sobre HOME**, y si la instantánea de una app web sale negra.
- **Brillo real con Auto.** `com.webos.service.pqcontroller/getInfo {"key":"backlight"}` da el nivel que aplica el driver.
  - Medido con el ajuste a 45: `energySaving` `off` → 45; `auto` en sala oscura → 23 y luego 21; `max` → 0. El ajuste `backlight` no cambia en ninguno.
  - Con Auto activo, el panel pide ese valor cada 2 s mientras está abierto y muestra «Auto · 21 %». Al desactivar Auto vuelve al nivel de antes (comprobado: 45).
  - Con el modo noche activo, OK sobre Brillo muestra 1,5 s «Primero desactiva Modo noche».
  - Con el modo noche activo, mover el brillo guarda el nivel de noche.
- **Cursor al pulsar ⚙: no tiene solución sin tocar otros botones.**
  - Lo muestra a propósito el servicio de entrada del mando (`lginput2`). `mrcu/getCursorHistory` dice «Cursor Show(SHOW KEY_SETTINGS Press)»; una flecha lo oculta («STANDBY»).
  - Ocultarlo desde el compositor (`compositor.cursorVisible`, `compositorWindow.cursorVisible`, tragarse `CursorShow`) no sirvió.
  - El interruptor es `mrcu/setHotkeyCursorPolicy {"button": <número>, "cursorPolicy": 0|1|2}`, pero el número de `KEY_SETTINGS` no está documentado. Al final el usuario pidió probarlo: un barrido puso `cursorPolicy: 1` en los botones 203–1023 (todos devolvieron `returnValue: true`, así que la respuesta no dice cuál es ⚙). **Sin verificar:** falta pulsar ⚙ y ↓ y leer `getCursorHistory`; los reinicios posteriores pueden haber devuelto las políticas a su valor de fábrica.
  - Leer `/dev/input/event*` no sirve: `lginput2` se queda el dispositivo y no llega nada.

**Incidente (2026-11-17).** Un `grep -r` sobre `/var/palm/jail` se quedó colgado y, unos dos minutos después, la tele se reinició sola con `PowerOnReason: cpuAbnormal` (el vigilante del sistema). Lección: **en la tele nunca hagas búsquedas recursivas en carpetas del sistema** (`/var/palm`, `/proc`, `/dev`, `/media`). Usa rutas concretas y comandos acotados.

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
- Si los ajustes de LG (por ejemplo `sportsAlarm`) hacen salir su proceso al apagarlos.
- Puntero, rueda y teclas numéricas del Magic Remote en el port.
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
