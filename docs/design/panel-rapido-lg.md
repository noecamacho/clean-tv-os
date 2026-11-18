# Panel rápido de la LG · propuesta de diseño

Propuesta de 2026-11-17 para el panel que abre la tecla ⚙ del Magic Remote en la OLED55C1PSA (webOS 6.5.3, rooteada). Parte del panel actual (`crates/lume-webos/panel/`) y del Centro de control de HOME (`crates/lume-core/src/app/control_center.rs`). Añade tres funciones que se están programando ahora: **Modo noche**, su **horario** y el **selector de modo de imagen**.

Maqueta: [`panel-rapido-lg.html`](panel-rapido-lg.html). Son cuatro estados a 1920×1080, con una leyenda bajo cada uno.

Todas las medidas van en unidades de 1280×720 (`u`). En la tele, 1 u = 1,5 px. Los identificadores técnicos van en inglés.

## 1. Límites que manda el compositor

- **QtQuick 2.4, nada más.** No hay `QtQuick.Controls` ni `layer.effect`, y no hay desenfoque sobre el vídeo, porque el vídeo va en un plano de hardware. El «vidrio» es un relleno oscuro translúcido. Las sombras y los halos se hacen con `Rectangle` superpuestos. Los degradados son verticales, así que uno horizontal es un `Rectangle` girado 90°.
- **Entrada.** El D-pad (Arriba/Abajo/Izquierda/Derecha/OK/Atrás) es la entrada principal. El puntero también funciona: pasar por encima, clic, arrastrar y rueda.
- **Cierre automático** a los 10 s sin actividad.
- **Lo habitual de fondo:** una película de Netflix en Dolby Vision, a oscuras. Ese es el caso que manda.

## 2. Jerarquía: qué importa viendo una película de noche

Lo que se hace con el panel mientras corre una película, de más a menos frecuente:

1. **Bajar o subir el brillo.** Es el motivo original del panel.
2. **Modo noche.** Es «el brillo, pero bien hecho»: una pulsación cambia cuatro ajustes. Debe estar pegado al brillo.
3. **Volumen y silencio.** Las teclas del mando ya mueven la Sonos, así que el deslizador sirve sobre todo para ver el nivel y para silenciar.
4. **Modo de imagen.** Se cambia poco y es arriesgado de noche, porque pasar por «Vívido» deslumbra.
5. **Apagar pantalla** y **Más ajustes.**

La tarjeta «Ahora» se queda arriba, pero solo informa y no recibe el foco. **El modo de imagen sale de la tarjeta** y pasa a su propio control, para no mostrarlo dos veces. A la derecha de la fila de sonido sigue el aparato («Sonos Arc»).

### Columna propuesta

La columna es la de HOME: 456 u de ancho, x = 770, y = 92, separación de 10 u y cápsulas de 69 u. La mitad mide (456 − 10) / 2 = 223 u.

| # | Fila | Alto | Ocupa | Foco |
|---|------|------|-------|------|
| 0 | Tarjeta «Ahora» (imagen y sonido) | 112 | 92–204 | no |
| 1 | **Brillo** (deslizador) | 69 | 214–283 | sí, **al abrir** |
| 2 | **Modo noche** ½ · **Horario** ½ | 69 | 293–362 | sí |
| 3 | Volumen (deslizador) | 69 | 372–441 | sí |
| 4 | Modo de imagen (selector en línea) | 69 | 451–520 | sí |
| 5 | Apagar pantalla ½ · Más ajustes ½ | 69 | 530–599 | sí |

La columna acaba en 599 u y deja 121 u libres abajo, donde suelen ir los subtítulos de Netflix. **Ningún control tapa la mitad izquierda de la imagen.**

## 3. Modelo de foco y caminos del D-pad

El foco recorre una **tabla de filas**, como `focus_rows` en `control_center.rs`, en lugar de la aritmética de índices de `home-panel.js` (`f >= 2 ? 1 : …`). Las filas de dos mitades recuerdan qué mitad se usó la última vez.

