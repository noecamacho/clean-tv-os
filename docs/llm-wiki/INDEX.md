# Wiki LLM · HOME

Esta wiki es el contexto de trabajo para otros agentes. Se actualizó el 16 de noviembre de 2026 (HOME 0.9.7) a partir del código local, las specs y las pruebas en la TCL; no sustituye la lectura del código cuando se cambia una función.

## Ruta de lectura

1. [Contexto y requisitos](context.md): qué pidió el usuario, qué existe y qué falta.
2. [Arquitectura](architecture.md): módulos Rust, puente Android, recursos y navegación.
3. [Diseño y rendimiento](design.md): tvOS 26, vidrio, fondos, foco y límites reales.
4. [Dispositivo y operaciones](operations.md): compilar, instalar, probar, recuperar.
5. [Referencias y procedencia](references.md): Lume, TV Home, artículo de AppleInsider, licencias.
6. [Pendientes y guía de colaboración](next-work.md): riesgos, decisiones abiertas y pruebas futuras.
7. [Guía de diseño de tvOS (HIG)](tvos-hig.md): lo que Apple publica, diferencias con HOME y lista priorizada de cambios. Estudiada el 2026-11-11; los vídeos solo por transcripción.
8. [APIs de TCL y Realtek en la 55Q5K](tcl-apis.md): librerías del fabricante que una app puede cargar, sus clases y métodos, y los límites. Úsala antes de dar por imposible un control de la tele.
9. [HOME en LG webOS](lg-webos.md): segunda tele (OLED55C1PSA, webOS 6.5.3). Límites de una app web, ruta nativa con root, CLI, simulador y lo que falta comprobar. Tele rooteada; port nativo `app.home.tv` en curso (`crates/lume-webos`) y diseño híbrido HOME × Luma en Figma.

## Fuentes de verdad

- `AGENTS.md`: contrato de trabajo actual.
- `openspec/specs/`: requisitos vigentes por capacidad; algunas secciones antiguas aún describen el upstream y deben armonizarse al tocarlas.
- `docs/tvos26-tcl-55q5k.md`: evidencia del APK y la TV.
- `docs/upstream/`: documentación preservada del origen, **histórica** para este fork.
- `llms-full.txt`: concatenación generada de esta wiki para asistentes que prefieren un solo archivo; actualízala con `uv run python scripts/build-llm-wiki.py`.

## Identidad y rutas

- Proyecto local: `~/Projects/clean-tv-os`.
- Nombre visible: **HOME**.
- Identificadores internos de compatibilidad: paquete `app.lume.tv`, actividad `LumeActivity`, crates `lume-*`, tema `themes/lume/theme.css`.
- Repositorio: `noecamacho/clean-tv-os`, rama `main`, historial propio; cambios en `CHANGELOG.md`.
- Origen del código: `douglaspands/lume-tv-launcher` en su commit `2e7421d` (GPL-3.0-or-later); no es un remoto.
