# Proposal

## Why

O Lume roda em Smart TVs Android TV de entrada com hardware severamente restrito — como a **TCL 55C715** (SoC Realtek `rtd285o`, 4× ARM Cortex-A53 32 bits a 1,1 GHz, GPU Mali-470 MP, 2 GB de RAM) e a **AIWA SMART TV** (SoC MediaTek `mt5867`, 4× ARM Cortex-A53 32 bits a 1,5 GHz, GPU Mali-G31, 913 MB de RAM com `low_ram=true`). Nesses processadores *in-order* com cache L1/L2 pequeno e GPUs de baixo preenchimento, qualquer desperdício algorítmico, alocação por quadro ou troca desnecessária de estado OpenGL ES 2.0 reflete diretamente em quadros perdidos:

1. **Na Grade de Apps (`apps.rs`)**, a TCL ainda apresenta **1,5% de quadros de 33 ms ou mais** (acima do limite de 1% da spec): `grid_cells()` clona e reordena todo o catálogo com `sort_by_cached_key` em todo quadro (60 fps), o tile `"Lume"` desenha o logo (`logo::draw_mark`, que emite 4 `Cmd::Rect`) dentro do laço de texto (quebrando o batch de `Text` ao meio e forçando trocas extras de shader/VBO na GPU), e tiles fora da área visível continuam medindo e truncando strings todo quadro.
2. **No Editor da Tela Inicial (`editor.rs`)**, a TCL roda travada a **33 ms por quadro (30 fps)** mesmo apenas movendo o foco: `editor_pass` executa duas vezes por quadro (uma para medir a altura e outra para desenhar), dispara animações em duplicidade e processa todas as linhas fora da área visível (`clip`).
3. **No Motor de Texto (`text/mod.rs` e `text/system_fonts.rs`)**, `ellipsize` faz busca binária alocando `String` com `format!` a cada passo e remedindo do índice zero ($O(K \log K)$), `draw_line` e `draw` medem a mesma string até 3 vezes por rótulo, e `SystemFonts::covers` faz *parse* completo das tabelas TrueType (`ttf_parser::Face::parse`) a cada caractere ausente sem cachear as faces.
4. **No Backend GLES2 e nos Limitadores (`render/gles2.rs`, `app_icons.rs`, `shell/mod.rs`, `android/mod.rs`)**, o envio de vértices concatena `text_vertices` e `image_vertices` num novo `Vec<TexVertex>` a cada quadro; `reduce_wallpaper` recria VBO e consulta uniform locations por string; `WALLPAPER_UPLOAD_ROWS = 40` leva 27 quadros (~450 ms) para subir um papel; `ICONS_PER_TURN = 1` re-varre o catálogo inteiro duas vezes por ícone sem *short-circuit*; e `AtlasMap::reserve` não recicla slots liberados.
5. **Na Compilação (`Cargo.toml` e `android/app/build.gradle`)**, o perfil `release` usa `opt-level = "z"` (tamanho), o que desativa o desenrolamento de laços (*loop unrolling*) e a autovetorização SIMD NEON do LLVM para os Cortex-A53/A55, apesar de a `liblume.so` ter ~1 MB e o APK ter 21,6 MB (folga de 18,4 MB abaixo do teto de 40 MB).

Esta change realiza o **tuning completo da aplicação e da compilação** em 100% Rust, guiado por **TDD, Loop Engineering e Graph Engineering** (subagentes paralelos, com instruções passo a passo desenhadas para que até modelos mais leves executem sem ambiguidade), e exige um **benchmark comparativo Antes vs. Depois** para comprovar o ganho real.

## What Changes

- **Otimização do Motor de Texto e Fontes (`crates/lume-core/src/text/`)**:
  - Substituir a busca binária com alocação em `TextEngine::ellipsize` por varredura linear única $O(K)$ acumulando larguras + kerning sem alocar `String` intermediária.
  - Eliminar a tripla medição em `draw_line`/`draw` retornando a largura junto com as posições ou reutilizando a medição no alinhamento horizontal.
  - Cachear as faces parseadas (ou tabelas de cobertura) em `SystemFonts` para evitar `ttf_parser::Face::parse` repetido em cada *cache miss* de glifo.