| Control | Izquierda / Derecha | OK | Arriba / Abajo |
|---|---|---|---|
| Brillo | −5 / +5. Mover el brillo a mano apaga Auto | alterna **Auto** (`energySaving` auto/off) | Arriba: se queda (no da la vuelta). Abajo: a Modo noche |
| Modo noche | Derecha: a Horario. Izquierda: se queda | **activa o desactiva** el modo noche | Arriba: Brillo. Abajo: Volumen |
| Horario | Izquierda: a Modo noche. Derecha: se queda | abre la hoja **Horario** (en el primer paso, alterna el horario) | Arriba: Brillo. Abajo: Volumen |
| Volumen | −2 / +2 con la Sonos, −5 / +5 con la tele. Moverlo quita el silencio | **silencia** o devuelve el sonido | Arriba: la última mitad usada de la fila 2. Abajo: Modo de imagen |
| Modo de imagen | modo **anterior / siguiente** de la lista del contenido actual, sin dar la vuelta | abre la hoja **Modo de imagen** (*should*). Sin hoja, no hace nada | Arriba: Volumen. Abajo: fila 5 |
| Apagar pantalla | Derecha: a Más ajustes | apaga la pantalla 300 ms después de soltar OK (como ahora) | Arriba: Modo de imagen. Abajo: se queda |
| Más ajustes | Izquierda: a Apagar pantalla | abre los ajustes de LG | igual |

**Siempre:**

- **Atrás** y **Salir** cierran el panel. Dentro de una hoja, Atrás vuelve al panel con el foco en el control que abrió la hoja.
- **⚙ corto** cierra el panel. **⚙ mantenido** 0,6 s abre los ajustes de LG.
- Las teclas de volumen pasan y el deslizador se actualiza. Cualquier otra tecla cierra el panel y sigue su camino (como ahora).

**Sin callejones.**

- Todo control se alcanza en cuatro pulsaciones como mucho desde Brillo.
- En los bordes el foco se queda donde está. No salta al otro extremo.
- Al llegar al final de un deslizador o de la lista de modos, el chevrón de ese lado se apaga al 30 %. Así el usuario entiende por qué no pasa nada.

**Puntero.**

- Pasar por encima mueve el foco.
- En los deslizadores, clic o arrastre fija el nivel y la rueda lo mueve de 5 en 5, como ahora.
- En el selector, los chevrones son zonas de clic de 48 u y la rueda cambia de modo.
- Un clic fuera del panel lo cierra.

## 4. Modo de imagen: selector en línea, con una hoja como extra

**Decisión: el selector va en línea (Izquierda/Derecha), igual que los deslizadores.** Una hoja con la lista completa queda como mejora (*should*).

Por qué en línea:

- **Es coherente.** En este panel, Izquierda/Derecha siempre cambia el valor del control enfocado.
- **Basta para la tarea habitual.** Lo normal es pasar a un modo vecino («Vívido → Cine»), y eso son una o dos pulsaciones.
- **La lista es corta.** Depende del contenido: unos 6–10 modos por tipo, no los 21 de la tele.

**Retardo antes de aplicar.** Cada cambio de modo recarga la imagen y trae su propio brillo, porque los ajustes se guardan por `dimension`. Si cada pulsación aplicara el modo al momento, al pasar por «Vívido» de camino a «Filmmaker» la sala se iluminaría de golpe. Por eso:

1. **Izquierda/Derecha solo cambian el nombre** que se ve. El texto entra deslizándose 8 u desde el lado de la pulsación, en 120 ms.
2. **Tras 600 ms sin más pulsaciones** se escribe `pictureMode`. Mientras tanto, el nombre se ve completo y atenuado (en la tele se probó «Cine …» y parecía un nombre cortado).
3. **Al confirmarse**, el panel vuelve a leer el brillo, porque cambió con el modo. El deslizador de Brillo muestra «…» hasta que llega la lectura.

**Lo que se ve.**

- Glifo `tv` y «Modo de imagen» a la izquierda.
- A la derecha, `‹ Cine ›`, con puntos de posición debajo (3 de 6). La posición también se dice en texto para lectores: «Cine, 3 de 6».
- **El contexto va en la tarjeta «Ahora».** Si el contenido es Dolby Vision, la tarjeta muestra la insignia DOLBY VISION. Así el usuario entiende por qué la lista es otra.

**La lista** sale de lo que acepte la tele para el `dynamicRange` actual (sdr / hdr / dolbyHdr). La lista por tipo ya existe en `home-panel.js` (`HOME_PANEL_MODE_LISTS`: 9 en SDR, 6 en HDR y 6 en Dolby Vision) y los nombres en español salen de `HOME_PANEL_MODES` (Vívido, Estándar, Cine, Cine en casa, Filmmaker…). Si la tele rechaza un modo, el selector lo salta y la cuenta «3 de 6» se corrige.

