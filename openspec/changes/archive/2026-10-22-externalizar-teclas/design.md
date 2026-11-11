# Design

## Context

Hoje o mapeamento de teclas do controle remoto está dividido em três lugares:
1. `crates/lume-core/src/app/mod.rs` define o `enum Key` (`Up`, `Down`, `Left`, `Right`, `Ok`, `Back`, `Home`, `Menu`).
2. `crates/lume-core/src/shell/keys.rs` define as constantes `KEYCODE_*` do Android e a função estática `from_android(code: i32) -> Option<Key>`, chamada diretamente por `crates/lume-app/src/android/mod.rs` em `Runtime::process_input`.
3. `tools/lume-desktop/src/demo.rs` mantém um `match` separado para converter strings (`"up"`, `"down"`, `"ok"`, etc.) em `Key`.

Não há persistência do mapeamento nos dados do app (`files/`) nem suporte para compilar o Lume com um mapeamento pré-configurado para uma TV específica.

## Goals / Non-Goals

**Goals:**
- Centralizar em `crates/lume-core/src/keys.rs` o `enum Key`, as constantes `KEYCODE_*`, os identificadores textuais (`"up"`, `"down"`, `"left"`, `"right"`, `"ok"`, `"back"`, `"home"`, `"menu"`), as listas padrão (`DEFAULT_*`) e a estrutura `KeyMap`.
- Persistir o mapeamento ativo em `files/keys.v1` usando `crate::storage` (`Kv`, versão `1`), criando o arquivo automaticamente no primeiro uso ou após limpar os dados do app.
- Garantir autocura automática (`load_or_heal`): se `files/keys.v1` estiver corrompido (queda de energia, truncamento) ou tiver alguma chave ausente/inválida, restaurar o padrão (total ou da chave afetada) e regravar imediatamente um `files/keys.v1` limpo em disco.
- Permitir desabilitar qualquer tecla definindo seu valor como vazio no arquivo (ex.: `menu=`).
- Permitir embutir um arquivo opcional de mapeamento em tempo de compilação (`LUME_KEYS_FILE`) via `crates/lume-core/build.rs` (com custo zero de parse em runtime), tornando-o o padrão de fábrica (`DEFAULT_*`) daquele binário.
- Manter performance máxima: 0 I/O no retorno a quente (`Shell::wake`), 0 escritas em disco quando `files/keys.v1` já existe e é válido, e busca `O(1)` em tabela inline `[Option<Key>; 320]` (320 bytes na stack) a cada tecla pressionada.
- Deixar a API de leitura, edição e restauração (`codes`, `set`, `clear`, `reset_key`, `reset`, `is_default`, `is_all_default` e `Effect::SaveKeys`) pronta em `LumeApp` e `Shell` para uma futura spec de edição de teclas pela interface.

**Non-Goals:**
- Não criar tela de interface gráfica (UI) para editar ou resetar teclas nesta change (ficará para uma spec futura).
- Não alterar nem adicionar código Java (`LumeActivity.java` e `LumeBridge.java` permanecem intocados).

---

## Decisions & Detailed Specification

### 1. Módulo Central `crates/lume-core/src/keys.rs`

Substituir `crates/lume-core/src/shell/keys.rs` pelo novo módulo `crates/lume-core/src/keys.rs`, exposto em `crates/lume-core/src/lib.rs` (`pub mod keys;`) e reexportado em:
- `crates/lume-core/src/app/mod.rs`: `pub use crate::keys::Key;`
- `crates/lume-core/src/shell/mod.rs`: `pub use crate::keys;` (para não quebrar chamadas existentes a `lume_core::shell::keys::*`).

#### 1.1. Enum `Key` e Metadados Canônicos

```rust
/// Remote control keys.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    /// OK, Enter or the center of the D-pad.
    Ok = 4,
    Back = 5,
    Home = 6,
    /// Opens the context menu of the focused item, where it has one.
    Menu = 7,
}
```

Como `Key` tem `#[repr(u8)]` com 8 variantes (`0..=7`), o compilador Rust aplica otimização de nicho e `std::mem::size_of::<Option<Key>>()` é exatamente **1 byte**.

