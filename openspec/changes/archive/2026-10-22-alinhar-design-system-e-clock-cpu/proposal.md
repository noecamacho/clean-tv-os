# Proposal

## Why

A documentação e as especificações do projeto (`design-system`, `home-editor`, `theming`, `wallpaper`, `launcher-shell` e `docs/theme-format.md`) ficaram defasadas em relação à implementação real das telas do Lume no código (`crates/lume-core/src/app/*.rs` e `themes/lume/theme.css`), especialmente na tela "Editar tela Início", nas telas de rolagem ("Sobre o Lume" e "Informações da TV"), nas medidas do painel de estatísticas e na lista de seletores do tema. Além disso, no painel "Status da TV" da tela de Configurações, a linha de CPU exibia a razão entre o clock atual e o clock máximo (`scaling_cur_freq / cpuinfo_max_freq`) formatada como porcentagem (`90%`), o que em TVs cujo governador mantém o clock alto em repouso (como a AIWA em `1,35 GHz / 1,50 GHz`) dava a falsa impressão de que a CPU estava sobrecarregada, quando na verdade estava 95% ociosa.

## What Changes

- **Alinhamento completo das especificações e documentação ao código real do app**:
  - Atualizar `openspec/specs/design-system/spec.md` para refletir fielmente o layout atual de todas as telas:
    - **Editar tela Início (`editor.rs`)**: cabeçalho sem subtítulo, início do conteúdo em `y = 152` (`TOP`), primeiro painel `.panel` ("APARÊNCIA") unificando o carrossel horizontal de mini-cards de papel de parede (`178x100`, vão `16`, em `x = 96`, largura útil `1088`, com rolagem horizontal `Anim::WallpaperScrollX`, contador `"1 / 6 ›"`, scrim e setas laterais, selo ativo `18x18` e anéis `3 px` / `2 px` / `1 px`) e, após divisor de `1 px`, a linha "Favoritos redondos" (`.editor-row`, altura `64`, largura `1088`); segundo painel `.panel` ("ORDEM DOS FAVORITOS") com cabeçalho, dica à direita, linha de descrição (`DESCRIPTION_HEIGHT = 48`), cartões de `168x116` (`56` nome + `60` controles com 3 botões redondos de `44x44`) e área "Adicionar app"; remoção das referências ao antigo botão "Concluído" (apenas "Restaurar padrão" `.button-secondary` existe no rodapé); documentação explícita dos dois estilos de diálogo de confirmação existentes hoje no app (o cartão escuro `640x280` com escurecimento de tela cheia no Editor e o cartão claro `560x260` sem escurecimento no Mapeamento de controle).
    - **Telas "Sobre o Lume" e "Informações da TV" (`settings.rs`)**: documentar que não têm subtítulo, usam janela de corte (`push_clip`) a partir de `y = 152`, iniciam o texto em `y = 192` com passo de rolagem `96`, e detalhar as medidas de "Informações da TV" (título de seção `.card-heading` com altura `32`, linhas de `40` com rótulo em `x = 64..416` e valor em `TV_VALUE_X = 448` com peso `700`, e vão de `32` entre seções).
    - **Tela "Configurações" e painel "Status da TV" (`settings.rs`)**: documentar a altura do painel (`PANEL_HEIGHT = 312`), cabeçalho em `y + 24`, as 3 linhas em `y + 72 + i * 80` e as barras de `8 px` de altura (raio `4`) em `top + 36`.
    - **Régua de layout, Menu de contexto e Seletores**: corrigir a altura de `.editor-row` na régua para `64` (passo `72`), mantendo `.settings-item` em `72` (passo `88`); registrar na tabela do menu de contexto o posicionamento sobre linhas de mapeamento (`Origin::KeyRow`, alinhado à direita da linha com `ROW_GAP = 8`); atualizar a lista completa dos 30 seletores de componentes do tema.
  - Atualizar `openspec/specs/home-editor/spec.md` removendo a menção a "uma instrução curta" (subtítulo) no cabeçalho.
  - Atualizar `openspec/specs/theming/spec.md` com a lista completa dos 30 seletores suportados pelo compilador de tema.
  - Atualizar `openspec/specs/wallpaper/spec.md` corrigindo "10 opções" para "6 opções" no primeiro requisito.
  - Atualizar `openspec/specs/launcher-shell/spec.md` corrigindo o cenário "Aba ativa destacada" (com o foco na grade, o sublinhado da aba ativa é branco, não azul).
  - Atualizar `docs/theme-format.md` (exemplo de `.dock-label` com fundo transparente `rgba(0, 0, 0, 0)`) e o comentário de `--font-primary` em `themes/lume/theme.css` (o relógio usa `--font-brand`).
