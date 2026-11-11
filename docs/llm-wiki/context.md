# Contexto del producto

## Objetivo del usuario

El usuario quiere un launcher para su TCL 55Q5K que se vea y se sienta lo más cercano posible a tvOS 26, con Liquid Glass aproximado, fondos de playa de gran calidad, nombre e icono visibles **HOME**, sin logos Lume, y navegación de aplicaciones muy fluida. La interfaz debe seguir el idioma de la TV, especialmente español e inglés. Pidió además un Centro de control con las funciones de tvOS que sean aplicables a este televisor. Autorizó proceder sin pedir confirmación en cada paso.

La idea inicial fue probar `tinulica/tv-home`, después usar el proyecto Rust `douglaspands/lume-tv-launcher` como base liviana y reproducir el aspecto visual. El código actual deriva de Lume. TV Home sigue como referencia separada; su código no fue fusionado.

## Implementado y comprobado

- Nombre y recursos visuales de HOME, sin marca Lume en la interfaz principal; identificadores internos conservados para las actualizaciones.
- Tres escenas de playa generadas para esta variante, empaquetadas como QOI con miniaturas. Beach Cove/Cala turquesa es la predeterminada.
- Home al estilo tvOS: cinco banners por fila, la primera sobre una plataforma de vidrio, nombre solo bajo el banner enfocado; barra superior de vidrio con Apps, entradas, ajustes y reloj.
- Centro de control de mosaicos de vidrio arriba a la derecha (se abre también con el botón Home estando en Inicio), navegación con D-pad, tarjetas Wi-Fi, Bluetooth, Entradas, Imagen, Sonido, Modo ambiente, Apps y Ajustes. Entradas se oculta si el selector no está disponible.
- Traducciones en `values` (inglés), `values-es`, `values-pt-rBR`, `values-id` y `values-ru`; la TV probada usa `es-ES`.
- APK firmado `0.4.0-tvos` instalado con `adb install -r`. Google TV continúa siendo el Home del sistema; el servicio de accesibilidad `HomeKeyService` hace que el botón Home abra HOME (véase operations.md).
- Pruebas Rust: 506 aprobadas, 2 ignoradas; verificación del APK: 31.08 MB, ARMv7 y ARM64, 9 recursos QOI de fondos.
- Capturas reales copiadas a `docs/llm-wiki/images/`: Home, barra superior y Centro de control en la TCL. La del panel muestra el rótulo español completo de Modo ambiente.

## Alcance real

Esto es una app launcher, no una modificación del firmware. No tiene acceso al reproductor de otras apps, a un compositor global de vídeo, a Apple Home, perfiles de Apple TV ni VPN del sistema. El vidrio refracta la textura del fondo de la propia app mediante una muestra de shader; no puede copiar el motor privado de Apple. El launcher del sistema no se toca; el botón Home abre HOME solo mientras el servicio de accesibilidad esté activado.

La tarjeta Sonido y la de Imagen llegan a la misma actividad `DisplaySoundActivity` en este firmware TCL. El acceso heredado llamado Sleep en el código abre **Modo ambiente** en la TCL y se etiquetó así en el Centro de control. El selector de entradas mostró Google TV Home, AirPlay y TV; no había un HDMI activo para probar cambio de fuente.
