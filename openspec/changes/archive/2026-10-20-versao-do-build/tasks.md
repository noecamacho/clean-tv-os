# Tasks

## 1. Regra da versão em Rust

- [x] 1.1 Criar o crate `crates/lume-version` no workspace, com `build.rs` (variável `LUME_VERSION_NAME` normalizada; senão hash curto do git com `-dirty` para arquivos rastreados; senão `dev`) e os `rerun-if-*` do design; verificar com `cargo build -p lume-version -q`
- [x] 1.2 Expor `VERSION` e `display()` (com "v" só para `X.Y.Z[-pré]`) e testes unitários de normalização (`v0.3.0`, `vv0.3.1`, `0.3.0`) e exibição (`0.3.0` → `v0.3.0`, `4d8438b-dirty` e `dev` sem "v"); verificar com `cargo test -p lume-version -q`
- [x] 1.3 Trocar `CARGO_PKG_VERSION` por `lume_version` em `crates/lume-core/src/app/settings.rs` (Sobre e canto das Configurações, via `display()`) e no log de `crates/lume-app/src/android/mod.rs`; verificar que `grep -rn CARGO_PKG_VERSION crates tools` não retorna nada e que `cargo run -p lume-desktop -- --screenshot about` mostra o hash do commit atual
- [x] 1.4 Fixar `version = "1.0.0"` no `Cargo.toml` do workspace com comentário de que não é a versão do build; atualizar o `Cargo.lock`; verificar `cargo test --workspace -q && cargo clippy --workspace --all-targets -q` limpos e `cargo fmt --all --check`

## 2. Gradle

- [x] 2.1 Em `android/app/build.gradle`, remover o `"0.2.1"` fixo e aplicar a mesma regra (normalizar `LUME_VERSION_NAME`; senão git via `providers.exec`; senão `dev`); `versionCode` pelo cálculo X.Y.Z quando for release, senão `LUME_VERSION_CODE`, senão `git rev-list --count HEAD`, senão 1
- [x] 2.2 Passar `LUME_VERSION_NAME` normalizada ao `cargo ndk` no `environment` da `CargoNdkTask` (e declará-la como `@Input`, para refazer a tarefa quando a versão mudar); verificar com `LUME_VERSION_NAME=vv9.9.9` num build que `aapt dump badging` (ou `verify-apk.py`) mostra `versionName='9.9.9'` e `versionCode='90909'`

## 3. Scripts e CI

- [x] 3.1 Em `scripts/container-build.sh`, calcular no host a versão (hash, `-dirty`, `dev`) e a contagem de commits quando `LUME_VERSION_NAME` não vier, normalizar quando vier, repassar `LUME_VERSION_NAME` e `LUME_VERSION_CODE` ao container (`--build-arg` e `ENV` no `docker/Dockerfile`) e nomear os arquivos `lume-<versão>-<variante>[-debugsigned]`; verificar com `bash -n` e um build `release` que gera `dist/lume-<hash>-release-debugsigned.apk`
- [x] 3.2 Em `.github/workflows/release.yml`, normalizar a tag com a mesma regra (`sed 's/^v*//'`), nomear o APK `lume-X.Y.Z[-debugsigned].apk` e titular o release "Lume vX.Y.Z" nas notas e no `gh release`; verificar lendo o diff e com `actionlint` se disponível
- [x] 3.3 Atualizar `README.md`, `README.pt-BR.md`, `CLAUDE.md` e `docs/handoff-dock-aurora.md` (nomes de APK, exemplo de `LUME_VERSION_NAME`, como a versão é definida); verificar com `grep -rn "lume-0\.\|LUME_VERSION_NAME" README* CLAUDE.md docs` que nada contradiz a regra nova

## 4. Verificação integrada

- [x] 4.1 Instalar o APK local na TCL e conferir que o canto das Configurações e a tela Sobre mostram o hash (e `-dirty` com uma edição pendente), e que `adb shell dumpsys package app.lume.tv | grep versionName` bate com a tela
- [x] 4.2 Endereço do repositório com fonte única: `repository = "https://github.com/douglaspands/lume-tv-launcher"` no `[workspace.package]` do `Cargo.toml` (herdado pelos crates), `SOURCE_URL` lendo `env!("CARGO_PKG_REPOSITORY")`, link da página de Releases (`…/releases`) no `README.md` e no `README.pt-BR.md` e link do repositório nas notas do `release.yml`; verificar com `grep -rn "lume-tv-launcher" --include=*.md --include=*.toml --include=*.rs --include=*.yml .` que todas as ocorrências são o mesmo endereço
- [x] 4.3 Revisar a tela Sobre: versão do build, endereço do repositório, licença e terceiros (o crate novo `lume-version` é do próprio Lume e não entra na lista); conferir com `cargo run -p lume-desktop -- --screenshot about a.png`
- [x] 4.4 Rodar `openspec validate versao-do-build --strict`
