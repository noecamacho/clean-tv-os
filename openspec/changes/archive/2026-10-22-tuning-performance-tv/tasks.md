# Tasks

> **Instruções para Agentes e Subagentes (inclusive modelos leves `flash` / `flash_lite`):**
> - **Sempre use `zsh -c "..."`** para executar comandos no terminal.
> - **Respeite a fronteira de arquivos de cada nó:** na Onda 1, cada subagente só pode editar os arquivos listados no seu grupo.
> - **Siga o Loop TDD (Red → Green → Verify):** escreva/atualize o teste primeiro, aplique a mudança exata descrita na tarefa e rode o comando de verificação indicado na própria tarefa até passar com zero erros.
> - **Regra 100% Rust:** NÃO edite nenhum arquivo `.java`.

## 1. Onda 0 (Sequencial): Benchmark de Base (Antes do Tuning)

- [x] 1.1 Criar o arquivo `crates/lume-core/tests/perf_benchmark.rs` contendo um teste `#[test] #[ignore] fn bench_before_after_tuning()` que meça (com `std::time::Instant`, aquecimento de 50 iterações e média de 500 iterações) e imprima no `stdout`:
  1. Tempo médio (µs) de 100 chamadas a `TextEngine::ellipsize` com string longa (`"Um Aplicativo de Streaming com Nome Muito Longo para TV 4K"`, `max_width = 140.0`) e string curta (`"YouTube"`, `max_width = 140.0`).
  2. Tempo médio (µs) e contagem de `Frame::build(&list).batches.len()` de um quadro completo da Grade de Apps (`Tab::Apps`) com 48 apps no catálogo.
  3. Tempo médio (µs) de um quadro completo do Editor (`Screen::Editor`, aberto via `Key::Menu` ou `open_editor`).
  4. Tempo médio (µs) de um quadro completo da tela Sobre (`Screen::About`).
  5. Tempo médio (µs) de 1.000 chamadas a `Shell::icons_wanted(1)` com 48 apps no catálogo.
- [x] 1.2 Executar `zsh -c "cargo test --release -p lume-core --test perf_benchmark -- --ignored --nocapture"` no código atual (antes do tuning) e guardar os números da base para a comparação final.

## 2. Onda 1 — Nó A (Paralelo): Motor de Texto e Fontes (`crates/lume-core/src/text/mod.rs`, `crates/lume-core/src/text/system_fonts.rs`)

> **Escopo exclusivo de arquivos do Nó A:** `crates/lume-core/src/text/mod.rs` e `crates/lume-core/src/text/system_fonts.rs`. Não edite nenhum outro arquivo.

- [x] 2.1 **[TDD - Red]** Em `crates/lume-core/src/text/mod.rs` (no módulo `mod tests`), adicionar o teste `ellipsize_single_pass_matches_exact_boundaries` verificando que:
  - Uma string que cabe exatamente em `max_width` retorna `Cow::Borrowed` (sem alocar `String`).
  - Uma string longa truncada com reticências tem `engine.measure(&result, &BODY) <= max_width`.
  - Espaços em branco antes das reticências são removidos (`"Hello world"` truncado após `"Hello "` vira `"Hello…"` e não `"Hello …"`).
- [x] 2.2 **[TDD - Green]** Em `crates/lume-core/src/text/mod.rs`, reescrever `TextEngine::ellipsize` (linhas 210–254) sem busca binária e sem `format!` dentro de laço:
  - Se `text.is_empty() || max_width <= 0.0`, retornar `Cow::Borrowed("")`.
  - Calcular `let ellipsis_w = self.char_advance(ELLIPSIS, None, px, weight, 0.0);`.
  - Percorrer `text.char_indices()` uma única vez acumulando `width` (somando `spacing` quando não é o primeiro caractere + `self.char_advance(ch, prev, px, weight, spacing)`).
  - A cada caractere, se a fatia `let trimmed = text[..idx + ch.len_utf8()].trim_end();` não for vazia e `width_of_trimmed + kern_trimmed_last_to_ellipsis + ellipsis_w <= max_width`, atualizar `best_end = trimmed.len()`.
  - Ao final do laço, se `width <= max_width`, retornar `Cow::Borrowed(text)`. Se ` width > max_width` e `ellipsis_w > max_width`, retornar `Cow::Borrowed("")`. Se `best_end == 0`, retornar `Cow::Borrowed("…")`. Caso contrário, retornar `Cow::Owned(format!("{}…", &text[..best_end]))`.
