# Changelog

Los cambios de HOME, del más reciente al más antiguo. El formato sigue
[Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

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