- **Mudança de "CPU" para "Clock da CPU" no painel Status da TV (`device-stats`)**:
  - Alterar o rótulo `stats_cpu` nos 5 arquivos `strings.xml` (`pt-rBR`: `"Clock da CPU"`, `en`: `"CPU clock"`, `es`: `"Reloj de CPU"`, `id`: `"Clock CPU"`, `ru`: `"Частота ЦП"`).
  - Ler em Rust (`/sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq` e `cpuinfo_max_freq`, com fallback para `scaling_max_freq`) a frequência média atual por núcleo e a frequência máxima por núcleo em kHz, sem alterar nenhum código Java.
  - Exibir na linha de clock da CPU do painel "Status da TV" o valor formatado como `<atual> / <máximo>` (ex.: `"1,35 GHz / 1,50 GHz"` ou `"750 MHz / 1,50 GHz"`, respeitando separador decimal por idioma), mantendo a barra de progresso proporcional a `current / max` e `"—"` quando indisponível.

## Capabilities

### New Capabilities
<!-- Nenhuma nova capacidade introduzida. -->

### Modified Capabilities
- `design-system`: Atualiza requisitos de botões, régua de layout, cabeçalhos, tela Editar tela Início, Informações da TV, Sobre o Lume, painel de estatísticas, menu de contexto e seletores para refletir a implementação real do app.
- `device-stats`: Atualiza o requisito do painel de estatísticas para exibir "Clock da CPU" no formato `<atual> / <máximo>` (em GHz/MHz por núcleo) com barra proporcional, em vez de porcentagem de frequência somada.
- `home-editor`: Ajusta a descrição do cabeçalho da tela de edição (sem subtítulo/instrução curta abaixo do título).
- `theming`: Atualiza a lista normativa dos seletores CSS suportados pelo tema para incluir todos os 30 componentes existentes.
- `wallpaper`: Corrige a contagem do catálogo de papéis de parede no requisito inicial de 10 para 6 opções.
- `launcher-shell`: Corrige o cenário "Aba ativa destacada" para refletir que o sublinhado é branco quando o foco está na grade de Apps.

## Impact

- **Especificações OpenSpec**: `design-system`, `device-stats`, `home-editor`, `theming`, `wallpaper`, `launcher-shell`.
- **Documentação e Tema**: `docs/theme-format.md`, comentário em `themes/lume/theme.css`.
- **Código Rust**:
  - `crates/lume-core/src/stats.rs`: adição de `format_freq(khz: u64, comma: bool) -> String` e leitura em Rust de `/sys/devices/system/cpu/cpu*/cpufreq/` por núcleo (`read_sysfs_cpu_freq()`).
  - `crates/lume-core/src/app/settings.rs`: exibição de `format!("{} / {}", format_freq(cur, comma), format_freq(max, comma))` para `CpuReading::Freq` no painel de estatísticas.
  - `crates/lume-app/src/android/bridge.rs`: uso de `lume_core::stats::read_sysfs_cpu_freq()` para preencher `stats.cpu` com a frequência média por núcleo em kHz.
  - `tools/lume-desktop/src/demo.rs`: ajuste dos valores sintéticos de `CpuReading::Freq` para kHz por núcleo (`1_350_000` e `1_500_000`).
- **Traduções**: `android/app/src/main/res/values*/strings.xml` (5 idiomas: `values`, `values-pt-rBR`, `values-es`, `values-id`, `values-ru`).
- **Nenhum arquivo Java modificado.**