- [x] 2.3 **[TDD - Green]** Em `crates/lume-core/src/text/mod.rs`, otimizar `draw_line` (linhas 282–305) para não medir o texto duas vezes após `ellipsize`: criar um método auxiliar `fn pen_positions_and_width(&self, text: &str, style: &TextStyle) -> (Vec<(GlyphKey, f32)>, f32)` e usá-lo em `draw_line` e `draw`, calculando `x` a partir da largura já retornada e emitindo os glifos diretamente sem segunda chamada a `measure` ou `pen_positions`.
- [x] 2.4 **[TDD - Green]** Em `crates/lume-core/src/text/system_fonts.rs`, evitar chamar `ttf_parser::Face::parse` repetidamente do zero para o mesmo arquivo em `SystemFonts::covers`: guardar junto de `LoadedFont` (ou num `Vec<Option<...>>` paralelo por candidato) os subtables/dados ou checar cobertura sem re-parsear arquivos já descartados/inválidos, mantendo todos os testes de `system_fonts` passando.
- [x] 2.5 **[Verify Nó A]** Executar `zsh -c "cargo test -p lume-core text::"` e confirmar que todos os testes passam sem warnings.

## 3. Onda 1 — Nó B (Paralelo): Grade de Apps, Editor, Configurações e Licenças (`crates/lume-core/src/app/apps.rs`, `crates/lume-core/src/app/editor.rs`, `crates/lume-core/src/app/settings.rs`, `crates/lume-core/src/licenses.rs`)

> **Escopo exclusivo de arquivos do Nó B:** `crates/lume-core/src/app/apps.rs`, `crates/lume-core/src/app/editor.rs`, `crates/lume-core/src/app/settings.rs`, `crates/lume-core/src/licenses.rs`. Não edite nenhum outro arquivo.

- [x] 3.1 **[TDD - Red]** Em `crates/lume-core/src/app/apps.rs` (dentro de `#[cfg(test)] mod tests`), adicionar um teste `apps_grid_does_not_split_text_batch_around_lume_tile` que desenha a aba Apps com apps antes e depois de `"Lume"` (ex.: `"Apple TV"` e `"YouTube"`), constrói `let frame = crate::render::Frame::build(&list);` e verifica que existe **no máximo 1 batch com `Program::Text`** dentro do recorte da grade (`clip.is_some()`), provando que `logo::draw_mark` não parte mais o batch de texto ao meio.
- [x] 3.2 **[TDD - Green]** Em `crates/lume-core/src/app/apps.rs`:
  1. Em `grid_cells(&self)` (linhas 41–75): remover `cells.sort_by_cached_key(...)`. Como `self.catalog.apps()` já vem ordenado por `(sort_key, package)`, calcular `let lume_key = normalize::sort_key(&lume_label);`, achar o índice de inserção com `let insert_at = self.catalog.apps().partition_point(|a| a.sort_key.as_str() <= lume_key.as_str());` e construir o `Vec<Cell>` em uma única passada $O(N)$ atribuindo `rect = tile_rect(i)` para cada posição `i`.
  2. Em `draw_apps(&mut self, p: &mut Painter)` (linhas 149–199):
     - Filtrar `tiles` com *viewport culling* vertical (`let view_bottom = visible.bottom();`, mantendo células com `rect.bottom() + 40.0 >= CLIP_TOP - 24.0 && rect.y <= view_bottom + 24.0`).
     - Mover o bloco `if cell.package.is_none() { ... crate::logo::draw_mark(p.list, mark, vars::PRIMARY, vars::BG); }` do 3º laço (linhas 177–189) para dentro do 1º laço (logo após `p.draw_box(*rect, style)`), mantendo no 3º laço apenas `if cell.package.is_some() { self.draw_tile_name(...); }` e `p.text(&cell.label, ...)`.
- [x] 3.3 **[TDD - Green]** Em `crates/lume-core/src/app/editor.rs`:
  1. Evitar que o passo `draw = false` (`let (_, content_bottom) = self.editor_pass(p, false, 0.0);`) dispare animações (`if draw { self.anim.animate_to(...); }`) ou meça/desenhe textos quando `draw == false`.
  2. Quando `draw == true` em `editor_pass`, aplicar *viewport culling* vertical: definir `let cull_top = inner.y - 32.0; let cull_bottom = inner.bottom() + 32.0;` e só emitir comandos de desenho (`p.draw_box`, `p.text`, `p.icon`, `draw_preview`, `draw_wallpaper_selector`, `draw_row`) para seções/linhas cujo retângulo vertical intersecte `cull_top..cull_bottom` (mantendo a adição de todas as `Row`s em `rows` para que a navegação por foco continue idêntica).
