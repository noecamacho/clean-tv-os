# Spec Delta

## Purpose

Define em quais dispositivos o lume roda, quais permissões ele pede e quais requisitos de distribuição atende para o GitHub e a Play Store.

## ADDED Requirements

### Requirement: Versões do Android suportadas
O lume SHALL instalar e funcionar em Android TV 9 (API 28) a Android TV 11 (API 30), e SHALL ser compilado contra um nível de API alvo aceito pela Play Store no momento do release.

#### Scenario: Android TV 9
- **WHEN** o APK é instalado na TCL 55C715 (Android TV 9)
- **THEN** a instalação é bem-sucedida, e todas as telas funcionam

#### Scenario: Android TV 11
- **WHEN** o APK é instalado em um dispositivo ou emulador Android TV 11
- **THEN** a instalação é bem-sucedida, e o catálogo lista os apps instalados

#### Scenario: Versão anterior
- **WHEN** alguém tenta instalar o APK em Android 8.1 ou anterior
- **THEN** o sistema recusa a instalação

### Requirement: Arquiteturas de CPU
O APK de release SHALL conter código nativo para `armeabi-v7a` e `arm64-v8a`. O APK de debug SHALL conter também `x86_64`, para rodar em emuladores Android TV. Todas as bibliotecas nativas SHALL ter alinhamento de página de 16 KB.

#### Scenario: Emulador x86_64
- **WHEN** o APK de debug é instalado num emulador Android TV 11 x86_64
- **THEN** o lume é executado com a biblioteca `x86_64`

#### Scenario: Release sem x86_64
- **WHEN** o APK de release é inspecionado
- **THEN** ele contém apenas as bibliotecas `armeabi-v7a` e `arm64-v8a`

#### Scenario: Userspace 32 bits
- **WHEN** o lume é instalado em uma TV com CPU de 64 bits e userspace de 32 bits
- **THEN** o lume é executado com a biblioteca `armeabi-v7a`

### Requirement: Gráficos mínimos
O lume SHALL funcionar em GPUs que suportam apenas OpenGL ES 2.0.

#### Scenario: GPU só com GLES 2.0
- **WHEN** o dispositivo reporta OpenGL ES 2.0 como versão máxima
- **THEN** todas as telas são renderizadas corretamente

### Requirement: Permissões mínimas
O lume SHALL NOT solicitar a permissão de internet, nenhuma permissão perigosa (runtime) e nem `QUERY_ALL_PACKAGES`. A visibilidade dos apps SHALL ser declarada por consultas de intent (LEANBACK_LAUNCHER e LAUNCHER).

#### Scenario: Inspeção do manifesto
- **WHEN** o manifesto final do APK é inspecionado (`aapt dump permissions`)
- **THEN** não aparecem `android.permission.INTERNET` nem `QUERY_ALL_PACKAGES`

#### Scenario: Nenhum tráfego de rede
- **WHEN** o lume é usado em todas as telas
- **THEN** o processo não abre nenhuma conexão de rede

### Requirement: Requisitos de app de TV
O lume SHALL declarar o recurso leanback como suportado, declarar o toque como não obrigatório e fornecer um banner de TV de 320x180 com a marca lume.

#### Scenario: Aparece no Android TV
- **WHEN** o lume é instalado e outro launcher está ativo
- **THEN** o lume aparece na lista de apps do sistema com o seu banner

### Requirement: Artefatos de distribuição
Cada release SHALL produzir um APK assinado (para o GitHub) e um AAB (para a Play Store), assinados com a mesma chave de app e com o mesmo identificador de pacote, de modo que um APK do GitHub possa ser atualizado por uma versão mais nova de qualquer um dos canais.

#### Scenario: Build local em container
- **WHEN** alguém com apenas um runtime de containers (wslc ou Docker) roda `scripts/container-build.sh release` num clone do repositório
- **THEN** o APK e o AAB são gerados em `dist/`, com as mesmas versões de ferramentas usadas pelo CI

#### Scenario: Release por tag
- **WHEN** uma tag `vX.Y.Z` é publicada no repositório
- **THEN** o CI gera o APK e o AAB assinados, com `versionName` X.Y.Z, e anexa o APK ao release do GitHub

#### Scenario: Atualização entre canais
- **WHEN** o lume 0.1.0 do GitHub está instalado e o usuário instala a 0.2.0 vinda do mesmo processo de assinatura
- **THEN** a atualização é aceita, e a configuração do usuário é mantida
