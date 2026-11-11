# Diseño, movimiento y rendimiento

## Dirección visual

Referencias: la [presentación práctica de tvOS 26 en AppleInsider](https://appleinsider.com/articles/25/06/10/tvos-26-hands-on-sleek-liquid-glass-redesign-new-control-center-and-more) y el repositorio TV Home, registrado en `references.md`. La composición usa la playa a pantalla completa, una barra translúcida en la parte superior, estante de apps bajo, contorno claro en el elemento enfocado y un panel de control en el extremo derecho. Son interpretaciones propias de imágenes públicas, no assets ni código de Apple.

La implementación usa el fondo QOI ya cargado como fuente de una muestra con refracción en los bordes, gradientes semitransparentes, borde blanco fino y tintes oscuros para legibilidad. Los tres fondos de playa son originales de esta variante; sus estrellas/metadatos acompañan los QOI. La fuente de iconos Lucide se reduce con `scripts/subset-fonts.py`.

Capturas de referencia tomadas en la TCL: [Home](images/home-tcl.png), [barra superior](images/topbar-tcl.png) y [Centro de control final](images/control-center-tcl.png). Para una evaluación visual, abre las imágenes originales a resolución completa; el Centro de control final incluye la etiqueta española completa «Modo ambiente».

## Movimiento y rendimiento

- Canvas lógico 1280×720 renderizado a **1920×1080** (1,5 px por unidad). El tope de 720p de la versión 0.3.9 ya no existe.
- **Redibujado parcial**: cada cuadro compara su lista de dibujo con la anterior (`Frame::damage`) y solo repinta el rectángulo que cambió, con `glScissor`, `EGL_EXT_buffer_age` y `EGL_KHR_partial_update` (`draw` en `crates/lume-app/src/android/mod.rs`). Los cambios de textura (iconos, fondo, atlas rehecho) fuerzan un cuadro completo vía `Gles2Backend::take_content_changed`. Es lo que Android hace por TV Home y la razón de que aquel fuera fluido a 1080p.
- La GPU real de la TCL 55Q5K es **PowerVR BXE-4-32** (OpenGL ES 3.2, Vulkan 1.1). La Mali-470 de `docs/device-notes.md` es la tele del autor de Lume.
- Foco de banner: 260 ms `cubic-bezier(0.2, 0, 0, 1)`, escala 1,12 con un pequeño rebote, sombra amplia, borde claro de 2 unidades y nombre solo bajo el banner enfocado.
- Centro de control: mosaicos de vidrio flotantes arriba a la derecha, sin panel; bajan desde la esquina en 240 ms; el mosaico enfocado se vuelve blanco.
- Medida en TCL (0.3.15, 1080p, navegación por la primera fila con mando virtual `uinput`): mediana 16,7 ms, p95 17,0 ms, 2 intervalos de 33 ms en ~120. Un cuadro completo a 1080p cuesta ~37 ms: lo pagan el desplazamiento vertical y la apertura del Centro de control, que oscurecen o mueven toda la pantalla. Son pruebas puntuales.
- Para medir, deja al menos 600 ms entre teclas: las pausas menores de 100 ms entre animaciones se confunden con cuadros lentos. `adb shell input keyevent` arranca una VM por tecla y ensucia la medida; usa `adb shell uinput`.

## Faltas conocidas respecto a tvOS

No se reproduce el compositor privado Liquid Glass, la refracción de vídeo de otros procesos, el Centro de control del sistema ni la experiencia de reproducción de Apple TV. No hay perfiles de usuario, Apple Home, VPN, panel de energía ni temporizador real de apagado. Cada función futura debe comprobarse en el firmware TCL antes de mostrar un control que sugiera una acción que no puede ejecutar.

Las specs de `openspec/specs/design-system/spec.md` heredaron reglas del tema original y tienen un apartado de variante tvOS 26. Si una sección antigua contradice la variante o la UI comprobada, actualiza la spec al modificar ese componente.