- [x] 3.4 **[TDD - Green]** Em `crates/lume-core/src/licenses.rs` e `crates/lume-core/src/app/settings.rs`:
  1. Em `licenses.rs`: adicionar `pub fn cached_lines() -> &'static [String]` usando `static LINES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();` e fazer `pub fn lines() -> Vec<String>` retornar `cached_lines().to_vec()`.
  2. Em `settings.rs` (`draw_about`, linhas 296–345): usar `licenses::cached_lines()` com `std::borrow::Cow::Borrowed(line.as_str())` para não alocar `Vec<String>` por quadro, e no laço de desenho só chamar `p.text(text, rect, ...)` se `y + height >= ABOUT_VIEW_TOP && y <= visible.bottom()`.
  3. Em `settings.rs` (`draw_tv_info`, linhas 260–275): só chamar `p.text(...)` para títulos e linhas cujo `[y, y + height]` intersecte `TV_VIEW_TOP..visible.bottom()`.
- [x] 3.5 **[Verify Nó B]** Executar `zsh -c "cargo test -p lume-core app:: && cargo test -p lume-core licenses::"` e confirmar que todos os testes passam sem warnings.

## 4. Onda 1 — Nó C (Paralelo): Backend OpenGL ES 2.0 e Upload de Wallpaper (`crates/lume-core/src/render/gles2.rs`)

> **Escopo exclusivo de arquivos do Nó C:** `crates/lume-core/src/render/gles2.rs`. Não edite nenhum outro arquivo.

- [x] 4.1 **[TDD - Red]** Em `crates/lume-core/src/render/gles2.rs` (ou em um função auxiliar pura testável `flip_rows_in_place(buf: &mut [u8], stride: usize)` no fim de `gles2.rs`), adicionar um teste unitário `flip_rows_in_place_reverses_top_to_bottom_without_allocating` verificando que uma imagem de 3 linhas (`stride = 8`) tem a linha 0 e a linha 2 invertidas *in-place* preservando a linha 1.
- [x] 4.2 **[TDD - Green]** Em `crates/lume-core/src/render/gles2.rs`:
  1. Na linha 35, alterar `const WALLPAPER_UPLOAD_ROWS: u32 = 40;` para `const WALLPAPER_UPLOAD_ROWS: u32 = 90;`.
  2. Em `ShaderProgram` (linhas 37–43) e `compile_program` (linhas 237–252): adicionar os campos `u_step: Option<glow::UniformLocation>` e `u_dest: Option<glow::UniformLocation>`, resolvendo-os uma única vez em `compile_program` (assim como `u_view`, `u_surface`, etc.).
  3. Em `reduce_wallpaper` (linhas 586–609): usar `self.reduce_program.u_step.as_ref()` e `self.reduce_program.u_dest.as_ref()` em vez de chamar `self.gl.get_uniform_location(...)` por string, e usar `self.tex_vbo` (em vez de criar e deletar um `vbo` temporário com `create_buffer`/`delete_buffer`).
  4. Em `read_pixels` (linhas 764–781): usar `flip_rows_in_place(&mut bottom_up, stride)` e retornar `bottom_up` diretamente, eliminando a alocação de um segundo `Vec<u8>` de tela cheia.
  5. Em `render` (linhas 921–930): eliminar `let mut tex_data = Vec::with_capacity(...)`. Quando `!frame.text_vertices.is_empty() || !frame.image_vertices.is_empty()`, fazer `bind_buffer(glow::ARRAY_BUFFER, Some(self.tex_vbo))`, alocar o buffer no driver com `self.gl.buffer_data_size(glow::ARRAY_BUFFER, total_bytes as i32, glow::STREAM_DRAW)` e enviar as duas fatias com `self.gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, bytemuck::cast_slice(&frame.text_vertices))` e `self.gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, text_bytes as i32, bytemuck::cast_slice(&frame.image_vertices))`.
- [x] 4.3 **[Verify Nó C]** Executar `zsh -c "cargo test -p lume-core render:: && cargo test -p lume-desktop"` e confirmar que todos os testes GL e de fidelidade de imagem passam.

## 5. Onda 1 — Nó D (Paralelo): Reciclagem de Atlas, Fila de Ícones, Catálogo e Storage (`crates/lume-core/src/app_icons.rs`, `crates/lume-core/src/shell/mod.rs`, `crates/lume-core/src/catalog/mod.rs`, `crates/lume-core/src/storage.rs`, `crates/lume-app/src/android/mod.rs`)

> **Escopo exclusivo de arquivos do Nó D:** `crates/lume-core/src/app_icons.rs`, `crates/lume-core/src/shell/mod.rs`, `crates/lume-core/src/catalog/mod.rs`, `crates/lume-core/src/storage.rs`, `crates/lume-app/src/android/mod.rs`. Não edite nenhum outro arquivo.

