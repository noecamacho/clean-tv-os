# Proposal

## Why

O Lume hoje conta apenas com traduções em inglês (`en`) e português do Brasil (`pt-BR`). Para expandir o Lume no mercado internacional, em especial nos mercados emergentes onde o Android TV e TV boxes baratas predominam (América Latina, Sudeste Asiático e Leste Europeu/Eurásia), precisamos de suporte nativo a idiomas desses mercados.

Como o motor de texto do Lume renderiza via `fontdue` sem complex shaping (HarfBuzz) e sem suporte a escrita da direita para a esquerda (RTL), devemos focar em idiomas que utilizam escrita latina ou cirílica linear, garantindo renderização de texto perfeita e alinhada ao orçamento de desempenho do launcher.

## What Changes

- **Espanhol (`values-es/strings.xml`):**
  - Tradução completa de todas as chaves de texto e plurais.
  - Cobre o mercado latino-americano hispânico (México, Colômbia, Argentina, Chile, Peru, etc.) e Espanha.
  - Regra de plural: `Family::OneIsOne` (já existente no Lume).
- **Indonésio (`values-id/strings.xml`):**
  - Tradução completa em Bahasa Indonesia.
  - Cobre o 4º país mais populoso do mundo (~275 milhões de habitantes), com massiva adoção de Smart TVs Android (Polytron, Coocaa, Xiaomi, TCL).
  - Escrita latina sem diacríticos complexos.
  - Regra de plural: `Family::NoPlural` (já mapeada para a família `id` em `crates/lume-core/src/i18n/plural.rs`).
- **Russo (`values-ru/strings.xml`):**
  - Tradução completa em russo.
  - Cobre a grande comunidade de usuários de Smart TVs e TV boxes econômicas na Eurásia (Rússia, Cazaquistão, Bielorrússia, Uzbequistão, etc.).
  - Alfabeto cirílico linear (totalmente suportado pelas fontes do sistema e `fontdue`).
  - Regra de plural: `Family::EastSlavic` (já implementada em `crates/lume-core/src/i18n/plural.rs`).
- **Verificação do build:**
  - `lume-strings` valida automaticamente no build se todas as chaves de `strings.xml` estão presentes e se os marcadores de posição (`%1$s`, `%1$d`) coincidem com o idioma de referência.

## Capabilities

### Modified Capabilities

- `localization`: adiciona o requisito de suporte a espanhol (`es`), indonésio (`id`) e russo (`ru`), garantindo correspondência completa com o arquivo de referência e cobertura das regras de plural.

## Impact

- Recursos Android: criação de `android/app/src/main/res/values-es/strings.xml`, `android/app/src/main/res/values-id/strings.xml` e `android/app/src/main/res/values-ru/strings.xml`.
- Rust: `crates/lume-strings/` (testes de compilação dos recursos), `crates/lume-core/src/i18n/plural.rs` (conferência do mapeamento das tags `es`, `id`, `ru`).
- Zero impacto no laço de renderização, zero permissões novas, APK mantido bem abaixo do limite de 6 MB.
