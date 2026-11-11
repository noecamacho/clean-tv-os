# Tasks

## 1. Persistência e traduções

- [x] 1.1 Adicionar `pub round_favorites: bool` a `HomeConfig` em `crates/lume-core/src/home/config.rs` (padrão `false` em `first_use`, gravado como `round_favorites=1`/`0` em `to_kv` e lido em `from_kv` com fallback `false` para arquivos sem a chave); adicionar testes unitários de round-trip e compatibilidade com arquivos antigos e verificar com `cargo test -p lume-core -q config`
- [x] 1.2 Adicionar `editor_round_favorites` em `android/app/src/main/res/values/strings.xml` ("Round favorites") e `values-pt-rBR/strings.xml` ("Favoritos redondos") e expor `StringId::EditorRoundFavorites`; ajustar `.editor-row` em `themes/lume/theme.css` (`height: 64px`); verificar com `cargo test --workspace -q`

## 2. Editor da tela Início (`Configurações › Tela Início`)

- [x] 2.1 Adicionar a linha `.editor-row` "Favoritos redondos" com chave Liga/Desliga (`48x28`, bolinha `20x20`) em `crates/lume-core/src/app/editor.rs` no topo do conteúdo (`y = 184`, `h = 64`), definir o foco inicial nela (`ROUND_FAVORITES`), alternar `self.config.round_favorites` com OK emitindo `Effect::SaveConfig` e garantir que "Restaurar padrão" volte para `false`; atualizar os vãos do editor (`ROW_GAP = 16`, `PANEL_PADDING_Y = 24`, cabeçalho→cartões `32`, `CARD_GAP = 16` para 6 cartões por linha, cartões→chips `20`, `BUTTONS_GAP = 24`) e o teste `the_layout_is_on_the_ruler`; adicionar/atualizar testes em `crates/lume-core/src/app/tests.rs` e verificar com `cargo test -p lume-core -q`

## 3. Recorte de ícones do dock (quadrado arredondado ou circular)

- [x] 3.1 Suportar raio de canto em `Cmd::Image` (`DisplayList::image_rounded` / `TexVertex` / `IMAGE_FRAGMENT` em `crates/lume-core/src/render/`), recortando imagens com `radius > 0` pela distância assinada com antialiasing; usar `radius_ratio = if self.config.round_favorites { 0.5 } else { 0.22 }` em `crates/lume-core/src/app/dock.rs` para fundo, imagem e `dock_ghost`; adicionar testes unitários em `render/batch.rs` e `app/dock.rs` e verificar com `cargo test -p lume-core -q`
- [x] 3.2 Em `android/app/src/main/java/app/lume/tv/LumeBridge.java`, fazer `renderDockIcon` entregar o quadrado completo do ícone sem `drawRoundRect` (deixando o recorte para a GPU) e subir `DOCK_CACHE_VERSION` de `"d1"` para `"d2"`; verificar compilação e lint com `cargo clippy --workspace --all-targets -q`

## 4. Verificação integrada

- [x] 4.1 Gerar capturas em `1280x720` com `lume-desktop` da tela `editor` (com 6 e 12 favoritos, ligada e desligada) e da tela `home` (com favoritos quadrados arredondados e redondos), medir os vãos com `scripts/measure-ink.py` e conferir contra a régua de `design-system`
- [x] 4.2 Na TCL e na AIWA: abrir `Configurações › Tela Início`, alternar "Favoritos redondos", verificar que o dock muda entre quadrado arredondado e circular na hora sem recarregar ícones e que "Restaurar padrão" devolve para desligado
- [x] 4.3 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume) e atualizá-la se a change trouxe componente de terceiros, endereço do repositório ou versão; sem novidade, registrar "Sobre revisado, sem mudança" e verificar com `cargo run -p lume-desktop -- --screenshot about`
- [x] 4.4 Validar a change com `openspec validate favoritos-redondos --strict`
