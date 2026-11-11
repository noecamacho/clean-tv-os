# Proposal

## Why

Atualmente, os códigos de teclas do Android (`KEYCODE_*`) estão acoplados a um `match` estático em `crates/lume-core/src/shell/keys.rs`, enquanto o `enum Key` vive em `crates/lume-core/src/app/mod.rs` e o parse textual das teclas (`"up"`, `"down"`, `"menu"`, etc.) está duplicado em `tools/lume-desktop/src/demo.rs`. Não há persistência do mapeamento nos dados do aplicativo nem suporte para compilar o Lume com um perfil de controle remoto pré-configurado para uma TV específica.

Centralizar todas as definições de teclas em um módulo único (`crates/lume-core/src/keys.rs`), persistir o mapeamento ativo em `files/keys.v1` com autocura automática contra corrupção e permitir embutir um arquivo opcional de mapeamento em tempo de compilação (`LUME_KEYS_FILE`) prepara toda a base — com custo zero no caminho crítico — para uma futura especificação de edição e restauração de teclas pela interface do usuário.

## What Changes

- **Módulo Centralizado de Teclas (`crates/lume-core/src/keys.rs`)**:
  - Reúne o `enum Key` (`#[repr(u8)]`: `Up`, `Down`, `Left`, `Right`, `Ok`, `Back`, `Home`, `Menu`), a constante `Key::ALL`, os identificadores textuais canônicos (`as_str` / `from_str`), todas as constantes `KEYCODE_*` do Android, as listas de códigos padrão de fábrica (`DEFAULT_*`) e a estrutura `KeyMap`.
  - Mantém `pub use crate::keys::Key;` em `crates/lume-core/src/app/mod.rs` e `pub use crate::keys;` em `crates/lume-core/src/shell/mod.rs` para preservar compatibilidade com o código existente.
- **Persistência em `files/keys.v1` nos Dados do App**:
  - Na inicialização (`Shell::start`), verifica se `files/keys.v1` existe no armazenamento interno do app (`storage::Kv`, versão `1`).
  - Se não existir (primeira execução ou após "Limpar dados" no Android), cria `KeyMap::default()` a partir das constantes embutidas na compilação e grava `files/keys.v1` em disco.
  - Se existir e estiver íntegro, lê os mapeamentos uma única vez na abertura a frio (sem regravar em disco) e sobrescreve as constantes em memória.
  - Cada tecla lógica aceita múltiplos códigos físicos do controle remoto separados por vírgula (ex.: `menu=82,174,165` permite que os botões Menu, Bookmark e um botão extra acionem a mesma ação Menu). Uma chave com valor vazio no arquivo (ex.: `menu=`) mapeia para uma lista vazia `[]`, desabilitando aquela tecla; uma chave omitida no arquivo mantém o padrão da compilação para aquela tecla.
- **Autocura contra Corrupção e Queda de Energia**:
  - Escritas utilizam `storage::save` (arquivo `.tmp` + `sync_all` + `rename` atômico).
  - Se na leitura `files/keys.v1` estiver truncado/corrompido (ex.: queda de energia), restaura `KeyMap::default()` e regrava imediatamente um `files/keys.v1` íntegro em disco.
  - Se `files/keys.v1` abrir mas tiver valor inválido em alguma tecla (ex.: `ok=23,abc`) ou alguma das 8 teclas estiver ausente, restaura apenas a tecla inválida/ausente para o seu `DEFAULT_*`, preserva as demais teclas válidas e regrava o `files/keys.v1` corrigido em disco.
- **Perfil de Compilação Opcional (`LUME_KEYS_FILE`)**:
  - `crates/lume-core/build.rs` e `scripts/container-build.sh` passam a aceitar opcionalmente a variável de ambiente `LUME_KEYS_FILE` apontando para um arquivo no mesmo formato `lume=1 ... #end`.
  - Quando não informada (padrão), o build usa as constantes padrão do Lume. Quando informada, o `build.rs` valida o arquivo em tempo de compilação e gera as constantes `DEFAULT_*` embutidas no binário, de modo que limpar os dados do app no Android ou acionar o futuro reset de fábrica restaure o perfil embutido naquela compilação.
- **Performance e Preparação para Edição Futura na UI**:
  - Tradução de keycode Android (`KeyMap::from_android`) em tempo `O(1)` usando tabela inline `[Option<Key>; 320]` (320 bytes na stack, zero alocação no heap por tecla pressionada) e 0 leituras de disco no retorno a quente (`Shell::wake`).
  - `LumeApp` passa a expor `pub keys: KeyMap`, `Effect` ganha a variante `Effect::SaveKeys` tratada em `Shell::finish`, e `KeyMap` já fornece a API completa (`codes`, `set`, `clear`, `reset_key`, `reset`, `is_default`, `is_all_default`) para a futura tela de edição de teclas.

## Capabilities

### Modified Capabilities
- `launcher-shell`: Especifica a centralização do mapeamento de teclas do controle remoto, a persistência e autocura de `files/keys.v1`, a possibilidade de desabilitar teclas com lista vazia, o suporte ao perfil opcional embutido em tempo de compilação (`LUME_KEYS_FILE`) e a recuperação do padrão da compilação ao limpar os dados do app.

## Impact

- **Código Rust (`lume-core`)**: `crates/lume-core/src/keys.rs` (novo módulo central, substituindo `crates/lume-core/src/shell/keys.rs`), `crates/lume-core/src/lib.rs`, `crates/lume-core/src/app/mod.rs`, `crates/lume-core/src/shell/mod.rs`, `crates/lume-core/build.rs`.
- **Código Android (`lume-app`)**: `crates/lume-app/src/android/mod.rs` (`process_input` consulta `self.shell.app().keys.from_android(code as i32)`).
- **Ferramentas Desktop (`lume-desktop`)**: `tools/lume-desktop/src/demo.rs` reutiliza `Key::from_str`.
- **Scripts e Container de Build**: `scripts/container-build.sh` e `docker/Dockerfile` (suporte opcional a `LUME_KEYS_FILE`).
- **Orçamentos (`resource-efficiency`)**: Zero novas dependências (`Cargo.toml` inalterado), ~320 bytes de RAM na tabela `O(1)`, zero escritas em disco em aberturas normais e zero leituras de disco no retorno a quente.