Métodos obrigatórios em `Key`:
- `pub const COUNT: usize = 8;`
- `pub const ALL: [Key; Key::COUNT] = [Key::Up, Key::Down, Key::Left, Key::Right, Key::Ok, Key::Back, Key::Home, Key::Menu];`
- `pub const fn index(self) -> usize { self as usize }`
- `pub const fn as_str(self) -> &'static str`:
  - `Key::Up => "up"`
  - `Key::Down => "down"`
  - `Key::Left => "left"`
  - `Key::Right => "right"`
  - `Key::Ok => "ok"`
  - `Key::Back => "back"`
  - `Key::Home => "home"`
  - `Key::Menu => "menu"`
- `pub fn parse_name(name: &str) -> Option<Key>`: aceita os nomes canônicos (case-insensitive via `trim().to_ascii_lowercase()`) e o alias `"enter"` para `Key::Ok` (compatível com `tools/lume-desktop/src/demo.rs`).
- `pub fn default_codes(self) -> &'static [i32]`: retorna a constante `DEFAULT_*` correspondente à variante.

#### 1.2. Constantes `KEYCODE_*` e Padrões de Fábrica (`DEFAULT_*`)

Manter todas as constantes públicas em `crates/lume-core/src/keys.rs`:
```rust
pub const KEYCODE_HOME: i32 = 3;
pub const KEYCODE_BACK: i32 = 4;
pub const KEYCODE_DPAD_UP: i32 = 19;
pub const KEYCODE_DPAD_DOWN: i32 = 20;
pub const KEYCODE_DPAD_LEFT: i32 = 21;
pub const KEYCODE_DPAD_RIGHT: i32 = 22;
pub const KEYCODE_DPAD_CENTER: i32 = 23;
pub const KEYCODE_ENTER: i32 = 66;
pub const KEYCODE_MENU: i32 = 82;
pub const KEYCODE_BUTTON_A: i32 = 96;
pub const KEYCODE_BUTTON_B: i32 = 97;
pub const KEYCODE_ESCAPE: i32 = 111;
pub const KEYCODE_NUMPAD_ENTER: i32 = 160;
pub const KEYCODE_BOOKMARK: i32 = 174;
```

Os slices padrão de fábrica (`DEFAULT_UP`, `DEFAULT_DOWN`, `DEFAULT_LEFT`, `DEFAULT_RIGHT`, `DEFAULT_OK`, `DEFAULT_BACK`, `DEFAULT_HOME`, `DEFAULT_MENU`) vêm de `include!(concat!(env!("OUT_DIR"), "/keys_generated.rs"))`, gerado por `crates/lume-core/build.rs`. Quando nenhum arquivo é passado na compilação, `keys_generated.rs` emite exatamente:
- `DEFAULT_UP: &[i32] = &[KEYCODE_DPAD_UP];` (`&[19]`)
- `DEFAULT_DOWN: &[i32] = &[KEYCODE_DPAD_DOWN];` (`&[20]`)
- `DEFAULT_LEFT: &[i32] = &[KEYCODE_DPAD_LEFT];` (`&[21]`)
- `DEFAULT_RIGHT: &[i32] = &[KEYCODE_DPAD_RIGHT];` (`&[22]`)
- `DEFAULT_OK: &[i32] = &[KEYCODE_DPAD_CENTER, KEYCODE_ENTER, KEYCODE_NUMPAD_ENTER, KEYCODE_BUTTON_A];` (`&[23, 66, 160, 96]`)
- `DEFAULT_BACK: &[i32] = &[KEYCODE_BACK, KEYCODE_ESCAPE, KEYCODE_BUTTON_B];` (`&[4, 111, 97]`)
- `DEFAULT_HOME: &[i32] = &[KEYCODE_HOME];` (`&[3]`)
- `DEFAULT_MENU: &[i32] = &[KEYCODE_MENU, KEYCODE_BOOKMARK];` (`&[82, 174]`)

Manter também a função livre `pub fn from_android(code: i32) -> Option<Key>` em `keys.rs` delegando para as constantes padrão (ou para uma tabela constante), para compatibilidade com testes unitários existentes, enquanto o runtime usa `KeyMap::from_android`.

---

### 2. Estrutura `KeyMap` de Alta Performance (`O(1)`) e Preparada para Edição

