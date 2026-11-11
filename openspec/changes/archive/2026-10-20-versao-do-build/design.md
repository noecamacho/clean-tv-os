# Design

## Context

- O Gradle já lê `LUME_VERSION_NAME` (com fallback fixo "0.2.1") para `versionName` e calcula o `versionCode` com `split("-")`. O `release.yml` preenche a variável com `${GITHUB_REF_NAME#v}` (tira um "v" só).
- O Rust usa `env!("CARGO_PKG_VERSION")` em `app/settings.rs` (Sobre e canto das Configurações) e no log de abertura (`lume-app/src/android/mod.rs`).
- O `.dockerignore` exclui `.git/`: dentro do container não há git. O `container-build.sh` já repassa `LUME_VERSION_NAME` como `--build-arg`, e o Dockerfile a exporta como `ENV`.
- O Gradle chama o `cargo ndk` por uma tarefa própria (`CargoNdkTask`), que herda o ambiente do processo do Gradle.
- `crates/lume-core/build.rs` já existe (tema, traduções, papel de parede, licenças) e é relativamente caro (roda `cargo tree`).
- Texto traduzido: `about_version` = "Versão %1$s" / "Version %1$s".

## Goals / Non-Goals

**Goals:** uma regra de versão, aplicada igual nos três caminhos de build (CI, container, cargo direto), sem arquivo para editar a cada release.

**Non-Goals:** mudar o esquema do `versionCode` de release; assinar builds locais; ler a versão em tempo de execução pelo `PackageManager`.

## Decisions

### 1. Quem calcula a versão

```
                    LUME_VERSION_NAME já definida?
                     |sim                    |não
                     v                       v
             normaliza (tira ^v+)      git disponível?
                     |                  |sim            |não
                     |           hash curto [+ -dirty]   "dev"
                     v                  v               v
                  versão final  <-------+---------------+
```

- **CI de release**: o `release.yml` passa a tag crua; a normalização acontece no build.
- **Container**: o `container-build.sh` roda a regra no host (onde há git) e passa o resultado em `LUME_VERSION_NAME`; também passa `LUME_VERSION_CODE` (contagem de commits), porque o container não consegue contá-los.
- **Gradle direto** (máquina com SDK, CI de debug): a mesma regra em Groovy, com `git` via `providers.exec`, quando a variável não vier.
- **cargo direto** (`lume-desktop`, testes): a mesma regra num `build.rs`.

O Gradle passa explicitamente `LUME_VERSION_NAME` (já normalizada) ao `cargo ndk` no `environment` da tarefa, para que o Rust do APK nunca recalcule por conta própria nem divirja do `versionName`.

Alternativa descartada: ler `PackageInfo.versionName` por JNI em tempo de execução. Funcionaria só no Android (o desktop continuaria precisando de outra fonte) e acrescentaria um método JNI com `-keep`.

### 2. Um crate pequeno para a versão (`lume-version`)

A regra em Rust fica num crate novo do workspace, com um `build.rs` só para isso, e a `lume-core` depende dele. Motivo: detectar `-dirty` exige reexecutar o script a cada build (uma edição não staged não muda nada que o Cargo observe), e isso não pode arrastar o `build.rs` caro da `lume-core`. O `build.rs` do crate emite `rerun-if-env-changed=LUME_VERSION_NAME` e, quando cai no git, `rerun-if-changed` para `.git/HEAD`, a ref atual e `.git/index`. Edições não staged entre dois `cargo run` do desktop podem não atualizar o `-dirty`; isso só afeta o desktop, porque os builds de APK sempre recebem a versão pronta do script ou do Gradle.

O crate expõe `VERSION` (a versão normalizada) e `display()` (com "v" quando é release). Testes unitários cobrem a normalização e a exibição com strings fixas.

### 3. Quando mostrar "v"

`v` só quando a versão começa com `X.Y.Z` (dígitos e pontos), o que exclui o hash (que pode começar com dígito, como `4d8438b`, mas não tem `.` logo depois) e `dev`. A frase continua traduzida: "Versão v0.3.0" / "Version v0.3.0"; "Versão 4d8438b-dirty". Não muda `strings.xml`.

### 4. `-dirty` só para arquivos rastreados

`git status --porcelain --untracked-files=no` não vazio. Arquivos não rastreados (saídas de build, rascunhos) não sujam a versão; um arquivo novo de código normalmente vem junto com edições rastreadas (o `mod` que o declara).

### 5. `versionCode`

- Release: o cálculo atual sobre X.Y.Z (0.3.0 → 300).
- Local: `git rev-list --count HEAD`; sem git, 1. Um build local quase sempre terá código menor que o de um release instalado, o que não atrapalha: a instalação local já exige `adb uninstall` (chave de debug diferente).
- Regra para decidir qual usar: se a versão normalizada é `X.Y.Z[-pré]`, cálculo de release; senão, contagem de commits (`LUME_VERSION_CODE` quando vier do script).

### 6. `Cargo.toml` em 1.0.0

Fica como a meta do projeto, com comentário. Como nenhum código lê mais `CARGO_PKG_VERSION`, ele não aparece em lugar nenhum para o usuário. Os dois usos atuais passam a ler do crate de versão.

## Risks / Trade-offs

- [Três implementações da mesma regra (bash, Groovy, Rust)] → regra curta (tirar `^v+`, hash, `-dirty`, `dev`), com os mesmos casos documentados nos cenários da spec; testes no Rust e verificação manual nos dois scripts.
- [Shallow clone no CI de debug: contagem de commits = 1] → aceitável: o APK de debug do CI não é publicado.
- [Release a partir de árvore suja] → impossível no CI (checkout limpo da tag); localmente a versão já denuncia com `-dirty`.

## Migration Plan

Nada a migrar no aparelho. Para lançar: commitar, criar a tag `v0.3.0` e empurrar. O `Cargo.toml` não é mais tocado em release.