**Hoja «Modo de imagen»** (*should*). OK sobre el selector cambia el contenido de la columna, en el mismo sitio:

- Una cabecera, que no recibe el foco: `‹  Modo de imagen · Dolby Vision`.
- Una fila de 56 u por modo, con una marca `check` en el actual.
- Arriba/Abajo recorren la lista.
- **OK aplica y vuelve al panel.** En la hoja no hay retardo, porque el usuario elige a propósito.
- Atrás vuelve sin cambiar nada.
- Caben siete filas. Con más, la lista se desplaza con el foco.

## 5. Modo noche y su horario

### Modo noche (media cápsula izquierda de la fila 2)

- **Una pulsación lo activa.** Baja la luz OLED, apaga `peakBrightness` y `dynamicContrast`, y enciende `eyeComfortMode`.
- **Otra pulsación lo desactiva** y devuelve los valores guardados al activarlo.
- **No pide confirmación.** El cambio es inmediato y se deshace con la misma tecla. Una confirmación solo añadiría una pulsación a oscuras.
- **Respuesta visible.** El disco del glifo se rellena de blanco con la luna en tinta. El texto pasa de «Desactivado» a «Activado». El deslizador de Brillo baja al nivel de noche en 240 ms y su prefijo pasa a «Noche · 40 %». El usuario ve el efecto en la pantalla y en el panel a la vez.
- **Mover el brillo con el modo noche activo ajusta el nivel de noche y lo recuerda.** No sale del modo noche. Así el usuario encuentra su nivel una vez y no vuelve a buscarlo.
- **Con el modo noche activo, Auto no se aplica.** Auto no atenúa en Dolby Vision y chocaría con el nivel de noche. OK sobre Brillo deja el valor igual y, durante 1,5 s, el hueco del valor muestra «Primero desactiva Modo noche». Luego vuelve a «Noche · 40 %».

Textos de la media cápsula:

| Estado | Línea 1 | Línea 2 |
|---|---|---|
| Desactivado | Modo noche | Desactivado |
| Activado a mano | Modo noche | Activado |
| Activado por el horario | Modo noche | Hasta las 07:00 |
| Leyendo o aplicando | Modo noche | … |

### Horario (media cápsula derecha de la fila 2)

- El panel **siempre muestra el estado del horario**: glifo `clock`, «Horario» y, debajo, «22:00–07:00» o «Desactivado».
- **Primer paso (*must*):** OK alterna el horario y las horas son 22:00–07:00 por defecto.
- **Objetivo (*should*):** OK abre la hoja **Horario del modo noche**, en la misma columna.

| Fila | Control | Izquierda / Derecha | OK |
|---|---|---|---|
| Programar | interruptor, «Sí» o «No» | — | alterna |
| Empieza | selector `‹ 22:00 ›` | −30 / +30 min (mantener repite) | — |
| Termina | selector `‹ 07:00 ›` | −30 / +30 min | — |
| Nota (no recibe el foco) | «Se activa solo cada noche. Si lo quitas a mano, vuelve mañana.» | | |

- Las horas van **de 30 en 30 minutos**: son 48 valores, a lo sumo 24 pulsaciones con repetición. Escribir dígitos con el D-pad sería peor.
- **El selector de horas es el mismo componente que el de modo de imagen** (`PanelPicker`), con otra lista.
- Los cambios se guardan al momento, como en tvOS. No hay botón «Guardar».
- Con «Programar: No», los selectores se apagan al 40 % y el foco los salta.
- Mientras la hoja está abierta, el cierre automático pasa a 20 s.

**Reglas del horario:**

- A la hora de inicio, el modo noche se activa aunque el panel esté cerrado.
- Si el usuario lo desactiva a mano dentro de la franja, se queda así hasta la próxima hora de inicio.
- A la hora de fin, se desactiva y devuelve los valores guardados.
- *Could:* al activarse por horario con una app en marcha, un aviso discreto de 3 s arriba a la derecha: «Modo noche activado · hasta las 07:00».

## 6. Estados y respuesta

