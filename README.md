# HOME · clean-tv-os

Launcher personal para Android TV, escrito principalmente en Rust y OpenGL ES 2.0. Esta variante para la TCL 55Q5K usa fondos de playa originales, una interfaz inspirada en tvOS 26 y un Centro de control con accesos a funciones compatibles de la TV.

**Estado:** versión 0.4.0, probada en la TCL 55Q5K a 1920×1080 y 60 cuadros por segundo. Google TV sigue siendo el launcher del sistema; un servicio de accesibilidad opcional hace que el botón Home abra HOME. Cambios en [CHANGELOG.md](CHANGELOG.md).

## Para continuar el proyecto

- [Guía para agentes](AGENTS.md)
- [Wiki LLM y contexto compilado](docs/llm-wiki/INDEX.md)
- [Contexto completo para LLM](llms-full.txt) y [mapa breve](llms.txt)
- [Compilación, pruebas y rollback de la TCL](docs/tvos26-tcl-55q5k.md)
- [Especificaciones actuales](openspec/specs/)

La interfaz pública se llama **HOME**. Algunos identificadores internos siguen llamándose `lume` para que la versión instalada pueda actualizarse sin perder datos. El código de base procede de [douglaspands/lume-tv-launcher](https://github.com/douglaspands/lume-tv-launcher) y conserva su licencia GPL-3.0-or-later ([LICENSE](LICENSE)). Los README originales se guardan en `docs/upstream/` como referencia histórica.

TV Home de `tinulica` es una referencia de diseño separada; su código no se ha integrado. Véase [referencias](docs/llm-wiki/references.md).
