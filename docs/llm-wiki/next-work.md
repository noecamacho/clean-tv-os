# Pendientes y colaboración

## Estado antes de iniciar cambios

El trabajo vive en `main` de `noecamacho/clean-tv-os`. El APK de `dist/` no se versiona. No hagas commit o push sin que el usuario lo solicite.

## Pendientes verificables (a 2026-11-16, HOME 0.9.7)

1. **Comprobar con el mando físico y a simple vista**, que nunca se han probado: el botón Home real, el encendido desde reposo y el arranque en frío (¿sigue activo `HomeKeyService`? ¿aparece HOME?), que el deslizador de Brillo se vea cambiar en el panel (por registro y lectura sí cambia), que el temporizador suspenda al vencer, y que el modo ambiente arranque solo tras el tiempo elegido (el salvapantallas de Google TV puede adelantarse).
2. **Aviso previo del temporizador.** HOME no puede dibujar sobre otra app; haría falta una notificación o una ventana superpuesta con permiso.
3. **Fecha en el modo ambiente.** Exige nombres de días y meses en los cinco idiomas.
4. **Modo de imagen desde HOME.** `TvPqManager.setPictureMode` es accesible, pero no se conoce qué entero es cada modo (el actual se lee como 10). Averiguarlo cambiando el modo en el menú de la tele y leyendo `getPictureMode()`; véase `tcl-apis.md`.
5. **Otras lecturas de `tcl-apis.md` sin probar en ejecución:** luz azul baja, MEMC, modo juego, lista de entradas (`TTvInputMgr`), datos del dispositivo (`tromlib`). La lista completa está en su sección «Sin comprobar».
6. **Código heredado sin entrada desde la interfaz:** `app/apps.rs` (pestaña Apps), `app/topbar.rs`, gran parte de `app/dock.rs`, `return_point` y `restore` en `app/mod.rs` (HOME ya no restaura el lugar al volver) y la lectura de estadísticas (`stats`, `wants_stats` devuelve `false`). Varias pruebas llegan a Ajustes a través de la pestaña Apps (`open_apps`); al borrar hay que reescribirlas.
7. **Specs heredadas en portugués** (`launcher-shell`, `device-stats`, `apps-screen`, `home-editor`…) describen el upstream. La spec viva de este fork es `home-screen`; armonizar o archivar las demás.
8. **Reordenación libre y carpetas, búsqueda, Top Shelf** (puntos 13 a 15 de `tvos-hig.md`). Top Shelf no es viable: las tablas de «seguir viendo» exigen `ACCESS_ALL_EPG_DATA`.
9. **Compilación anticipada (AOT)** de Max y Apple TV: `cmd package compile -m speed -f` no cambió su estado (`verify`); Prime Video sí quedó en `speed`.
10. **Probar una entrada HDMI real** y medir sesiones largas (memoria retenida con el renderizador conservado: ~73 MB PSS).
11. **Otros idiomas.** Solo se ha visto la interfaz en español; inglés, portugués, indonesio y ruso se actualizaron sin verlos en pantalla.

## Reglas de colaboración

Trabaja en cambios pequeños y explica qué se cambió, por qué, qué pruebas se hicieron y qué no pudo probarse. Actualiza `openspec/specs/` y la página de wiki pertinente si cambia una decisión duradera. Regenera el contexto compilado con `uv run python scripts/build-llm-wiki.py` y comprueba `git diff --check` al terminar. La UI y respuesta al usuario deben ser españolas o seguir el idioma del sistema; conserva inglés como fallback. Evita publicar APKs o repositorios sin instrucción del usuario.