| Estado | Cómo se ve (nunca solo con color) |
|---|---|
| Valor aún desconocido | «…» en el hueco del valor. La barra no se mueve y una pulsación se guarda para después (como ahora) |
| Auto | prefijo «Auto · 100 %» |
| Noche | prefijo «Noche · 40 %» y luna en el disco de la cápsula de Modo noche |
| Silenciado | glifo `volume-x` en vez de `volume-2`, «Silenciado» y la barra al 35 % |
| Aplicando un modo | «Cine …» con los puntos de posición quietos |
| La Sonos no responde | «Sin conexión» en el hueco del valor. Izquierda/Derecha no hacen nada y la barra queda al 35 % |
| Final de lista o de nivel | el chevrón de ese lado al 30 % |
| Control desactivado (selectores sin horario) | todo el control al 40 % y el foco lo salta |

**Estados de los interruptores.** Encendido y foco no pueden usar el mismo blanco: en HOME el blanco ya significa «aquí está el foco».

| Estado | Cápsula | Disco | Texto |
|---|---|---|---|
| Apagado, sin foco | vidrio | disco translúcido, glifo blanco | blanco |
| **Encendido, sin foco** | vidrio | **disco blanco, glifo en tinta** | blanco, «Activado» |
| Apagado, con foco | **blanca** | disco en tinta al 10 %, glifo en tinta | tinta |
| **Encendido, con foco** | **blanca** | **disco en tinta, glifo blanco** | tinta, «Activado» |

## 7. Movimiento

Los tiempos son los de HOME. Nada dura más de 240 ms.

| Qué | Duración | Curva |
|---|---|---|
| Abrir el panel (opacidad y subida de 22 u) | 240 ms | `cubic-bezier(0.2, 0, 0, 1)` (ya está) |
| Cerrar el panel | 160 ms | ease-in (`Easing.InQuad`, ya está) |
| Foco: color de la cápsula | 120 ms | lineal (ya está) |
| Foco: escala 1 → 1,03 y halo | 120 ms | `cubic-bezier(0.2, 0, 0, 1)` |
| Nivel de un deslizador (tecla) | 120 ms | `cubic-bezier(0.2, 0, 0, 1)` |
| Nivel tras activar o desactivar Modo noche | 240 ms | `cubic-bezier(0.2, 0, 0, 1)` |
| Valor del selector (entra 8 u por el lado de la pulsación) | 120 ms | `cubic-bezier(0.2, 0, 0, 1)` |
| Entrar en una hoja (opacidad y desplazamiento de 40 u) | 240 ms | `cubic-bezier(0.2, 0, 0, 1)` |
| Volver de una hoja | 160 ms | ease-in |

- Arrastrar con el puntero no anima: la barra sigue al dedo.
- Una tecla repetida corta la animación en curso y parte del valor de ese momento. `NumberAnimation` con `restart()` ya lo hace.

## 8. Legibilidad sobre vídeo claro y oscuro

- **Vídeo claro (exteriores de día, nieve).** Hoy el vidrio es `#b80e141e` (72 %). Sobre blanco puro, el fondo resultante es ≈ rgb(81, 84, 93): el blanco da ≈ 7,6:1 y el texto secundario (`#b3ffffff`) ≈ 4,6:1, justo en el límite. Propuesta:
  1. Vidrio al **78 %** (`#c70e141e`).
  2. **Una velada horizontal** detrás de la columna: un `Rectangle` girado de 560 u de ancho, de transparente a `#73000000`. Sube el secundario por encima de 5:1 y tapa menos que oscurecer toda la pantalla.
- **Vídeo oscuro (a oscuras, de noche).** El problema es al revés: la cápsula blanca de foco (`#f0f6f8fc`) deslumbra. **Con el modo noche activo, la cápsula de foco baja a `#d9dfe4ea`** (85 %). La tinta `#101620` sigue por encima de 12:1.
- **Tamaños mínimos:** texto 15 u (22,5 px), etiquetas 17–19 u, glifos 22–24 u. Las insignias de 12 u en negrita son la única excepción. Solo muestran formatos y ya existen.
- **Borde de 1 px** `#50ffffff` en todo el vidrio, para que la columna se separe de un fondo negro.

## 9. Accesibilidad

