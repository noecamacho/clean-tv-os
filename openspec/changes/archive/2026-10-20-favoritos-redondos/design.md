# Design

## Context

- `HomeConfig` (`crates/lume-core/src/home/config.rs`) persiste a configuração da tela Início em `config.v1` usando `storage::Kv` (hoje apenas a chave `favorites`).
- `app/editor.rs` desenha a tela `Configurações › Tela Início` com o painel `.panel` de favoritos (cartões de `168x116`, vão de `24`, 5 por linha) e os botões `Restaurar padrão` e `Concluído`.
- `app/dock.rs` desenha o fundo colorido de cada item do dock com `RADIUS_RATIO = 0.22` e a textura da imagem por cima; em `LumeBridge.java`, `renderDockIcon` hoje recorta a imagem com `DOCK_RADIUS_RATIO = 0.22f` antes de gravar o cache `d1.rgba`.
- `themes/lume/theme.css` já define `.editor-row` (fundo `--surface-row`, foco `--surface-focus` com anel `--focus-ring`).

## Goals / Non-Goals

**Goals:**
- Permitir ligar e desligar "Favoritos redondos" na tela `Configurações › Tela Início`, com efeito imediato no dock sem precisar regerar ou recarregar os ícones em cache.
- Enxugar os vãos verticais e horizontais do editor dentro da régua de 4/8 do design system, fazendo caber 6 cartões de favoritos por linha sem sobra lateral.

**Non-Goals:**
- Mudar o formato dos tiles 16:9 da grade de Apps (continuam retangulares 16:9).
- Adicionar novas telas de configuração fora de `Configurações › Tela Início`.

## Decisions

### 1. Persistência em `HomeConfig` (`round_favorites`)

`HomeConfig` ganha o campo `pub round_favorites: bool` (padrão `false` em `HomeConfig::first_use`, que também é o estado aplicado por "Restaurar padrão").
- Na escrita (`to_kv`), grava `round_favorites=1` ou `round_favorites=0` junto de `favorites`.
- Na leitura (`from_kv`), lê `kv.get("round_favorites") == Some("1")`; quando a chave não existe (arquivos `config.v1` de versões anteriores), assume `false`, mantendo compatibilidade total.

### 2. Linha "Favoritos redondos" no topo do editor (`.editor-row`)

Acima do painel de favoritos, em `y = 184` (`32 px` abaixo do subtítulo em `152`), o editor desenha uma linha `.editor-row` de `1152 x 64` (`LEFT = 64`, `WIDTH = 1152`):
- À esquerda (`x = LEFT + 24`): o rótulo `StringId::EditorRoundFavorites` ("Favoritos redondos" / "Round favorites") no estilo de `.editor-row` (`22 px`, peso `700`, `--text`).
- À direita (`x = LEFT + WIDTH - 24 - 48`, centrado verticalmente): uma chave Liga/Desliga (`switch`) de `48 x 28` (raio `14`) com bolinha interna de `20 x 20` (raio `10`, margem de `4`):
  - **Desligado:** trilha `--control`, bolinha `--text-muted` à esquerda (`x + 4`).
  - **Ligado:** trilha `--primary`, bolinha `--on-primary` à direita (`x + 24`).
- A linha inteira é um único `Focusable` (`ROUND_FAVORITES`): o foco inicial ao abrir o editor começa nela, e `OK` inverte `self.config.round_favorites` e emite `Effect::SaveConfig`.

### 3. Vãos enxutos na régua de 4/8 e 6 cartões por linha

Tabela de métricas do editor no canvas `1280x720`:

| Item | Caixa | Regra na régua |
|---|---|---|
| Caminho / Título / Subtítulo | linhas-base `52`, `116`, `152` | cabeçalho padrão de tela secundária (`page.rs`) |
| Linha "Favoritos redondos" | `x = 64..1216`, `y = 184..248` (`h = 64`) | `32` abaixo da linha-base do subtítulo (`152`); `.editor-row` de `64` |
| Vão até o painel | `16` (`y = 248..264`) | múltiplo de 8; mesmo vão entre itens de Configurações |
| Painel de favoritos (`.panel`) | `x = 64..1216`, começa em `y = 264` | enchimento lateral `32` (`1088` úteis) e vertical `24` |
| Cabeçalho do painel → cartões | `32` entre o topo interno e o topo dos cartões | ~`16` de vão visual abaixo da legenda de `16 px` |
| Grade de cartões | 6 colunas de `168` com vão `16` (`CARD_GAP = 16`), `h = 116` | `6 * 168 + 5 * 16 = 1088` (preenche 100% da largura útil, 12 favoritos em 2 linhas) |
| Cartões → "Adicionar app:" | `20` | múltiplo de 4 |
| Painel → botões inferiores | `24` (`BUTTONS_GAP = 24`) | múltiplo de 8; botões de `48` de altura nas margens |

### 4. Recorte do ícone do dock na GPU para troca instantânea

Hoje `LumeBridge.renderDockIcon` recorta o bitmap com `DOCK_RADIUS_RATIO = 0.22f` antes de salvar no cache em disco (`d1.rgba`). Se o recorte ficasse preso ao bitmap salvo, alternar "Favoritos redondos" exigiria invalidar e reprocessar o cache de todos os favoritos.
- Em vez disso, `LumeBridge.renderDockIcon` passa a gravar o quadrado completo do ícone (`drawWholeIcon`, sem `drawRoundRect`) e incrementa `DOCK_CACHE_VERSION` para `"d2"` (descartando uma única vez os arquivos `d1` já cortados).
- No Rust (`DisplayList::image_rounded` / `Cmd::Image` + shader `IMAGE_FRAGMENT`), um quad de imagem pode receber um raio de canto (0 para imagens retangulares comuns como banners e papel de parede; `rect.w * 0.22` ou `rect.w * 0.5` para os ícones do dock e para o `dock_ghost`), aplicando a mesma função de distância assinada (`round_rect_distance`) de `RECT_FRAGMENT` com antialiasing de 1 pixel físico.
- Assim, `dock.rs` escolhe `radius_ratio = if self.config.round_favorites { 0.5 } else { 0.22 }` tanto para o fundo colorido quanto para a imagem e para o fantasma de remoção, e a troca entre quadrado arredondado e círculo é imediata.

## Risks / Trade-offs

- [Cache `d1` antigo na TV com cantos já cortados em `0.22`] → subir `DOCK_CACHE_VERSION` de `"d1"` para `"d2"` em `LumeBridge.java` faz o `pruneCache` substituir automaticamente os arquivos antigos na primeira abertura.
- [Aumento de atributos no `TexVertex` de imagens] → adicionar centro/meio-tamanho/raio (ou coordenadas locais e raio) apenas aos vértices de `Program::Image` (que tem poucas dezenas de quads por quadro) tem custo desprezível na GPU e mantém `Program::Text` intacto.
