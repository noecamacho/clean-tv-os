# clean-tv-os · guía para agentes

## Proyecto y alcance

Este es **HOME**, un launcher personal para la TCL 55Q5K. La interfaz pública se llama HOME y busca una apariencia cercana a tvOS 26: playas originales, superficies translúcidas, foco fluido y Centro de control. La implementación parte del código Rust de `douglaspands/lume-tv-launcher`, bajo GPL-3.0-or-later. `app.lume.tv`, nombres de crates y clases Java siguen siendo identificadores internos para conservar las actualizaciones instaladas. No los cambies como simple limpieza visual.

La base de verdad para este fork es `docs/llm-wiki/INDEX.md`, seguida de `openspec/specs/` y `docs/tvos26-tcl-55q5k.md`. `docs/upstream/` conserva documentación del proyecto original y puede contradecir este fork; no la uses como instrucciones actuales. `CLAUDE.md` apunta a este archivo.

## Estado actual

- Repositorio propio en GitHub (`noecamacho/clean-tv-os`, rama `main`), con historial nuevo que parte del estado de Lume en su commit `2e7421d`. Los cambios se anotan en `CHANGELOG.md`. No hagas commit ni push salvo petición explícita del usuario.
- APK firmado: `dist/lume-0.4.0-tvos-release.apk`, instalado como actualización en la TCL. Google TV sigue siendo el Home del sistema, pero el servicio de accesibilidad `HomeKeyService` (activado a petición del usuario) hace que el botón Home y el encendido abran HOME; véase «Botón Home» en `docs/llm-wiki/operations.md`. No deshabilites componentes Google/TCL ni cambies el launcher del sistema sin una petición expresa.
- La firma y el APK de respaldo están fuera de Git, en `~/Projects/.clean-tv-os-private/`. No leas ni imprimas contraseñas y nunca subas esos archivos.
- El origen (`douglaspands/lume-tv-launcher`) ya no es un remoto; nunca publiques este fork allí.

## Flujo de trabajo

1. Lee `docs/llm-wiki/INDEX.md` y la página pertinente. Comprueba `git status` antes de editar.
2. Mantén los cambios de UI en Rust; usa la capa Java solo cuando necesites una API Android. Conserva la compatibilidad de `app.lume.tv` y la firma para instalar con `adb install -r`.
3. Anota el cambio en `CHANGELOG.md`. Actualiza la spec afectada en `openspec/specs/` y la wiki cuando cambies comportamiento o una decisión duradera. Los textos de interfaz viven en `android/app/src/main/res/values*/strings.xml`; comprueba español e inglés.
4. Verifica con pruebas relevantes, APK y TV según el alcance. No repitas pruebas costosas sin riesgo concreto. La tele puede no estar en red; documenta lo que no hayas podido comprobar.
5. Responde al usuario en español. Código y comentarios técnicos pueden seguir en inglés. Usa `zsh` para comandos y `uv run` para Python.

## Comandos rápidos

Desde la raíz del repositorio:

```sh
docker run --rm -v "$PWD:/src" -w /src lume-toolchain:latest sh -lc 'cargo fmt --all && cargo test -q -p lume-core'
set -a; source ~/Projects/.clean-tv-os-private/signing/credentials.env; set +a
LUME_VERSION_NAME=v0.4.0-tvos scripts/container-build.sh release
uv run python scripts/verify-apk.py dist/lume-0.4.0-tvos-release.apk
adb connect 192.168.0.229:5555
adb -s 192.168.0.229:5555 install -r dist/lume-0.4.0-tvos-release.apk
adb -s 192.168.0.229:5555 shell am start -n app.lume.tv/.LumeActivity
```

Para una versión nueva, incrementa el parche (`v0.4.1-...` o posterior); Android rechaza un `versionCode` menor. Consulta `docs/llm-wiki/operations.md` antes de instalar o cambiar ajustes.

## Límites y referencias

- `tinulica/tv-home` es **referencia visual/funcional**, no código incorporado. Véase `docs/llm-wiki/references.md`. No copies su código sin revisar licencia y atribución.
- La capa de vidrio aproxima la estética con la textura del wallpaper; no es el motor privado Liquid Glass de Apple ni refracta vídeo de otros procesos.
- Las funciones de perfiles, Apple Home, VPN y apagado del sistema no están implementadas. Diseña cada una solo si existe una API/permiso Android viable para esta TV.
- Para rollback, pruebas de Home, ajustes y entradas, consulta `docs/llm-wiki/operations.md`.
