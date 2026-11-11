# Tasks

## 1. Geração das Constantes de Teclas no Build (`build.rs` e Container)

- [x] 1.1 Em `crates/lume-core/build.rs`, adicionar a função `compile_keys(manifest: &std::path::Path)` chamada a partir de `main()`, que monitora `cargo:rerun-if-env-changed=LUME_KEYS_FILE` e gera `$OUT_DIR/keys_generated.rs` contendo as 8 constantes `pub const DEFAULT_UP: &[i32]`, `DEFAULT_DOWN`, `DEFAULT_LEFT`, `DEFAULT_RIGHT`, `DEFAULT_OK`, `DEFAULT_BACK`, `DEFAULT_HOME` e `DEFAULT_MENU`. Quando `LUME_KEYS_FILE` não estiver definida ou estiver vazia, emitir os códigos padrão de fábrica (`&[19]`, `&[20]`, `&[21]`, `&[22]`, `&[23, 66, 160, 96]`, `&[4, 111, 97]`, `&[3]`, `&[82, 174]`). Quando `LUME_KEYS_FILE` apontar para um arquivo, emitir `cargo:rerun-if-changed=<path>`, validar o cabeçalho `lume=1`, o marcador final `#end`, as chaves permitidas (`up`, `down`, `left`, `right`, `ok`, `back`, `home`, `menu`) e os valores `i32` (aceitando vazio `chave=` como `&[]`), falhando o build com `panic!` em caso de erro de sintaxe
- [x] 1.2 Em `scripts/container-build.sh` e `docker/Dockerfile`, repassar opcionalmente `LUME_KEYS_FILE` (via `--secret id=lume_keys_file` ou `--build-arg`, nos mesmos moldes de `LUME_KEYSTORE` em `scripts/container-build.sh`) para que o build dentro do container utilize o arquivo quando a variável estiver definida no host

## 2. Módulo Central `crates/lume-core/src/keys.rs` e `KeyMap`

- [x] 2.1 Criar `crates/lume-core/src/keys.rs` (e registrá-lo com `pub mod keys;` em `crates/lume-core/src/lib.rs`), movendo para ele:
  - O `enum Key` (`#[repr(u8)]`: `Up = 0`, `Down = 1`, `Left = 2`, `Right = 3`, `Ok = 4`, `Back = 5`, `Home = 6`, `Menu = 7`) com `Key::COUNT = 8`, `Key::ALL`, `Key::index(self) -> usize`, `Key::as_str(self) -> &'static str`, `Key::parse_name(name: &str) -> Option<Key>` (incluindo `"enter"` -> `Key::Ok`) e `Key::default_codes(self) -> &'static [i32]`
  - Todas as constantes `KEYCODE_*` (`KEYCODE_HOME`, `KEYCODE_BACK`, `KEYCODE_DPAD_UP`, `KEYCODE_DPAD_DOWN`, `KEYCODE_DPAD_LEFT`, `KEYCODE_DPAD_RIGHT`, `KEYCODE_DPAD_CENTER`, `KEYCODE_ENTER`, `KEYCODE_MENU`, `KEYCODE_BUTTON_A`, `KEYCODE_BUTTON_B`, `KEYCODE_ESCAPE`, `KEYCODE_NUMPAD_ENTER`, `KEYCODE_BOOKMARK`) e o `include!(concat!(env!("OUT_DIR"), "/keys_generated.rs"))`
  - A função livre `pub fn from_android(code: i32) -> Option<Key>` usando o mapeamento padrão
- [x] 2.2 Implementar a struct `KeyMap` em `crates/lume-core/src/keys.rs` com:
  - Campos `bindings: [Vec<i32>; Key::COUNT]`, `fast_table: [Option<Key>; 320]` (tabela `O(1)` inline de 320 bytes na stack) e `overflow: Vec<(i32, Key)>`
  - `Default` inicializado a partir de `Key::default_codes` e sincronizado via `rebuild_lookup(&mut self)`
  - `pub fn from_android(&self, code: i32) -> Option<Key>` consultando `fast_table` em `O(1)` para `0..320` e `overflow` para códigos fora dessa faixa
  - Métodos prontos para a futura tela de edição: `codes(&self, key: Key) -> &[i32]`, `set(&mut self, key: Key, codes: Vec<i32>)` (deduplicando e removendo conflitos de outras teclas), `clear(&mut self, key: Key)`, `reset_key(&mut self, key: Key)`, `reset(&mut self)`, `is_default(&self, key: Key) -> bool` e `is_all_default(&self) -> bool`