- [x] 5.1 **[TDD - Red]** Em `crates/lume-core/src/app_icons.rs` (`mod tests`), adicionar o teste `removed_slots_are_reused_before_hitting_capacity` que insere `CAPACITY` pacotes, remove 2 pacotes, insere 2 novos pacotes com nomes diferentes e verifica que ambos retornam `Some(region)` com `icons.len() == CAPACITY` e sem colidir com os pacotes ativos.
- [x] 5.2 **[TDD - Green]** Em `crates/lume-core/src/app_icons.rs`:
  - Adicionar `free_slots: Vec<usize>` em `struct AtlasMap`.
  - Em `AtlasMap::clear(&mut self)`, chamar `self.free_slots.clear()`.
  - Em `AtlasMap::remove(&mut self, package: &str)`, mudar para `if let Some((slot, _)) = self.slots.remove(package) { self.free_slots.push(slot); true } else { false }`.
  - Em `AtlasMap::reserve(&mut self, package: &str)`, obter o slot de `self.free_slots.pop().unwrap_or(self.slots.len())`.
- [x] 5.3 **[TDD - Green]** Em `crates/lume-core/src/shell/mod.rs` (linhas 155–185) e `crates/lume-app/src/android/mod.rs` (linhas 68–77 e 406–456):
  - Em `Shell::icons_wanted(&self, limit: usize)`: se `limit == 0`, retornar `Vec::new()`. Usar um `HashSet<&str>` para evitar duplicatas e iterar sobre `priority.chain(rest)` com `.filter(...).take(limit).collect()`, parando imediatamente ao encontrar `limit` pacotes.
  - Em `crates/lume-app/src/android/mod.rs`: alterar `const ICONS_PER_TURN: usize = 1;` para `const ICONS_PER_TURN: usize = 8;` e passar `(started: Instant, budget: Duration)` para `load_icons(&mut self, started: Instant, budget: Duration) -> bool`, checando `if started.elapsed() >= budget { break; }` antes de cada ícone dentro dos laços de `dock_wanted` e `tiles_wanted`.
- [x] 5.4 **[TDD - Green]** Em `crates/lume-core/src/catalog/mod.rs` e `crates/lume-core/src/storage.rs`:
  - Em `Catalog::choose_activities` (linhas 160–177): usar um `HashMap<&str, usize>` mapeando `package -> índice em chosen` para agrupar atividades do mesmo pacote em $O(N)$ em vez de `chosen.iter().position(...)` ($O(N^2)$).
  - Em `Catalog::replace_with` (linhas 182–207): criar um `HashMap<&str, (Option<u64>, bool)>` dos apps atuais antes de montar a nova lista para buscar `installed_at` e `system` em $O(1)$ em vez de `.iter().find(...)` linear.
- [x] 5.5 **[Verify Nó D]** Executar `zsh -c "cargo test -p lume-core app_icons:: && cargo test -p lume-core shell:: && cargo test -p lume-core catalog:: && cargo test -p lume-core storage:: && cargo check -p lume-app"` e confirmar que tudo compila e passa.

## 6. Onda 1 — Nó E (Paralelo): Tuning de Compilação para ARM Cortex-A53 e NEON (`Cargo.toml`, `.cargo/config.toml`)

> **Escopo exclusivo de arquivos do Nó E:** `Cargo.toml` e `.cargo/config.toml`. Não edite nenhum outro arquivo.

- [x] 6.1 Em `Cargo.toml` (linhas 19–24), alterar `[profile.release]` de `opt-level = "z"` para `opt-level = 3`, mantendo `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` e `strip = true`.
- [x] 6.2 Criar `.cargo/config.toml` configurando as flags de CPU para os alvos Android:
  - `[target.aarch64-linux-android]` com `rustflags = ["-C", "target-cpu=cortex-a53"]`
  - `[target.armv7-linux-androideabi]` com `rustflags = ["-C", "target-cpu=cortex-a53", "-C", "target-feature=+neon"]`
- [x] 6.3 **[Verify Nó E]** Executar `zsh -c "cargo check --workspace --all-targets"` para garantir que a configuração do Cargo é válida.

## 7. Onda 2 (Sequencial): Validação Integrada, Benchmark Antes vs. Depois e Tela Sobre

- [x] 7.1 Executar a suíte completa de testes, lint e formatação: `zsh -c "cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check"`.
- [x] 7.2 Executar o benchmark pós-tuning: `zsh -c "cargo test --release -p lume-core --test perf_benchmark -- --ignored --nocapture"`, calcular o ganho (*speedup* e redução de tempo/batches) em relação à base da tarefa 1.2, e registrar a tabela comparativa **Antes vs. Depois** em `docs/device-notes.md`.
- [x] 7.3 Validar a change no OpenSpec com `zsh -c "openspec validate tuning-performance-tv --strict"`.
- [x] 7.4 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume): verificar se alguma crate, fonte, ícone, licença ou endereço do repositório mudou (`zsh -c "cargo run -p lume-desktop -- --size 1920x1080 --locale pt-BR --screenshot about /tmp/about.png"`); como não há componente de terceiros novo, registrar `"Sobre revisado, sem mudança"`.