- **Tuning de Renderização, Culling e Batching (`crates/lume-core/src/app/`, `crates/lume-core/src/render/`)**:
  - **Grade de Apps (`apps.rs`)**: substituir `sort_by_cached_key` por quadro em `grid_cells()` por inserção ordenada $O(N)$ (`partition_point`) do tile `"Lume"` sobre a fatia já ordenada de `Catalog::apps()`; mover `logo::draw_mark` do tile `"Lume"` para o laço de caixas (`Rect`), garantindo estritamente 1 draw call de `Rect`, 1 de `Image` e 1 de `Text`; aplicar *viewport culling* vertical (`rect.bottom() + 40.0 > CLIP_TOP && rect.y < visible.bottom()`) antes de desenhar tiles e rótulos.
  - **Editor (`editor.rs`)**: eliminar o duplo passo (`editor_pass(p, false, ...)` seguido de `editor_pass(p, true, ...)`), calculando a altura do conteúdo sem duplicar layout/medições/animações, e aplicar *viewport culling* vertical nas linhas roláveis.
  - **Telas Sobre e Informações da TV (`settings.rs`, `licenses.rs`)**: cachear `licenses::lines()` em memória estática (`OnceLock<Vec<String>>`) para zerar alocações por quadro em `draw_about`, e aplicar *viewport culling* vertical nas linhas de `draw_about` e `draw_tv_info`.
  - **Backend OpenGL ES 2.0 (`gles2.rs`)**: eliminar a alocação por quadro de `tex_data: Vec<TexVertex>` em `Gles2Backend::render` (usando `glBufferData` + dois `glBufferSubData` direto de `frame.text_vertices` e `frame.image_vertices`); cachear os uniform locations (`u_step`, `u_dest`) no `ShaderProgram` e reutilizar um VBO fixo em `reduce_wallpaper`; inverter linhas *in-place* em `read_pixels` sem alocar segundo buffer de tela cheia; elevar `WALLPAPER_UPLOAD_ROWS` de `40` para `90` (reduzindo o envio de 27 para 12 quadros).
- **Tuning de Limitadores, Catálogo, Storage e Atlas de Ícones (`crates/lume-core/src/`, `crates/lume-app/src/android/`)**:
  - **Reciclagem de Slots no Atlas de Ícones (`app_icons.rs`)**: adicionar *free-list* em `AtlasMap` para reaproveitar slots liberados em `remove()`.
  - **Fila de Ícones com Short-Circuit (`shell/mod.rs`, `android/mod.rs`)**: substituir `.fold()` por iterador com `.take(limit)` em `Shell::icons_wanted` e buscar a fila do lote em `load_icons_batch` sem re-varrer o catálogo inteiro a cada ícone individual (`ICONS_PER_TURN`).
  - **Estruturas $O(N)$ em Catálogo e Storage (`catalog/mod.rs`, `storage.rs`)**: otimizar `Catalog::choose_activities`, `Catalog::replace_with` e `Kv` para evitar buscas lineares $O(N^2)$ na carga e atualização de apps.
- **Tuning de Compilação para ARM Cortex-A53/A55 (`Cargo.toml`, `android/app/build.gradle`, `scripts/container-build.sh`)**:
  - Mudar `[profile.release]` de `opt-level = "z"` para `opt-level = 3`, mantendo `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` e `strip = true`.
  - Configurar flags de arquitetura do Rust para Android: `-C target-cpu=cortex-a53` em `aarch64-linux-android` e `-C target-cpu=cortex-a53 -C target-feature=+neon` em `armv7-linux-androideabi`, habilitando escalonamento de instruções para pipeline *in-order* e vetorização SIMD NEON de 128 bits nas TVs 32 bits.
- **Revisão dos Orçamentos na Spec `resource-efficiency` e Benchmark Antes vs. Depois**:
  - Atualizar `resource-efficiency` para medir o retorno a quente sob controle do Lume (**≤ 150 ms** da janela entregue ao primeiro quadro), explicitar o critério de memória privada em segundo plano (**Private Dirty ≤ 10 MB**) para TVs `low_ram` onde o PSS compartilhado do Android infla a conta, e manter o rigor de **≤ 1%** de quadros ≥ 33 ms em todas as telas (incluindo Grade e Editor).
  - Criar e executar um **benchmark automatizado Antes vs. Depois** (`cargo test --release -p lume-core -- --nocapture bench_`) medindo tempos de CPU por quadro nas telas (`home`, `apps`, `editor`, `about`), truncamento de texto (`ellipsize`) e construção de batches, além das medidas em dispositivo real quando disponível.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `resource-efficiency`: refina o requisito de **Retorno a quente** para medir a latência sob controle do Lume (até 150 ms entre a entrega da `ANativeWindow` e o primeiro quadro desenhado), adiciona ao requisito de **Memória em segundo plano** o critério de memória privada (`Private Dirty` ≤ 10 MB) para dispositivos `low_ram`, e inclui a verificação de **Fluidez de 60 quadros por segundo** também na navegação do Editor e da Grade de Apps após o tuning.

## Impact

- **Rust (`crates/lume-core`)**: `text/mod.rs`, `text/system_fonts.rs`, `app/apps.rs`, `app/editor.rs`, `app/settings.rs`, `licenses.rs`, `render/gles2.rs`, `app_icons.rs`, `shell/mod.rs`, `catalog/mod.rs`, `storage.rs`.
- **Rust (`crates/lume-app`)**: `android/mod.rs`, `android/bridge.rs`, `android/egl.rs` (100% Rust; nenhum arquivo `.java` é alterado).
- **Build**: `Cargo.toml`, `.cargo/config.toml` (ou variáveis `CARGO_TARGET_*_RUSTFLAGS` em `android/app/build.gradle` e `scripts/container-build.sh`).
- **Documentação e Benchmarks**: `docs/device-notes.md` (tabela comparativa Antes vs. Depois do tuning).