- [x] 2.3 Implementar a persistência e autocura em `KeyMap` (`crates/lume-core/src/keys.rs`) usando `crate::storage`:
  - `pub fn save(&self, path: &Path) -> std::io::Result<()>` gravando `VERSION = 1` com as 8 chaves de `Key::ALL`
  - `pub fn load_or_heal(path: &Path) -> KeyMap`:
    - Se o arquivo não existir (`StorageError::Missing`), cria `KeyMap::default()`, grava em `path` e retorna
    - Se o arquivo estiver corrompido (`StorageError::Corrupt` / `UnknownVersion` / `Io`), registra `log::warn!`, cria `KeyMap::default()`, sobrescreve `path` com o arquivo íntegro e retorna
    - Se o arquivo abrir (`Ok(kv)`), lê cada tecla de `Key::ALL`: valor vazio (`menu=`) define `vec![]` (tecla desabilitada sem marcar reparo); chave ausente ou com token não numérico restaura `key.default_codes()` apenas para aquela tecla e marca `needs_save = true`; se `needs_save` for verdadeiro, regrava `path` com o arquivo corrigido
- [x] 2.4 Remover o arquivo antigo `crates/lume-core/src/shell/keys.rs`, substituir `pub mod keys;` por `pub use crate::keys;` em `crates/lume-core/src/shell/mod.rs`, substituir a definição de `enum Key` em `crates/lume-core/src/app/mod.rs` por `pub use crate::keys::Key;`, e atualizar `parse_keys` em `tools/lume-desktop/src/demo.rs` para usar `Key::parse_name`

## 3. Integração com `LumeApp`, `Shell` e Android (`lume-app`)

- [x] 3.1 Em `crates/lume-core/src/app/mod.rs`, adicionar a variante `Effect::SaveKeys` ao `enum Effect` e o campo `pub keys: crate::keys::KeyMap` na struct `LumeApp` (inicializado com `KeyMap::default()` em `LumeApp::new`)
- [x] 3.2 Em `crates/lume-core/src/shell/mod.rs`, adicionar `const KEYS_FILE: &str = "keys.v1";`, carregar o mapeamento em `Shell::start` com `self.app.keys = crate::keys::KeyMap::load_or_heal(&files.join(KEYS_FILE));`, e tratar `Effect::SaveKeys` em `Shell::finish` chamando `save_log(self.app.keys.save(&self.files.join(KEYS_FILE)), KEYS_FILE)`
- [x] 3.3 Em `crates/lume-app/src/android/mod.rs`, atualizar `Runtime::process_input` (linha ~394) para mapear o código de tecla usando `self.shell.app().keys.from_android(code as i32)` em vez de `keys::from_android(code as i32)`

## 4. Testes Automatizados

- [x] 4.1 Adicionar testes unitários em `crates/lume-core/src/keys.rs` cobrindo:
  - Tamanho de `Option<Key>` igual a 1 byte (`assert_eq!(std::mem::size_of::<Option<Key>>(), 1)`)
  - Mapeamento padrão de todas as teclas (`from_android` e `KeyMap::default().from_android`), incluindo códigos na faixa `0..320` e códigos de overflow (`> 320`)
  - Criação automática de `keys.v1` quando o arquivo não existe (`load_or_heal`)
  - Sobrescrita de teclas ao carregar `keys.v1` existente sem regravar o arquivo quando válido (inclusive múltiplos códigos para a mesma tecla, ex.: `menu=82,174,165`)
  - Desabilitação de tecla com valor vazio (`menu=`) retornando `None` para `82` e `174`
  - Autocura de arquivo truncado/corrompido (sem `#end` ou cabeçalho inválido), verificando que `keys.v1` em disco passa a ser válido imediatamente após `load_or_heal`
  - Autocura parcial quando uma linha tem token inválido (`ok=23,abc`) ou quando uma chave está ausente, verificando que as outras teclas customizadas são preservadas e o arquivo em disco é regravado corrigido
  - Métodos de edição e restauração (`set`, `clear`, `reset_key`, `reset`, `is_default`, `is_all_default` e remoção de conflitos entre teclas)
  - Verificar com `cargo test -p lume-core keys`
- [x] 4.2 Adicionar testes de integração em `crates/lume-core/src/shell/mod.rs` cobrindo:
  - `Shell::start` cria `files/keys.v1` no primeiro boot e recria após remoção do arquivo (simulando "Limpar dados" do Android)
  - `Shell::start` respeita `files/keys.v1` customizado e repara `files/keys.v1` corrompido
  - `Effect::SaveKeys` persiste alterações de `app.keys` em `files/keys.v1`
  - Verificar com `cargo test -p lume-core shell`

## 5. Validação Geral e Tela Sobre

- [x] 5.1 Executar `cargo test --workspace && cargo clippy --workspace --all-targets` e garantir saída limpa sem avisos
- [x] 5.2 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume) com `cargo run -p lume-desktop -- --screenshot about` e atualizar se a change trouxe componente de terceiros, licença, endereço do repositório (`https://github.com/douglaspands/lume-tv-launcher`) ou versão; sem novidade, registrar "Sobre revisado, sem mudança"