- **El foco se ve de tres maneras a la vez:** la cápsula pasa a blanca, crece al 103 % y lleva un halo de 3 u (`#59ffffff`). Así se distingue aunque el vídeo sea muy claro y aunque el usuario no perciba bien los colores.
- **Ningún estado depende solo del color.** Cada uno tiene texto y glifo: «Activado» con disco relleno, «Silenciado» con `volume-x`, chevrones apagados. Ver la tabla de la sección 6.
- **Contraste:** tinta sobre blanco ≈ 16:1, blanco sobre vidrio ≥ 7:1, secundario ≥ 4,5:1 (con la velada, ≥ 5:1).
- **Movimiento:** solo opacidad y desplazamientos cortos. No hay rebotes ni parpadeos. El panel no anima nada si nadie toca el mando.
- **Textos completos:** «Silenciado», no un icono suelto. Los modos van con su nombre en el idioma de la tele.
- **Puntero:** zonas de clic de al menos 48 u.

## 10. Textos (es / en)

| Clave | Español | English |
|---|---|---|
| `brightness` | Brillo | Brightness |
| `auto` | Auto · {n} % | Auto · {n} % |
| `nightPrefix` | Noche · {n} % | Night · {n} % |
| `nightBlocksAuto` | Primero desactiva Modo noche | Turn off Night mode first |
| `volume` | Volumen | Volume |
| `muted` | Silenciado | Muted |
| `offline` | Sin conexión | Not connected |
| `night` | Modo noche | Night mode |
| `on` / `off` | Activado / Desactivado | On / Off |
| `nightUntil` | Hasta las {hh:mm} | Until {hh:mm} |
| `schedule` | Horario | Schedule |
| `scheduleSheet` | Horario del modo noche | Night mode schedule |
| `scheduleEnabled` | Programar | Scheduled |
| `yes` / `no` | Sí / No | On / Off |
| `starts` / `ends` | Empieza / Termina | Starts / Ends |
| `scheduleNote` | Se activa solo cada noche. Si lo quitas a mano, vuelve mañana. | Turns on by itself every night. If you turn it off, it comes back tomorrow. |
| `pictureMode` | Modo de imagen | Picture mode |
| `pictureSheet` | Modo de imagen · {tipo} | Picture mode · {type} |
| `position` | {i} de {n} | {i} of {n} |
| `screenOff` | Apagar pantalla | Turn off screen |
| `settings` | Más ajustes | More settings |
| `toast` | Modo noche activado · hasta las {hh:mm} | Night mode on · until {hh:mm} |

Los nombres de los modos de imagen ya están en `HOME_PANEL_MODES` (es / en).

## 11. Cómo construirlo con lo que hay

**Se reutiliza:**

- `HomePanel.qml`: la animación de abrir y cerrar, `FontLoader`, el clic fuera y la columna. Hay que añadir las filas nuevas y la velada (un `Rectangle` girado).
- `PanelSlider.qml`, sin cambios de forma:
  - el prefijo «Noche · » usa la propiedad `prefix` que ya existe;
  - el glifo de silencio sale del estado `muted`, que ya existe;
  - solo hace falta una propiedad `nightLevel` (o pasarle `prefix` desde fuera) y animar `fill` con `Behavior`.
- `PanelButton.qml`, para Apagar pantalla y Más ajustes. Ampliado con `sublabel` (segunda línea) y `on` (disco relleno), sirve para Modo noche y Horario, y para las filas de las hojas, con una marca opcional.
- `SliderLabels.qml`, dentro del selector, para el glifo, el nombre y el valor.
- `PanelFormats.qml`: deja de recibir `videoMode` (o la fila de imagen deja la derecha vacía).

**Componentes nuevos (todos en QtQuick 2.4 puro):**

- `PanelPicker.qml`: una cápsula de 69 u con `‹ valor ›`, puntos de posición y chevrones que se apagan en los extremos. Lo usan Modo de imagen y las horas del horario.
- `PanelSheet.qml`: una cabecera con `‹` y título, y una `Column` de filas. Entra y sale con las animaciones de la tabla del apartado 7.

**En `home-panel.js`:**

- **Foco por tabla de filas** (`HOME_PANEL_ROWS`), con la mitad recordada por fila. Sustituye a `f >= 2 ? 1 : …`.
- **Retardo de 600 ms** para `pictureMode` y lectura del brillo después.
- **Pila de hojas** (`homePanelSheet: '' | 'picture' | 'schedule'`) para que Atrás vuelva atrás.
- **Lista de modos por `dynamicRange`**: ya existe (`HOME_PANEL_MODE_LISTS`).
- **Temporizador del horario**, fuera del panel abierto: `genericTimer` cada minuto, o lo que use quien implemente el modo noche.

