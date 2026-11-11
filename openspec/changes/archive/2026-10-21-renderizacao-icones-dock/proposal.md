# Proposal

## Why

Em TVs com menor densidade de tela física (como Smart TVs 720p em `tvdpi`/213 dpi), o Android fornece por padrão ícones com baixa resolução (~48px ou 64px). Quando esses ícones são desenhados na célula de 128x128 do dock, eles perdem nitidez e definição visual.

Além disso, aplicativos com ícones não-adaptativos ou que contêm transparência (como o Globoplay) precisam acompanhar harmonicamente a geometria configurada para os demais favoritos no dock (quadrado de cantos arredondados por padrão ou circular), sem regras ou condicionais específicas de app no código.

## What Changes

- **Carregamento preferencial em alta resolução (`LumeBridge.java`):**
  - O método `loadIcon` busca preferencialmente o drawable em densidade `DisplayMetrics.DENSITY_XXHIGH` (480 dpi, ~144px+), garantindo assets nítidos para a textura de 128x128 mesmo em TVs 720p.
  - Limite estrito de no máximo 2 tentativas (atividade e aplicação) com fallback direto para o padrão do sistema caso o app não forneça alta densidade.

- **Filtragem bilinear e anti-aliasing (`LumeBridge.java`):**
  - Aplicação de `PaintFlagsDrawFilter` com `ANTI_ALIAS_FLAG` e `FILTER_BITMAP_FLAG` para garantir que o downscale dos assets para 128x128 ocorra com bordas suaves e agradáveis.
  - Invalidação de cache via `DOCK_CACHE_VERSION = "d5"`.

- **Consistência visual dos favoritos (`dock.rs`):**
  - Mantém o preenchimento base `p.list.fill` de forma universal para todos os favoritos, assegurando que ícones com transparência acompanhem a forma geométrica global do dock.

## Capabilities

### Modified Capabilities

- `app-catalog`: especifica a preferência por alta resolução no carregamento dos ícones do dock e a consistência geométrica dos favoritos com transparência.

## Impact

- Código Java: ~30 linhas em `LumeBridge.java` para consulta de alta densidade e filtragem suave.
- Zero impacto no tamanho do APK e zero impacto de performance em navegação normal (o carregamento em alta resolução roda apenas uma vez por app na gravação do cache em disco).
