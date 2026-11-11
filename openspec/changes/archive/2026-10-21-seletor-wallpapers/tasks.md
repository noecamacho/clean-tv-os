# Tasks

## 1. Assets e Módulo de Wallpapers

- [x] 1.1 Gerar os arquivos de textura Raw 24-bit RGB dos 10 wallpapers a partir dos masters em alta definição e atualizar `crates/lume-core/build.rs` para embuti-los no binário
- [x] 1.2 Implementar o enum `WallpaperId` e a estrutura do catálogo de 10 wallpapers em `crates/lume-core/src/wallpaper/mod.rs`, permitindo consultar metadados, identificadores e texturas
- [x] 1.3 Adicionar testes unitários em `crates/lume-core/src/wallpaper/mod.rs` e `crates/lume-core/src/app/tests.rs` validando dimensões e razão de contraste (>= 4.5:1) de todos os 10 papéis de parede; verificar com `cargo test -p lume-core wallpaper`

## 2. Persistência de Configuração

- [x] 2.1 Adicionar a propriedade de papel de parede em `HomeConfig` (`crates/lume-core/src/home/config.rs`), com persistência da chave `wallpaper` em `config.v1` e fallback padrão para `"aurora"`
- [x] 2.2 Adicionar testes unitários cobrindo round-trip, arquivo sem a chave e reparo de valor inválido em `crates/lume-core/src/home/config.rs`; verificar com `cargo test -p lume-core home::config`

## 3. Renderização Dinâmica de Textura na GPU

- [x] 3.1 Atualizar `crates/lume-core/src/render/gles2.rs` para rastrear o identificador do wallpaper carregado na GPU e recriar/atualizar a textura quando o papel de parede ativo mudar
- [x] 3.2 Atualizar `crates/lume-core/src/app/painter.rs` e `crates/lume-core/src/app/mod.rs` para desenhar o wallpaper ativo selecionado na configuração; verificar com `cargo test -p lume-core render`

## 4. Modo Edição: Mini-Cards e Remoção de Concluído

- [x] 4.1 Adicionar strings traduzidas para os nomes dos 10 papéis de parede e título da seção em `android/app/src/main/res/values/strings.xml` e `values-pt-rBR/strings.xml`
- [x] 4.2 Implementar a linha de Mini-Cards (178x100) com miniaturas dos wallpapers, rótulos e foco D-Pad em `crates/lume-core/src/app/editor.rs`, aplicando a troca e persistência imediata ao selecionar com OK
- [x] 4.3 Remover o botão `DONE` ("Concluído") da tela de edição em `crates/lume-core/src/app/editor.rs`, ajustando a navegação para que descer dos apps aponte para `RESTORE` e o retorno ocorra via botão Voltar (`back_from_editor`)
- [x] 4.4 Atualizar testes da tela de edição em `crates/lume-core/src/app/tests.rs` refletindo a nova navegação sem botão Concluído e a seleção de papel de parede; verificar com `cargo test -p lume-core editor`

## 5. Validação Geral e Tela Sobre

- [x] 5.1 Gerar capturas de tela do modo edição e da tela inicial com wallpapers alternados via `cargo run -p lume-desktop -- --screenshot editor` e verificar conformidade com o design system
- [x] 5.2 Executar suite completa de testes e linter com `cargo test --workspace && cargo clippy --workspace --all-targets` e garantir saída totalmente limpa
- [x] 5.3 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume) com `cargo run -p lume-desktop -- --screenshot about` e atualizar se houver novas dependências/licenças ou registrar "Sobre revisado, sem mudança"

## 6. Qualidade dos papéis em TVs HD e 4K

- [x] 6.1 Gerar os 10 papéis em 1920x1080 (Full HD) com dithering (`tools/wallpaper/aurora06.py`, `generate_wallpapers.py`), atualizar `crates/lume-core/build.rs` e `src/wallpaper/mod.rs` e ajustar a calibração do topo para o pico de luminância; verificar com `cargo test --workspace`
- [x] 6.2 Subir os orçamentos (APK 40 MB, PSS 64 MB) em `scripts/verify-apk.py`, `CLAUDE.md` e no delta de `resource-efficiency`
- [x] 6.3 Medir na TCL e na AIWA: PSS por categoria, primeiro quadro, interface pronta e quadros de 33 ms ou mais; comparar visualmente os degradês em 4K e registrar em `docs/device-notes.md`
- [x] 6.4 Splash: tempo mínimo do logo (`--splash-hold-cold` 600 ms, `--splash-hold-warm` 300 ms), entrada de 300 ms com escala, saída em 60% da revelação (600 ms a frio); atualizar `splash.rs` e os testes
- [x] 6.5 Descrição no painel de favoritos do editor com espaço de 48 px até os cartões

