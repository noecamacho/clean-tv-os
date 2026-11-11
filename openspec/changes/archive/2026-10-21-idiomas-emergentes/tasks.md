# Tasks

## 1. Arquivos de recursos e regras de tradução

- [x] 1.1 Criar `android/app/src/main/res/values-es/strings.xml` com a tradução completa em espanhol para todas as chaves e plurais do arquivo de referência `values/strings.xml`
- [x] 1.2 Criar `android/app/src/main/res/values-id/strings.xml` com a tradução completa em indonésio (Bahasa Indonesia) para todas as chaves e plurais
- [x] 1.3 Criar `android/app/src/main/res/values-ru/strings.xml` com a tradução completa em russo para todas as chaves e plurais
- [x] 1.4 Validar o mapeamento das famílias de plural em `crates/lume-core/src/i18n/plural.rs` e verificar paridade e compilação das strings com `cargo test -p lume-strings -q` e `cargo test --workspace -q`

## 2. Verificação visual e régua de layout

- [x] 2.1 Gerar capturas das telas (`home`, `apps`, `settings`, `about`, `editor`) em `1280x720` para os idiomas `es`, `id` e `ru` usando `lume-desktop` e validar com `scripts/measure-ink.py` contra a régua do design system
- [x] 2.2 Verificar que nenhum texto sofre sobreposição ou quebra indesejada com `cargo run -p lume-desktop -- --pseudo-locale --screenshot editor /tmp/editor-pseudo.png`
- [x] 2.3 Validar em TV (a TCL ou a AIWA) trocando o idioma do sistema para espanhol, indonésio e russo e verificando a troca imediata no retorno à tela inicial

## 3. Fechamento e conformidade

- [x] 3.1 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume) e atualizá-la se a change trouxe algo novo: componente de terceiros (crate, fonte, ícones) e sua licença, endereço do repositório (https://github.com/douglaspands/lume-tv-launcher) ou versão; sem novidade, a tarefa registra "Sobre revisado, sem mudança". Verificar com `cargo run -p lume-desktop -- --screenshot about`
  - Sobre revisado, sem mudança (nenhum componente de terceiros, licença, endereço ou versão novo).
