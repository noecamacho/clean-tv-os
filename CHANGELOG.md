# Changelog

Los cambios de HOME, del más reciente al más antiguo. El formato sigue
[Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [Sin publicar]

### Añadido

- **HOME para la LG OLED55C1PSA (webOS 6.5.3, rooteada):** crate `crates/lume-webos` (SDL2 por
  FFI, GLES2 de `lume-core`, puente al bus Luna), paquete aparte `app.home.tv` 0.1.0,
  `scripts/webos-build.sh`, `docker/Dockerfile.webos` y el mando virtual `scripts/lg-key.sh`.
  Corre en la tele con la lista real de apps; faltan fondo, iconos y entradas. El APK de la TCL
  no cambia.
- `docs/llm-wiki/lg-webos.md`: root, sustitución del Home, estado del port y diseño híbrido
  HOME × Luma (Figma, página 06).

## [0.9.7] - 2026-11-15

### Corregido

- Cuatro fondos tenían un nombre que no correspondía a su imagen: ahora son «Valle entre
  montañas», «Castillo en la niebla», «Bosque brumoso» y «Montaña al atardecer» (en los cinco
  idiomas).
- La descripción del editor hablaba del «dock»; ahora dice «fila principal».

## [0.9.6] - 2026-11-15

### Añadido

- **HOME vuelve siempre a su punto de partida.** Al salir a otra app o suspender la tele queda
  en Inicio, plegado, con la primera app en foco y sin paneles ni menús abiertos. Ya no
  restaura el lugar desde el que se abrió una app.
- **Tiempo del modo ambiente configurable** en Ajustes › Pantalla de inicio: 5, 10 o 30 minutos,
  o desactivado.
- **OK sobre el deslizador de Brillo** activa el brillo automático donde la tele lo permite
  (`setLightSensorOnoff`). La 55Q5K acepta la orden pero no enciende el sensor, y HOME abre
  entonces el panel de imagen.
- `docs/llm-wiki/tcl-apis.md`: mapa de las librerías de TCL y Realtek de la tele y lo comprobado.

### Cambiado

- El oscurecido del Centro de control va en la pasada del fondo y los banners se atenúan uno a
  uno: dibujar el Inicio debajo ya no cuesta cuadros (de 12 lentos de 122 a 1–2).

### Eliminado

- La sección de fondos y de seis por fila del editor, con su código.

## [0.9.3] - 2026-11-15

### Corregido

- **El brillo del Centro de control ahora mueve la retroiluminación real.** En esta TCL el brillo
  de Android (`Settings.System.SCREEN_BRIGHTNESS`) no llega al panel; HOME usa la librería de
  imagen de la tele (`com.tcl.tv.display`, `TvBacklightManager.getBacklight/setBacklight`) por
  reflexión, y deja el ajuste de Android solo como respaldo en otras marcas.
- **El Centro de control sigue el volumen y el brillo en vivo.** Mientras está abierto vuelve a
  leerlos cada 300 ms, así que refleja las teclas de volumen del mando.

## [0.9.2] - 2026-11-15

### Añadido

- **Modo ambiente propio.** Tras 10 minutos sin teclas en Inicio, o con Atrás sobre la primera
  app, queda el fondo entero con la hora; cualquier tecla devuelve Inicio y no hace nada más.
  Antes Atrás abría la pantalla de salvapantallas de Google TV.

### Cambiado

- **Un solo estilo en todas las pantallas.** El editor, el mapeo de control, la información del
  televisor y Acerca de llevan el título centrado como Ajustes (sin migas), paneles de vidrio
  oscuro y foco blanco con sombra, sin anillo.
- El editor ya solo ordena la fila principal: fondo y apps por fila tienen su página en Ajustes.
- El Centro de control se dibuja sobre el Inicio atenuado, y Wi‑Fi y Bluetooth dicen
  «Activado» o «Desactivado».
- Arranque en frío más corto: espera de 300 ms y revelado de 360 ms (antes 600 y 600).
- Los avisos son una cápsula de vidrio oscuro.

### Corregido

- «Acerca de» enlazaba al repositorio del proyecto original.

## [0.9.1] - 2026-11-15

### Cambiado

- **Ajustes con la estructura de tvOS.** Título centrado, emblema de vidrio a la izquierda con
  la versión debajo y la lista de filas a la derecha; la fila con foco se vuelve blanca, crece
  y proyecta sombra. La lista reúne lo de HOME (Pantalla de inicio, Fondo de pantalla, Mapeo de
  control, Información del televisor, Acerca de) y, bajo «Ajustes de la tele», Imagen, Sonido,
  Red y Todos los ajustes.
- **Fondo de pantalla** es una página propia: la pantalla entera muestra, sin atenuar, el fondo
  de la fila con el foco, y OK lo elige. **Pantalla de inicio** es otra: el interruptor de seis
  por fila y la entrada al editor de la fila principal.
- Atrás en Ajustes vuelve a Inicio (antes llevaba a la antigua pestaña Apps).
- El icono del temporizador del Centro de control es un reloj.

### Eliminado

- La tarjeta de estadísticas (memoria, disco, CPU) de la pantalla de Ajustes, y su lectura
  periódica.

## [0.9.0] - 2026-11-15

### Cambiado

- **Inicio como tvOS.** En reposo el fondo se ve nítido y entero, y solo aparece la fila
  principal en su placa de vidrio contra el borde inferior. Abajo desde esa fila despliega la
  biblioteca: todas las filas suben de una vez, la placa queda arriba y el fondo se desenfoca
  y oscurece; Arriba en la segunda fila lo pliega. El nombre de la app solo se muestra en la
  biblioteca y la cápsula de la hora se desvanece al desplegar.
- **Centro de control nuevo.** Suspender, Wi‑Fi, Bluetooth, deslizadores de Brillo y Volumen
  que se mueven con izquierda/derecha, AirPlay, Temporizador de reposo (15, 30, 60, 120 min) y
  una fila de botones redondos (Entradas, Duplicar pantalla, Imagen, Sonido, Ajustes de HOME,
  Ajustes de Android). Se abre con el foco en Wi‑Fi, no en Suspender.
- **Volver a HOME es inmediato.** Al salir a otra app se conservan el contexto OpenGL, los
  programas, las texturas, los glifos y los iconos (se sueltan si el sistema avisa de poca
  memoria). Medido en la TCL: de ~1,2 s a ~45 ms desde la pulsación hasta el primer cuadro
  completo, y sin splash.
- El botón Home abre HOME al pulsar, no al soltar.
- El hilo que dibuja y atiende el mando corre con prioridad −10.
- El sonido de foco se pide después de dibujar el cuadro, no antes.
- Revelado tras un arranque tibio: 180 ms (antes 360).

### Añadido

- Permiso `WRITE_SETTINGS` (para el brillo; se concede con
  `adb shell appops set app.lume.tv WRITE_SETTINGS allow`). Sin él, el deslizador abre el panel
  de imagen de la tele.
- Suspender y temporizador mediante la acción global del servicio de accesibilidad.
- Iconos Lucide `power`, `sun`, `airplay`, `timer`, `cast`.

## [0.8.2] - 2026-11-15

### Añadido

- Motor gráfico Vulkan 1.3 opcional (`crates/lume-app/src/android/vulkan.rs`): dibuja los mismos
  cuadros que el de OpenGL ES, con *dynamic rendering*, *push descriptors*, redibujado parcial
  (`VK_KHR_incremental_present`) y caché de *pipelines* en disco. Se activa con
  `adb shell setprop debug.lume.renderer vulkan` y vuelve a OpenGL ES al reiniciar la tele.
  Los shaders están en `crates/lume-app/shaders/` y se compilan con `scripts/build-shaders.sh`.

### Sin cambios

- OpenGL ES 3.2 sigue siendo el motor por defecto. Medido en la TCL 55Q5K (Android 14), Vulkan
  queda empatado en fluidez (16,7 ms de mediana en navegación, desplazamiento y Centro de
  control con los dos) y tarda más en arrancar en frío, así que no aporta nada visible.

## [0.7.2] - 2026-11-11

Sin cambios de funcionamiento respecto a 0.7.1. Se probó renderizar a 3840×2160 y se descartó:
la navegación pasó de ningún intervalo lento a 27 de unos 120, y el desplazamiento y el Centro
de control bajaron a 33 ms de mediana. El número de versión sube solo porque la compilación de
prueba ya lo había usado en la tele.

## [0.7.1] - 2026-11-11

### Añadido

- Opción «Seis iconos por fila» en Ajustes de HOME > Pantalla de inicio: el tamaño
  predeterminado de tvOS. Cinco por fila (el «Grande» de tvOS) sigue siendo el de HOME.
- La cápsula del reloj muestra el símbolo de Wi-Fi en naranja cuando el Wi-Fi está apagado.

### Eliminado

- La opción «Favoritas redondas», que no tenía efecto desde que Inicio dejó de usar el dock de
  Lume; su interruptor es ahora el de seis iconos por fila.

## [0.7.0] - 2026-11-11

Más reglas de la guía de tvOS (`docs/llm-wiki/tvos-hig.md`, puntos 4, 5, 6, 8 y 9).

### Añadido

- Al bajar por la rejilla el fondo se desenfoca, además de oscurecerse.
- Accesibilidad tomada de los ajustes de la tele: con las animaciones de Android apagadas no hay
  rebote, inclinación, banda de luz, temblor ni desplazamientos (solo fundidos); con «texto de
  alto contraste» el vidrio pasa a ser casi opaco.
- Sonido de foco propio (`res/raw/focus_tick.wav`, generado para este proyecto): suena desde el
  lado de la pantalla donde está el foco y más bajo cuando el foco corre. Solo si los sonidos
  del sistema están activados.
- Atrás sobre la primera app abre el modo ambiente de la tele, como tvOS abre su salvapantallas.

### Cambiado

- Tipografía más cercana a la escala de tvOS: nombre de app a 20 unidades y peso Medium; hora y
  mosaicos en Medium.

### Corregido

- El nombre bajo el banner enfocado quedaba cortado en la fila inferior al desplazarse.

## [0.6.1] - 2026-11-11

Cuatro reglas de la guía de diseño de tvOS (véase `docs/llm-wiki/tvos-hig.md`).

### Añadido

- Reposo: tras diez segundos sin teclas en Inicio, lo que no tiene el foco se atenúa y el banner
  enfocado crece un poco; cualquier tecla lo deshace.
- Estado de pulsación: mientras OK está pulsado, el banner o el mosaico enfocado se hunde.

### Cambiado

- Tomar el foco es más rápido que dejarlo (×1,4), y ambos se acortan (×0,6) cuando el foco se
  mueve seguido.
- Menú contextual: las acciones que no aplican se omiten en vez de mostrarse apagadas, y las
  que quitan algo («Quitar de Inicio», «Desinstalar») van al final.

## [0.6.0] - 2026-11-11

### Añadido

- Contexto OpenGL ES 3.2 donde la tele lo tiene (3.0 y 2.0 como alternativas). Los shaders
  siguen en GLSL ES 1.00, que todas las versiones compilan.
- Inclinación con perspectiva real del banner que recibe el foco (`push_tilt` en la lista de
  dibujo; la GPU interpola en perspectiva con la `w` de cada vértice).

### Cambiado

- El vidrio se difumina con cuatro pasadas a 240×135 al cargar el fondo: ya no se ven manchas.
- El velo inferior de Inicio y el oscurecido al bajar van en la propia pasada del fondo (color
  por vértice) y la plataforma es una sola pasada de vidrio. El desplazamiento vertical pasó de
  33 ms a 16,6 ms de mediana en la TCL 55Q5K.
- El velo del Centro de control ya no se anima: abrirlo y cerrarlo va a 60 cuadros.

### Corregido

- El botón Home creaba una segunda instancia de HOME en una tarea aparte (el servicio la
  lanzaba como MAIN/HOME), y esa primera pulsación no hacía nada.

## [0.5.1] - 2026-11-11

### Añadido

- Los mosaicos de Wi-Fi y Bluetooth muestran su estado real con un disco azul cuando están
  encendidos.
- El banner que recibe el foco se inclina un instante hacia donde se movió el foco.
- Sonido de navegación del sistema al mover el foco; solo suena si los sonidos del sistema
  están activados en la tele.

### Cambiado

- El menú contextual pasa de tarjeta clara a vidrio oscuro con texto blanco.

### Corregido

- Home abría Inicio en vez del Centro de control mientras un menú terminaba de cerrarse.

## [0.5.0] - 2026-11-11

### Añadido

- Vidrio esmerilado: las superficies de vidrio muestran una copia pequeña del fondo, difuminada
  una sola vez al cargarlo, con lente en el borde y brillo especular en el contorno.
- Pulsación larga de OK (medio segundo) para abrir el menú del icono enfocado; OK actúa al
  soltar.
- Al mover un favorito, los favoritos tiemblan como en el modo edición de tvOS.
- Una banda de luz cruza el banner cuando recibe el foco.
- Las apps se abren creciendo desde su banner.
- El Centro de control se cierra con animación hacia su esquina y devuelve el foco a donde
  estaba; nuevo mosaico «Ajustes de HOME».
- Al bajar por la rejilla, el fondo se oscurece.

### Cambiado

- Inicio ya no tiene barra de pestañas: arriba a la derecha queda una cápsula con la hora que,
  con el foco, abre el Centro de control.
- La plataforma de la primera fila usa un radio concéntrico con el de los banners.

### Eliminado

- El acceso a la pantalla Apps y a su barra superior desde Inicio (Inicio ya lista todas las
  apps). El código de esa pantalla sigue en el repositorio, sin entrada desde la interfaz.
- El mosaico «Apps» (ajustes de aplicaciones de Android) del Centro de control.

## [0.4.0] - 2026-11-11

Primera versión con historial propio. El código parte de
[douglaspands/lume-tv-launcher](https://github.com/douglaspands/lume-tv-launcher) en su commit
`2e7421d` (GPL-3.0-or-later); la documentación de aquel proyecto se conserva en `docs/upstream/`.

### Añadido

- Identidad HOME: nombre, icono y banner propios, sin marca Lume en la interfaz. Los
  identificadores internos (`app.lume.tv`, crates `lume-*`) se conservan para actualizar la app
  instalada.
- Tres fondos de playa originales (Cala turquesa, Atardecer, Amanecer) en QOI con miniaturas.
- Vidrio aproximado: una muestra refractada del fondo por píxel, sin pasada de blur.
- Centro de control: mosaicos de vidrio flotantes arriba a la derecha con Wi-Fi, Bluetooth,
  Entradas, Imagen, Sonido, Modo ambiente, Apps y Ajustes. Cada uno abre el ajuste de la tele.
- `HomeKeyService`: servicio de accesibilidad opcional con el que el botón Home y el encendido
  abren HOME sin cambiar el launcher del sistema. Con HOME ya en Inicio, Home abre y cierra el
  Centro de control.
- Redibujado parcial: cada cuadro repinta solo lo que cambió (`EGL_EXT_buffer_age`,
  `EGL_KHR_partial_update` y `glScissor`).
- Wiki para agentes en `docs/llm-wiki/` y contexto compilado `llms-full.txt`.

### Cambiado

- Inicio al estilo tvOS: cinco banners por fila, la primera sobre una plataforma de vidrio; el
  banner enfocado crece a 1,12× con un pequeño rebote y sombra amplia, y solo él muestra su
  nombre.
- Render a 1920×1080 en lugar de 1280×720. En la TCL 55Q5K la navegación mide 16,7 ms de
  mediana y 17,0 ms de p95 entre cuadros.
- Los banners se guardan a 320×180, el tamaño en que las apps de Android TV los publican.

### Eliminado

- README traducidos y logotipo de Lume.