```rust
const FAST_TABLE_SIZE: usize = 320;
const VERSION: u32 = 1;

#[derive(Clone, Debug)]
pub struct KeyMap {
    /// Active keycodes per logical Key, indexed by `key.index()` (0..8).
    bindings: [Vec<i32>; Key::COUNT],
    /// O(1) lookup table for Android keycodes in 0..320 (320 bytes inline on the stack).
    fast_table: [Option<Key>; FAST_TABLE_SIZE],
    /// Fallback list only for custom keycodes outside 0..320.
    overflow: Vec<(i32, Key)>,
}
```

Nota: `PartialEq` e `Eq` em `KeyMap` comparam `self.bindings == other.bindings` (já que `fast_table` e `overflow` são derivados deterministicamente de `bindings`).

#### 2.1. Reconstrução da Tabela `O(1)` (`rebuild_lookup`)
Sempre que `KeyMap` é criado ou modificado (`default`, `from_kv`, `set`, `clear`, `reset_key`, `reset`), o método privado `rebuild_lookup(&mut self)`:
1. Limpa `self.fast_table = [None; FAST_TABLE_SIZE]` e `self.overflow.clear()`.
2. Itera por `key in Key::ALL` e por `&code in &self.bindings[key.index()]`:
   - Se `0 <= code && (code as usize) < FAST_TABLE_SIZE`:
     - Se `self.fast_table[code as usize].is_none()`, define `self.fast_table[code as usize] = Some(key)`.
   - Caso contrário (`code < 0 || code as usize >= FAST_TABLE_SIZE`):
     - Se `!self.overflow.iter().any(|&(c, _)| c == code)`, adiciona `(code, key)` em `self.overflow`.

#### 2.2. Resolução em Tempo de Execução (`KeyMap::from_android`)
```rust
#[inline]
pub fn from_android(&self, code: i32) -> Option<Key> {
    if code >= 0 && (code as usize) < FAST_TABLE_SIZE {
        return self.fast_table[code as usize];
    }
    self.overflow
        .iter()
        .find_map(|&(c, key)| (c == code).then_some(key))
}
```
- **Custo**: Para qualquer código Android normal (`0..319`), é 1 única instrução de indexação em array de 320 bytes na stack, sem alocação no heap e sem hashing.

#### 2.3. API Pronta para a Futura Spec de Edição de Teclas
Implementar e testar desde já em `KeyMap`:
- `pub fn codes(&self, key: Key) -> &[i32]`: retorna os códigos ativos de `key`.
- `pub fn set(&mut self, key: Key, codes: Vec<i32>)`:
  - Remove códigos duplicados mantendo a ordem.
  - Também remove qualquer código de `codes` que estivesse associado a outra `Key` em `self.bindings`, evitando conflito onde um mesmo botão físico dispararia duas ações.
  - Atualiza `self.bindings[key.index()] = codes` e chama `self.rebuild_lookup()`.
- `pub fn clear(&mut self, key: Key)`: atalho para `self.set(key, Vec::new())` (desabilita a tecla).
- `pub fn reset_key(&mut self, key: Key)`: restaura `key` para `key.default_codes().to_vec()` (removendo conflitos e chamando `rebuild_lookup()`).
- `pub fn reset(&mut self)`: restaura todas as teclas (`*self = KeyMap::default()`).
- `pub fn is_default(&self, key: Key) -> bool`: retorna `self.codes(key) == key.default_codes()`.
- `pub fn is_all_default(&self) -> bool`: retorna `Key::ALL.into_iter().all(|k| self.is_default(k))`.

---

### 3. Persistência e Autocura em `files/keys.v1`

O arquivo `keys.v1` usa `crate::storage::{self, Kv, StorageError}` com `VERSION = 1`:

```ini
lume=1
up=19
down=20
left=21
right=22
ok=23,66,160,96
back=4,111,97
home=3
menu=82,174
#end
```

Cada tecla lógica pode mapear múltiplos códigos físicos enviados pelo controle remoto separados por vírgula (como em `ok=23,66,160,96` e `menu=82,174`, ou com botões adicionais customizados pelo usuário como `menu=82,174,165`). Todos os códigos mapeados são indexados na tabela rápida `fast_table` (ou `overflow`), de modo que pressionar qualquer um dos botões físicos no controle remoto dispara a mesma tecla lógica no Lume em tempo `O(1)`.

