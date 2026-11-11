## MODIFIED Requirements

### Requirement: Artefatos de distribuição
Cada release SHALL produzir um APK assinado (para o GitHub) e um AAB (para a Play Store), assinados com a mesma chave de app e com o mesmo identificador de pacote, de modo que um APK do GitHub possa ser atualizado por uma versão mais nova de qualquer um dos canais.

#### Scenario: Build local em container
- **WHEN** alguém com apenas um runtime de containers (wslc ou Docker) roda `scripts/container-build.sh release` num clone do repositório
- **THEN** o APK e o AAB são gerados em `dist/`, com as mesmas versões de ferramentas usadas pelo CI

#### Scenario: Release por tag
- **WHEN** uma tag `vX.Y.Z` é publicada no repositório
- **THEN** o CI gera o APK e o AAB assinados, com `versionName` X.Y.Z, anexa ao release do GitHub o APK chamado `lume-X.Y.Z.apk` e dá ao release o título "Lume vX.Y.Z"

#### Scenario: Atualização entre canais
- **WHEN** o lume 0.1.0 do GitHub está instalado e o usuário instala a 0.2.0 vinda do mesmo processo de assinatura
- **THEN** a atualização é aceita, e a configuração do usuário é mantida

## ADDED Requirements

### Requirement: Versão do build
A versão de cada build SHALL vir do próprio build, sem editar arquivos do repositório, e SHALL ser a mesma no Android (`versionName`), na interface e no log do app.

- Num build de release, a versão SHALL ser a tag sem os caracteres "v" do início (`v0.3.0`, `vv0.3.0` e `0.3.0` dão `0.3.0`), e o `versionCode` SHALL ser calculado a partir dos números X.Y.Z (X·10000 + Y·100 + Z), ignorando um sufixo de pré-release.
- Num build local, a versão SHALL ser o hash curto do commit atual. Se algum arquivo rastreado pelo git tiver mudança não commitada, a versão SHALL terminar em `-dirty`. O `versionCode` SHALL ser o número de commits do histórico do commit atual.
- Sem informação de git disponível, a versão SHALL ser `dev` e o `versionCode` SHALL ser 1.
- A versão do pacote Rust (`Cargo.toml`) SHALL NOT influenciar a versão do build.

#### Scenario: Tag com "v"
- **WHEN** o CI gera o release a partir da tag `v0.3.0`
- **THEN** o `versionName` é `0.3.0` e o `versionCode` é 300

#### Scenario: Tag com "v" repetido
- **WHEN** a versão informada ao build é `vv0.3.1`
- **THEN** o `versionName` é `0.3.1`

#### Scenario: Build local limpo
- **WHEN** alguém gera o APK localmente no commit `4d8438b`, sem mudanças não commitadas, num histórico de 6 commits
- **THEN** o `versionName` é `4d8438b`, o `versionCode` é 6 e o arquivo em `dist/` leva `4d8438b` no nome

#### Scenario: Build local com mudanças
- **WHEN** alguém gera o APK localmente no commit `4d8438b` com um arquivo rastreado modificado e não commitado
- **THEN** o `versionName` é `4d8438b-dirty`

#### Scenario: Sem git
- **WHEN** o código é compilado fora de um repositório git e sem versão informada
- **THEN** o `versionName` é `dev` e o `versionCode` é 1