## 12. Cambios priorizados

**Must**

1. **Fila 2 con Modo noche y Horario, justo debajo de Brillo.** Modo noche se activa con una pulsación, sin confirmar. El prefijo «Noche · N %» aparece en Brillo y el disco relleno en la cápsula.
2. **Estados de los interruptores distintos del foco** (tabla de la sección 6). El blanco solo significa foco.
3. **Selector «Modo de imagen» en línea**, con Izquierda/Derecha, retardo de 600 ms antes de aplicar, lista según el contenido y una nueva lectura del brillo al confirmarse. El modo sale de la tarjeta «Ahora».
4. **Foco por tabla de filas**: sin dar la vuelta, la mitad recordada, Atrás siempre cierra o vuelve, y los chevrones o extremos apagados como aviso.
5. **Horario visible en el panel**, con OK para activarlo o desactivarlo. Horas por defecto: 22:00–07:00.
6. **Textos en español e inglés** de la sección 10.

**Should**

7. **Hoja «Horario del modo noche»** con «Programar», «Empieza» y «Termina» (selectores de 30 en 30 min).
8. **Hoja «Modo de imagen»** con la lista completa. OK aplica al momento.
9. **Vidrio al 78 % y velada horizontal** detrás de la columna, para vídeo claro.
10. **Foco con halo de 3 u y escala 1,03**, además del blanco.
11. **Brillo con el modo noche activo**: ajusta y recuerda el nivel de noche. OK muestra «Primero desactiva Modo noche».
12. **«Sin conexión»** cuando la Sonos no contesta.

**Could**

13. Cápsula de foco al 85 % con el modo noche activo.
14. Aviso de 3 s al activarse por horario con una app en marcha.
15. Cierre automático de 20 s dentro de las hojas.
16. Retirar Auto del todo si sigue sin atenuar nada en la C1 (véase «Brillo, lo que hay» en `docs/llm-wiki/lg-webos.md`).

## Preguntas abiertas

- **Modo noche y modo de imagen.** El código en curso aplica el modo noche a cada modo que aparece en pantalla (`HOME_PANEL_NIGHT_KEYS`, por `dimension`). La propuesta lo da por bueno. Falta que el usuario confirme que lo prefiere a que el modo noche se desactive al cambiar de modo.
- **Nivel de noche por defecto.** El código usa 40 de luz OLED (`homePanelSchedule.level`). ¿Basta un solo nivel, o uno por tipo de contenido (SDR / HDR / Dolby Vision)?
- **Horario.** ¿Basta con mostrarlo y activarlo desde el panel (*must*), o el usuario quiere cambiar las horas desde la tele (hoja, *should*)?

## Variante Liquid Glass

Exploración del 2026-11-17. Maqueta: [`panel-glass-lg.html`](panel-glass-lg.html) (sobre HOME, sobre una película, modo noche y detalle ×2). La columna, las filas, el foco y los textos son los del panel actual. Solo cambia el material.

### Qué es real y qué se simula

- **Corrige el apartado 1.** El compositor usa Qt 5.12: hay `QtQuick 2.12`, `QtGraphicalEffects 1.x` (`FastBlur`, `OpacityMask`…) y `ShaderEffect` con sombreadores propios. GPU: Mali-G51 de 3 núcleos.
- **Sobre gráficos (HOME, inicio de LG, menús de apps) el desenfoque es real.** Se copia la escena detrás de la columna y se desenfoca.
- **Sobre vídeo no hay desenfoque.** La película va en un plano de hardware por debajo de los gráficos. Ahí la escena GL es transparente: la copia sale vacía. No se finge.
- **La lente del borde es una aproximación.** El sombreador desplaza la muestra hacia dentro cerca del filo. No es la refracción física de Apple.
- **El filo especular, el brillo de arriba y las sombras son pintados.** No siguen una luz real ni el movimiento.
- **Sin velada lateral.** El usuario la quitó. La legibilidad sale del tinte de cada cápsula.

### Material

