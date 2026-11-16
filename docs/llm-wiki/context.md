# Contexto del producto

## Objetivo del usuario

El usuario quiere un launcher para su TCL 55Q5K que se vea y se sienta lo más cercano posible a tvOS 26, con Liquid Glass aproximado, fondos de playa de gran calidad, nombre e icono visibles **HOME**, sin logos Lume, y navegación de aplicaciones muy fluida. La interfaz debe seguir el idioma de la TV, especialmente español e inglés. Pidió además un Centro de control con las funciones de tvOS que sean aplicables a este televisor. Autorizó proceder sin pedir confirmación en cada paso.

La idea inicial fue probar `tinulica/tv-home`, después usar el proyecto Rust `douglaspands/lume-tv-launcher` como base liviana y reproducir el aspecto visual. El código actual deriva de Lume. TV Home sigue como referencia separada; su código no fue fusionado.

## Implementado y comprobado

- Nombre y recursos visuales de HOME, sin marca Lume en la interfaz principal; identificadores internos conservados para las actualizaciones.
- Tres escenas de playa generadas para esta variante, empaquetadas como QOI con miniaturas. Beach Cove/Cala turquesa es la predeterminada.
- Inicio en dos estados, como tvOS: en reposo el fondo nítido y solo la fila principal en su placa de vidrio contra el borde inferior; Abajo despliega la biblioteca entera sobre el fondo desenfocado. Cinco o seis banners por fila, nombre solo bajo el banner con foco en la biblioteca, cápsula de hora arriba a la derecha. OK largo abre el menú del icono.
- Centro de control sobre el Inicio atenuado (botón Home en Inicio, o la cápsula): Suspender, Wi‑Fi, Bluetooth, deslizadores de Brillo (retroiluminación real por la librería de TCL) y Volumen que siguen en vivo a la tele, AirPlay, Temporizador y botones redondos para Entradas, Duplicar pantalla, Imagen, Sonido, Ajustes de HOME y Ajustes de Android.
- Ajustes con la estructura de tvOS (emblema a la izquierda, lista a la derecha), con páginas de Pantalla de inicio y Fondo de pantalla (vista previa a pantalla completa); editor, mapeo de control, información y Acerca de con el mismo estilo.
- Modo ambiente propio (fondo y hora) y vuelta al punto de partida cada vez que HOME deja la pantalla; volver desde otra app tarda unos 50 ms porque el renderizador se conserva.
- Traducciones en `values` (inglés), `values-es`, `values-pt-rBR`, `values-id` y `values-ru`; la TV probada usa `es-ES`.
- APK firmado `0.9.7-tvos` instalado con `adb install -r`. Google TV continúa siendo el Home del sistema; el servicio de accesibilidad `HomeKeyService` hace que el botón Home abra HOME (véase operations.md).
- Pruebas Rust: 494 aprobadas, 2 ignoradas; verificación del APK: ARMv7 y ARM64, 9 fondos QOI.
- Capturas reales copiadas a `docs/llm-wiki/images/`: Home, barra superior y Centro de control en la TCL. La del panel muestra el rótulo español completo de Modo ambiente.

## Alcance real

Esto es una app launcher, no una modificación del firmware. No tiene acceso al reproductor de otras apps, a un compositor global de vídeo, a Apple Home, perfiles de Apple TV ni VPN del sistema. El vidrio refracta la textura del fondo de la propia app mediante una muestra de shader; no puede copiar el motor privado de Apple. El launcher del sistema no se toca; el botón Home abre HOME solo mientras el servicio de accesibilidad esté activado.

Los botones Imagen y Sonido llegan a la misma actividad `DisplaySoundActivity` en este firmware TCL. Qué puede y qué no puede cambiar una app en esta tele está en `tcl-apis.md`. El selector de entradas mostró Google TV Home, AirPlay y TV; no había un HDMI activo para probar cambio de fuente.