#### 3.1. Serialização (`to_kv` e `save`)
```rust
fn to_kv(&self) -> Kv {
    let mut kv = Kv::with_capacity(Key::COUNT);
    for key in Key::ALL {
        let codes: Vec<String> = self.codes(key).iter().map(i32::to_string).collect();
        kv.set_list(key.as_str(), codes.iter().map(String::as_str));
    }
    kv
}

pub fn save(&self, path: &Path) -> std::io::Result<()> {
    storage::save(path, VERSION, &self.to_kv())
}
```

#### 3.2. Leitura com Sobrescrita Granular e Detecção de Reparo (`from_kv`)
`from_kv(kv: &Kv) -> (KeyMap, bool)` retorna o `KeyMap` carregado e uma flag `needs_save: bool` indicando se alguma chave estava ausente ou continha valor inválido e precisou ser reparada:
1. Começa com `let mut map = KeyMap::default();` e `let mut needs_save = false;`.
2. Para cada `key in Key::ALL`:
   - Consulta `kv.get(key.as_str())`:
     - **`None` (chave ausente no arquivo)**:
       - Mantém o `key.default_codes()` já presente em `map`.
       - Define `needs_save = true` (para que a chave ausente seja escrita no arquivo curado).
     - **`Some("")` (chave presente com valor vazio, ex.: `menu=`)**:
       - Define `map.bindings[key.index()] = Vec::new()` (tecla **desabilitada** intencionalmente).
       - `needs_save` permanece `false`.
     - **`Some(raw)` (chave presente com valores separados por vírgula)**:
       - Faz o parse de cada item (`raw.split(',')`) como `item.trim().parse::<i32>()`.
       - Se **todos** os itens forem inteiros `i32` válidos:
         - Deduplica mantendo a ordem e guarda em `map.bindings[key.index()]`.
       - Se **qualquer** item falhar no parse (ex.: `ok=23,abc` ou `ok=,`):
         - Registra `log::warn!("ignoring invalid keycodes for {}: {:?}", key.as_str(), raw)`.
         - Mantém `key.default_codes()` para aquela tecla e define `needs_save = true`.
3. Chama `map.rebuild_lookup()` e retorna `(map, needs_save)`.

#### 3.3. Autocura Completa (`load_or_heal`)
```rust
/// Loads `keys.v1` from `path`. If the file is missing, corrupt, or had missing/invalid entries,
/// repairs it in memory and atomically writes the clean `keys.v1` to disk.
pub fn load_or_heal(path: &Path) -> KeyMap {
    match storage::load(path, VERSION) {
        Ok(kv) => {
            let (map, needs_save) = KeyMap::from_kv(&kv);
            if needs_save {
                if let Err(e) = map.save(path) {
                    log::warn!("cannot write repaired {}: {e}", path.display());
                }
            }
            map
        }
        Err(StorageError::Missing) => {
            let map = KeyMap::default();
            if let Err(e) = map.save(path) {
                log::warn!("cannot write initial {}: {e}", path.display());
            }
            map
        }
        Err(e) => {
            log::warn!("healing {}: {e}", path.display());
            let map = KeyMap::default();
            if let Err(err) = map.save(path) {
                log::warn!("cannot write healed {}: {err}", path.display());
            }
            map
        }
    }
}
```
Comportamento garantido:
- **Boot normal (`keys.v1` existe e todas as 8 chaves são válidas)**: `needs_save == false` → **zero escrita em disco**.
- **Primeiro boot ou após "Limpar dados" no Android (`StorageError::Missing`)**: cria `KeyMap::default()` (com as constantes da compilação) e grava `files/keys.v1`.
- **Queda de energia / arquivo corrompido (`StorageError::Corrupt` / `UnknownVersion`)**: loga aviso, restaura `KeyMap::default()` e sobrescreve `files/keys.v1` atomicamente com um arquivo limpo.
- **Corrupção parcial em uma linha (`ok=23,xyz`) ou chave faltando**: restaura apenas a tecla afetada para `DEFAULT_*`, mantém as demais customizações do usuário e regrava `files/keys.v1` limpo.

---

### 4. Integração no `Shell`, `LumeApp` e `lume-app` (Android)