| | Sobre gráficos (`frost = 1`) | Sobre vídeo (`frost = 0`) |
|---|---|---|
| Fondo | copia desenfocada, saturación ×1,7 | nada |
| Tinte | `#700c121c` (44 %) | `#b3141a26` → `#cc090c14` (70 → 80 %) |
| Lente | banda de borde que concentra el fondo | — |
| Filo | 1,3 u, fuerte arriba, tenue abajo | igual |
| Sombras | interior abajo, profundidad suave | interior abajo, profundidad más marcada |
| Modo noche | filo al 55 %, foco `#d9dfe4ea` | igual |

El foco no cambia: cápsula blanca, halo de 3 u (`PanelFocus`) y escala 1,03. En los deslizadores se vuelve blanco el nivel, no la cápsula. «Encendido» sigue siendo el disco relleno.

**Cuándo usar cada uno.** `home-panel.js` ya sabe qué app está delante. Con un reproductor, TV en directo o una entrada HDMI: `frost = 0`. Con HOME, el inicio de LG o un menú: `frost = 1`. Si hay duda, `frost = 0`, que siempre se lee bien.

### Receta QML

Un solo `ShaderEffect` por cápsula hace la máscara, el tinte, la lente, el filo y las sombras. La textura desenfocada se comparte. Va en un `PanelGlass.qml` nuevo que sustituye al `Rectangle` de vidrio de `PanelSlider`, `PanelButton`, `PanelChoice` y `PanelFormats`.

En `HomePanel.qml`, una vez:

```qml
import QtQuick 2.12
import QtGraphicalEffects 1.0

// Snapshot of what lies behind the column, at half resolution, only when asked.
ShaderEffectSource {
    id: backdrop
    sourceItem: viewsRoot        // the compositor's views root; the panel must NOT be inside it
    sourceRect: Qt.rect(column.x - 24 * u, column.y - 24 * u, column.width + 48 * u, column.height + 48 * u)
    textureSize: Qt.size(sourceRect.width / 2, sourceRect.height / 2)
    live: false
    recursive: false
    visible: false
}
FastBlur {
    id: frosted
    width: backdrop.sourceRect.width / 2
    height: backdrop.sourceRect.height / 2
    source: backdrop
    radius: 32                   // at half size ≈ 24 u at full size
    cached: true
    visible: false
}
Timer {                          // refresh once a second while open over graphics
    interval: 1000; repeat: true
    running: root.shown && root.frost
    onTriggered: backdrop.scheduleUpdate()
}
// In open(): if (frost) backdrop.scheduleUpdate() before the opening animation starts.
```

`PanelGlass.qml`:

```qml
import QtQuick 2.12

ShaderEffect {
    id: glass
    property real u: 1
    property real radius: height / 2     // 34 u for the "Now" card
    property real margin: 16 * u         // room for the depth shadow
    property var source                  // frosted (shared)
    property rect area                   // this capsule in source coordinates, normalized
    property real frost: 1               // 0 over video
    property color tint: frost > 0 ? "#700c121c" : "#cc090c14"
    property color top: frost > 0 ? "#00000000" : "#b3141a26"
    property real rim: 1                 // 0.55 with Night mode
    property real shadow: frost > 0 ? 0.18 : 0.34

    // The quad is larger than the capsule by `margin` on each side.
    x: -margin; y: -margin
    width: parent.width + 2 * margin; height: parent.height + 2 * margin
    property size size: Qt.size(parent.width, parent.height)

    fragmentShader: "
        varying highp vec2 qt_TexCoord0;
        uniform sampler2D source;
        uniform highp vec4 area;
        uniform highp vec2 size;
        uniform highp float radius, margin, frost, rim, shadow;
        uniform lowp vec4 tint, top;               // assumed premultiplied: check on Qt 5.12
        uniform lowp float qt_Opacity;
        highp float sdf(highp vec2 p) {          // rounded rect, < 0 inside
            highp vec2 q = abs(p - size * 0.5) - (size * 0.5 - vec2(radius));
            return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
        }
        void main() {
            highp vec2 p = qt_TexCoord0 * (size + 2.0 * margin) - vec2(margin);
            highp float d = sdf(p);
            lowp float a = clamp(0.5 - d, 0.0, 1.0);
            highp vec2 c = clamp(p, vec2(radius), size - vec2(radius));
            highp vec2 v = p - c;
            highp vec2 n = v / max(length(v), 0.001);          // outward normal
            highp float edge = clamp(1.0 + d / (0.22 * size.y), 0.0, 1.0);
            highp vec2 t = p / size;
            // Lens: near the rim, sample further in, so the background bunches at the edge.
            highp vec2 uv = area.xy + area.zw * (t - n * edge * edge * 0.06);
            lowp vec4 bg = texture2D(source, uv) * frost;
            bg.rgb *= 1.0 + 0.25 * edge * frost;               // brighter in the lens band
            lowp vec4 tn = mix(top, tint, clamp(t.y, 0.0, 1.0)); // vertical tint (over video)
            tn = top.a > 0.0 ? tn : tint;
            lowp vec4 col = bg * (1.0 - tn.a) + tn;
            col.rgb += 0.10 * (1.0 - smoothstep(0.0, 0.46, t.y)); // top sheen
            col.rgb *= 1.0 - 0.22 * edge * clamp(n.y, 0.0, 1.0);  // inner shadow, bottom
            lowp float spec = (1.0 - smoothstep(0.0, 1.6, abs(d + 0.8)))
                            * (0.25 + 0.75 * clamp(-n.y, 0.0, 1.0));
            col.rgb += vec3(0.85 * spec * rim);                   // specular rim, top-lit
            highp float ds = sdf(p - vec2(0.0, 0.35 * margin));
            lowp float sh = shadow * (1.0 - smoothstep(0.0, margin, ds)) * (1.0 - a);
            gl_FragColor = (col * a + vec4(0.0, 0.0, 0.0, sh)) * qt_Opacity;
        }"
}
```

