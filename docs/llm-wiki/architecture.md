# Arquitectura y mapa del código

## Flujo

`LumeActivity` (ciclo de vida y eventos Android) ↔ `LumeBridge` (consultas/Intents Android) ↔ JNI de `lume-app` ↔ `Shell`/`LumeApp` de `lume-core` ↔ lista de dibujo GLES2. Toda la UI propia, foco, transición, composición y estado reside en Rust. Java se usa para las APIs Android que el NDK no expone de forma adecuada.

## Módulos principales

| Ruta | Responsabilidad |
| --- | --- |
| `crates/lume-core/src/app/` | Estado de pantallas, D-pad, focus, Home, Apps, Centro de control, ajustes y menús. |
| `app/home.rs`, `app/apps.rs`, `app/topbar.rs` | Diseño del estante, cuadrícula y barra superior. |
| `app/control_center.rs` | Panel de vidrio, tarjetas y efectos de accesos directos. |
| `app/painter.rs`, `render/gles2.rs`, `render/shaders.rs` | Comandos de dibujo, textura del fondo y GLES2. |
| `ui/animation.rs`, `display.rs`, `shell/` | Animaciones, escala de 1280×720 y bucle de render. |
| `home/config.rs`, `storage.rs` | Favoritos, wallpaper y persistencia local. |
| `android/app/src/main/java/app/lume/tv/` | Actividad y puente de Android; resolución de ajustes según modelo TCL/AIWA. |
| `android/app/src/main/res/values*/strings.xml` | Traducciones Android compiladas hacia el traductor Rust. |
| `themes/lume/theme.css` | Tokens visuales compilados por el proyecto. |
| `crates/lume-core/assets/wallpaper/` | Fondos QOI y miniaturas incluidas en el APK. |
| `tools/wallpaper/` | Fuentes y scripts de preparación de wallpapers. |
| `scripts/container-build.sh` | Buildx/Docker, Gradle, firma y artefactos en `dist/`. |

## Invariantes

- Mantén el `applicationId` `app.lume.tv` y la misma clave de firma si el objetivo es actualizar la app ya instalada y conservar preferencias. Renombrar paquete o firma requiere migración explícita.
- Usa los efectos existentes (`OpenTvSetting`, `OpenInputs`, `OpenAndroidSettings`) para abrir paneles del sistema. No implementes conjeturas de toggles privilegiados.
- El foco del Centro de control se calcula por fila y columna; Back vuelve a la pestaña anterior y deja enfocado el botón de control.
- La app evita animación continua en reposo. El bucle duerme cuando no hay cambio visual; no agregues blur o pasadas a pantalla completa sin medir en la TCL.
- Las specs de `openspec/specs/` deben registrar una capacidad nueva; el directorio `docs/upstream/` no es especificación de HOME.