1. **`crates/lume-core/src/app/mod.rs`**:
   - Adicionar `SaveKeys` ao `enum Effect`:
     ```rust
     /// The key mapping changed and must be written to disk.
     SaveKeys,
     ```
   - Adicionar `pub keys: crate::keys::KeyMap` na struct `LumeApp` (inicializado como `KeyMap::default()` em `LumeApp::new`, ou ajustado pelo `Shell::start`).
2. **`crates/lume-core/src/shell/mod.rs`**:
   - Adicionar `const KEYS_FILE: &str = "keys.v1";` ao lado de `CONFIG_FILE`, `CATALOG_FILE`, `USAGE_FILE`.
   - Em `Shell::start`:
     ```rust
     let keys = crate::keys::KeyMap::load_or_heal(&files.join(KEYS_FILE));
     ```
     e atribuir `app.keys = keys;`.
   - Em `Shell::finish`:
     ```rust
     Effect::SaveKeys => save_log(
         self.app.keys.save(&self.files.join(KEYS_FILE)),
         KEYS_FILE,
     ),
     ```
3. **`crates/lume-app/src/android/mod.rs`**:
   - Em `Runtime::process_input` (linha ~394): substituir `keys::from_android(code as i32)` por `self.shell.app().keys.from_android(code as i32)`.

---

### 5. Perfil de Compilação Opcional (`LUME_KEYS_FILE` em `build.rs`)

Em `crates/lume-core/build.rs`:
1. Adicionar `compile_keys(&manifest);` em `main()`.
2. Em `compile_keys`:
   - Emitir `println!("cargo:rerun-if-env-changed=LUME_KEYS_FILE");`.
   - Se `std::env::var("LUME_KEYS_FILE")` não estiver definida ou estiver vazia:
     - Gerar `$OUT_DIR/keys_generated.rs` com os valores padrão de fábrica:
       - `up`: `&[19]`
       - `down`: `&[20]`
       - `left`: `&[21]`
       - `right`: `&[22]`
       - `ok`: `&[23, 66, 160, 96]`
       - `back`: `&[4, 111, 97]`
       - `home`: `&[3]`
       - `menu`: `&[82, 174]`
   - Se `LUME_KEYS_FILE` estiver definida com um caminho:
     - Resolver o caminho (se relativo, resolver em relação à raiz do workspace `manifest.join("../..")` ou ao diretório atual) e emitir `println!("cargo:rerun-if-changed={}", path.display());`.
     - Ler o arquivo em tempo de compilação e validar o formato `lume=1 ... #end`:
       - Primeira linha deve ser `lume=1` e última linha `#end` (ignorando linhas vazias e comentários `#`).
       - Cada linha `chave=v1,v2,...` deve usar uma das 8 chaves válidas (`up`, `down`, `left`, `right`, `ok`, `back`, `home`, `menu`) e conter inteiros `i32` válidos (ou vazio `chave=` para desabilitar a tecla por padrão).
       - Qualquer erro de leitura, chave desconhecida ou número inválido deve encerrar o build com `panic!("invalid LUME_KEYS_FILE {}: ...", path.display())`.
       - Chaves omitidas no arquivo passado usam os valores padrão de fábrica daquela tecla.
     - Gerar `$OUT_DIR/keys_generated.rs` com as constantes `pub const DEFAULT_UP: &[i32] = &[...];` ... `pub const DEFAULT_MENU: &[i32] = &[...];`.
3. Em `scripts/container-build.sh` e `docker/Dockerfile`:
   - Se `LUME_KEYS_FILE` estiver definido no ambiente ao chamar `scripts/container-build.sh`, copiar/passar o arquivo para o contexto de build (ou passar o caminho relativo ao repositório via `--build-arg LUME_KEYS_FILE=...`) para que o build em container também suporte a variável.

## Risks / Trade-offs

- **[Queda de energia durante a gravação de `keys.v1`]** → Mitigado por duas camadas: escrita atômica via arquivo temporário + `fsync` + `rename` em `storage::save`, e autocura em `KeyMap::load_or_heal` que detecta qualquer truncamento e recria `keys.v1` no boot seguinte.
- **[Impacto no tempo de abertura a frio]** → O arquivo tem ~90 bytes. Em aberturas normais é apenas 1 leitura sequencial sem escrita (`needs_save == false`), custando < 0,2 ms. No retorno a quente (`Shell::wake`), o `KeyMap` permanece na RAM (0 I/O).