- `area` sale de `mapToItem(column, 0, 0)` menos el origen de `backdrop.sourceRect`, dividido por su tamaño. Se recalcula con la escala del foco.
- **Foco:** en botones y selector, `tint: "#f0f6f8fc"` (de noche `#d9dfe4ea`). El deslizador no cambia el vidrio: su nivel sigue siendo el `Rectangle` de siempre, encima.
- **Plan B sin sombreador:** `OpacityMask` de `frosted` con un `Rectangle` redondeado como máscara, un `Rectangle` de tinte y otro de 1 px con `Gradient` para el filo. Pierde la lente.

### Coste en GPU

- **La copia se hace al abrir y luego una vez por segundo**, no en cada fotograma (`live: false` + `scheduleUpdate()`). Lo que hay detrás casi no se mueve mientras el panel tiene el foco.
- **Media resolución.** La zona de la columna mide ≈ 760 × 920 px; la copia, ≈ 380 × 460. Un `FastBlur` a ese tamaño cuesta poco y solo corre al refrescar (`cached: true`).
- **Una textura para todas las cápsulas.** Cada cápsula la lee con su `area`. No hay un desenfoque por cápsula.
- **Un `ShaderEffect` pequeño por cápsula** (7 en total), unos 0,6 MP por fotograma. Asequible para la Mali-G51.
- **Sobre vídeo no se copia ni se desenfoca nada.** El `Timer` no corre y `frost = 0`. Solo quedan los 7 sombreadores.
- **Cerrado no cuesta nada.** El panel no se dibuja y el `Timer` se para.
- Hay que medir en la tele que abrir siga en 240 ms sin saltos. Si no, refrescar solo al abrir.

### Qué cambia frente al panel actual

- `Rectangle` de vidrio (`#c70e141e` + borde `#50ffffff`) → `PanelGlass`. Desaparece el borde de 1 px: lo sustituye el filo.
- Sobre gráficos, el tinte baja del 78 % al 44 %, porque el desenfoque ya separa el texto del fondo.
- Sobre vídeo, el tinte medio sigue en ≈ 78 %, ahora en degradado, con filo, brillo y sombra de profundidad.
- `import QtQuick 2.4` → `2.12` en los archivos del panel, más `QtGraphicalEffects 1.0` en `HomePanel.qml`.
- `home-panel.js` pasa `frost` según la app de delante, y `rim` según el modo noche.
- Sin cambios: la columna, las filas, el foco, el halo, las animaciones, los textos y el «encendido» como disco relleno.

### Por comprobar en la tele

- Que el panel se pueda poner **fuera** de `viewsRoot`. Si está dentro, la copia lo incluiría a sí mismo. La alternativa es copiar solo en `open()`, antes de que suba la opacidad, sin refrescar.
- Que la copia de una app web (HOME es una app web) salga con contenido y no negra.
- El coste real del `FastBlur` y de los 7 sombreadores con el perfil de GPU de LG.
