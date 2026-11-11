# Tasks

## 1. Ponte Java e manifesto

- [x] 1.1 Em `AndroidManifest.xml`, declarar `com.android.tv.action.VIEW_INPUTS` em `<queries>`. Verificar com `adb shell dumpsys package app.lume.tv` e com o teste da tarefa 1.3.
- [x] 1.2 Em `LumeBridge`, adicionar os comandos `inputs` ("1" ou "0", via `resolveActivity`) e `open-inputs` (`startActivity` com `FLAG_ACTIVITY_NEW_TASK`, "ok" ou "missing"); conferir `proguard-rules.pro`. Verificar com `scripts/container-build.sh release` e `verify-apk.py` (limite 6 MB).
- [x] 1.3 Em `lume-app`, expor `inputs_available()` e `open_inputs()` na ponte (`bridge.rs`) e tratar `Effect::OpenInputs` em `android/mod.rs`. Verificar com `cargo test --workspace` e `cargo clippy --workspace --all-targets` limpos.

## 2. Barra superior e configurações (Rust)

- [x] 2.1 Adicionar o ícone de entradas em `scripts/subset-fonts.py` e regerar `icons.rs` e a fonte; comparar com o ícone do Google TV numa captura do desktop. Verificar com `cargo run -p lume-desktop -- --screenshot home h.png`.
- [x] 2.2 Em `topbar.rs`, incluir o botão de entradas (novo `ids::INPUTS`) à esquerda da engrenagem, presente só quando a ponte diz que existe; foco e contorno azul como a engrenagem. Teste de foco com e sem o botão em `app/tests.rs`.
- [x] 2.3 Em `mod.rs`, a engrenagem devolve `Effect::OpenAndroidSettings` e o botão devolve `Effect::OpenInputs`; `open_settings()` só pelo tile `GRID_SETTINGS`. Testes: OK na engrenagem, OK nas entradas, tile abre as configurações do Lume.
- [x] 2.4 Remover `settings_return` e `ITEM_ANDROID`; voltar das configurações leva o foco ao `GRID_SETTINGS` na aba Apps. Reescrever os testes que assumiam engrenagem → configurações do Lume (`tests.rs` ~419-488, 932, 968, 1790). Verificar com `cargo test --workspace`.

## 3. Textos e grafia "Lume"

- [x] 3.1 Em `values/strings.xml` e `values-pt-rBR/strings.xml`: `app_name` "Lume", "About Lume", "Sobre o Lume", comentários, e remover `settings_android`. Verificar com `cargo test -p lume-strings` e um `grep -rn "\blume\b"` sem sobras em texto de usuário.
- [x] 3.2 Atualizar README, `docs/translations.md`, `docs/device-notes.md`, `CLAUDE.md` e a prosa das specs tocadas para "Lume" e para o novo comportamento da engrenagem. Verificar por `grep` e `openspec validate v0-2-1 --strict`.
- [x] 3.3 Registrar o design system do frontend na spec `design-system` (tokens, tipografia, medidas, componentes, foco, regras de desenho) e apontá-la no `CLAUDE.md`. Verificar com `openspec validate v0-2-1 --strict`.

## 4. Painel de estatísticas

- [x] 4.1 Confirmar na TCL e na AIWA o que um app lê: `/proc/meminfo`, `/proc/stat`, frequência dos núcleos e `StatFs`; registrar em `docs/device-notes.md`. Verificar com um APK de debug e `adb logcat`.
- [x] 4.2 Na ponte Java, adicionar o comando `stats` (memória e disco); no Rust, interpretar a resposta e calcular o uso de CPU por duas leituras. Verificar com testes unitários do interpretador (resposta completa, sem CPU, malformada) e `cargo test --workspace`.
- [x] 4.3 Desenhar o painel à direita da tela de Configurações (cartão, barras na cor primária, traço quando falta CPU), sem foco. Verificar com `lume-desktop --screenshot settings` em 1280x720 e 1920x1080.
- [x] 4.4 Ligar o temporizador de 2 s só com `Screen::Settings` ativa e cancelá-lo ao sair, abrir a Sobre ou o editor e ir a segundo plano; respeitar `Shell::is_busy`. Verificar com testes do laço (nenhum quadro pedido fora da tela) e `cargo clippy --workspace --all-targets`.

## 5. Versão e verificação em dispositivo

- [x] 5.1 Subir a versão para 0.2.1 (`Cargo.toml`, `CLAUDE.md`, nomes de APK, `verify-apk.py`); gerar `dist/lume-0.2.1-release-debugsigned.apk` e verificar com `verify-apk.py` (limite 6 MB).
- [x] 5.2 Instalar na TCL (`.6`) e na AIWA (`.3`): botão de entradas abre o menu, engrenagem abre as configurações do Android, tile "Lume" abre as do Lume, Voltar devolve o foco ao tile. Confirmar também o painel de estatísticas com valores coerentes. Registrar em `docs/device-notes.md`.
- [x] 5.3 **Encerrada sem a validação em Android 9, por decisão do autor (2026-10-20): a máquina não tem emulador nem SDK de AVD, e os testes em Android 9 e em emulador ficam para uma spec futura** (instalar o Android Studio ou os `cmdline-tools` com `system-images;android-28;android-tv;x86`). Nada foi verificado em API 28 nesta change. Android 9: validar num emulador Android TV 9 (API 28) a instalação, a engrenagem e o botão de entradas escondido quando a ação não resolve; registrar o resultado ou, sem emulador, declarar a lacuna. Medir PSS, quadros perdidos (com a tela de Configurações aberta) e retorno a quente na AIWA (`low_ram`) contra os orçamentos de `resource-efficiency`.

## 6. Ajustes de espaço e botões (revisão do autor)

- [x] 6.1 Telas Configurações, Editar tela Início e Sobre: título na linha-base 104, subtítulo 140, conteúdo a partir de y=200; editor com enchimento 32, cartões com 24 de espaço, chips com 16 e 40 até "Restaurar padrão". Verificar com `lume-desktop --screenshot settings|editor` em 1280x720.
- [x] 6.2 Botões primário (azul do logo, anel de foco branco), secundário (contornado) e de perigo no tema (`.button`, `.button-secondary`, `.button-danger`) e no editor ("Concluído", "Restaurar padrão", diálogo). Verificar com `cargo test --workspace` e capturas.
- [x] 6.3 Registrar espaçamento e botões na spec `design-system` e os novos seletores na delta de `theming` e em `docs/theme-format.md`. Verificar com `openspec validate v0-2-1 --strict`.
- [x] 6.4 Configurações sem caminho e com a versão no rodapé direito; editor com caminho "Configurações › Tela Início", título "Tela Início", "Restaurar padrão" à esquerda e "Concluído" à direita sob o painel (Baixo vai para "Concluído", Esquerda para "Restaurar padrão"), foco inicial no primeiro favorito; título 64 px abaixo do caminho. Verificar com `cargo test --workspace` e capturas.
- [x] 6.5 Anel de foco azul (offset 4) também nos botões primário e de perigo, consistente com o design system; specs `design-system`, `home-editor` e `launcher-shell` atualizadas.
- [x] 6.6 Caminho "Configurações" mantido nas Configurações do Lume e "Configurações › Tela Início" no editor; Voltar do editor leva às Configurações (foco em "Tela Início") e de lá ao tile "Lume"; quem abre o editor pelo dock volta à tela Início. Verificar com `cargo test --workspace`.
- [x] 6.7 Revisão do autor na TCL e na AIWA com o APK reconstruído: aprovada pelo autor (2026-10-20).
