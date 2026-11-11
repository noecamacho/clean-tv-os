# Proposal

## Why

A tela Sobre e a tela de Configurações mostram a versão do `Cargo.toml`, que precisa ser editada à mão a cada release e não acompanha a tag: um release `v0.3.0` com o `Cargo.toml` esquecido em 0.2.1 mostraria 0.2.1 na TV, enquanto o Android registraria 0.3.0. E um APK gerado localmente não diz de que commit veio, o que atrapalha os testes na TV. Esta é a primeira das três changes do release v0.3.0 (depois `menu-de-contexto` e `splash`).

## What Changes

- A versão do build passa a ter uma fonte só, a variável `LUME_VERSION_NAME`, lida pelo Gradle e pelo Rust. Nenhum arquivo precisa ser editado para lançar uma versão: basta a tag.
- **Release (tag)**: a tag `v0.3.0` vira a versão `0.3.0` (todos os "v" do início são removidos, então `vv0.3.0` e `0.3.0` dão o mesmo resultado). O `versionCode` continua vindo dos números (300). O APK se chama `lume-0.3.0.apk` e o release do GitHub "Lume v0.3.0".
- **Build local** (`scripts/container-build.sh`, Gradle direto ou `cargo run` do desktop): a versão é o hash curto do commit (`4d8438b`), com o sufixo `-dirty` quando há mudança não commitada em arquivo rastreado. O `versionCode` é a contagem de commits (`git rev-list --count HEAD`).
- **Sem git** (ex.: zip do código-fonte): versão `dev`, `versionCode` 1.
- Na interface (tela Sobre e canto da tela de Configurações), uma versão de release aparece com "v" na frente (`v0.3.0`); um hash ou `dev` aparecem como estão.
- O log de abertura do app passa a escrever a mesma versão do build.
- **BREAKING (processo)**: o `Cargo.toml` do workspace fica fixo em `1.0.0`, a meta do projeto, com um comentário dizendo que não é a versão do build; a versão `0.2.1` fixa do `build.gradle` sai.
- O endereço do repositório (`https://github.com/douglaspands/lume-tv-launcher`) passa a ter fonte única, o campo `repository` do `Cargo.toml`, lido pela tela Sobre; os READMEs ganham o link da página de Releases e as notas de release o link do repositório.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `platform-compatibility`: o requisito de artefatos de distribuição passa a definir de onde vem a versão (tag normalizada no release; hash, `-dirty` e contagem de commits no build local; `dev` sem git).
- `launcher-shell`: a tela de Configurações do Lume e a tela Sobre mostram a versão do build (com "v" quando é release).

## Impact

- Build: `android/app/build.gradle` (normalização, fallback para o git, `versionCode`, repasse da variável ao `cargo ndk`), `scripts/container-build.sh` (calcula a versão no host, pois o `.git` não entra no container; nome dos arquivos), `.github/workflows/release.yml` (normalização da tag, título "Lume v…"), `Cargo.toml` (1.0.0 comentado) e `Cargo.lock`.
- Rust: `crates/lume-core/build.rs` (expõe a versão ao código), `crates/lume-core/src/app/settings.rs` (Sobre e canto das Configurações), `crates/lume-app/src/android/mod.rs` (log de abertura); testes da formatação.
- Docs: `README.md`, `README.pt-BR.md`, `CLAUDE.md` e `docs/handoff-dock-aurora.md` (nomes de APK e `LUME_VERSION_NAME`).
- Sem mudança de comportamento em tempo de execução além do texto da versão; sem custo de memória ou de quadros.
