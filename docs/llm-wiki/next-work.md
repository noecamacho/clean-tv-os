# Pendientes y colaboración

## Estado antes de iniciar cambios

El trabajo vive en `main` de `noecamacho/clean-tv-os`. El APK de `dist/` no se versiona. No hagas commit o push sin que el usuario lo solicite.

## Pendientes verificables

1. Hacer una revisión visual en la TV con el usuario: proporciones de la barra y panel, iconografía y sensación del foco. Hay una captura final documentada, pero la semejanza exacta con tvOS 26 sigue siendo subjetiva.
2. Medir navegación prolongada, arranque y memoria en la versión 0.3.9 si se cambia el render. Las medidas actuales son muestras puntuales.
3. Probar una entrada HDMI cuando haya una fuente conectada. Solo se verificó que se abre el panel TCL.
4. La tarjeta Sonido llega al mismo menú de Imagen en este firmware. Podría ofrecer un acceso más específico si se encuentra una actividad estable que exista realmente en el DEX de esta TCL.
5. Revisar las specs heredadas y la pantalla Acerca de antes de cualquier distribución. Hay textos/README upstream archivados y nombres internos Lume que no deben cambiarse sin un plan de migración.
6. Perfiles, VPN, automatización de hogar, energía y temporizador: primero comprobar permisos/APIs reales en Android 12 TCL. No mostrar un botón que lleve a una acción diferente de la etiqueta.

7. Diseño tvOS pendiente: quitar la barra superior con pestañas (tvOS no la tiene), modo edición con temblor para reordenar con el D-pad, vidrio con fondo pre-difuminado, y animación de cierre del Centro de control.
8. Controles directos: Wi-Fi/Bluetooth solo abren ajustes (Android 12 con `targetSdk` 36 no deja alternarlos); `tcl_sys_audio_sound_mode` existe en `Settings.System` pero no se ha probado que escribirlo cambie el audio; no se encontró clave de modo de imagen ni de brillo.
9. `HomeKeyService`: probar encendido desde reposo y arranque en frío, y si el firmware desactiva el servicio.

## Reglas de colaboración

Trabaja en cambios pequeños y explica qué se cambió, por qué, qué pruebas se hicieron y qué no pudo probarse. Actualiza `openspec/specs/` y la página de wiki pertinente si cambia una decisión duradera. Regenera el contexto compilado con `uv run python scripts/build-llm-wiki.py` y comprueba `git diff --check` al terminar. La UI y respuesta al usuario deben ser españolas o seguir el idioma del sistema; conserva inglés como fallback. Evita publicar APKs o repositorios sin instrucción del usuario.
