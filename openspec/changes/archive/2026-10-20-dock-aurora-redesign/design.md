# Design

## Context

Ver `proposal.md` para a motivação. O que existe hoje e condiciona a abordagem:

- Renderer GLES2 próprio: lista de comandos (`DisplayList`) com retângulos arredondados, gradientes de até 3 paradas, anéis, sombras, glifos e imagens. Não há blur, gradiente radial nem polígonos. As imagens usam um único programa (`Program::Image`) e uma única textura, o atlas de ícones (1024x2048 RGBA, cerca de 8 MB), com células de 256x144 preenchidas pela plataforma via `iconPixels` (banner 16:9 ou ícone centralizado).
- O buffer de renderização é limitado a 1920x1080 e ampliado pelo sistema em 4K (`display-adaptation`).
- O foco é um conjunto de `Row`/`Focusable` recalculado a cada quadro em `draw_home` e `draw_apps`. Animações usam `Anim::*` e `focus_progress(id)`.
- Quadros só são pedidos quando algo muda (`resource-efficiency`). O carrossel hoje mantém um prazo (`carousel_deadline`).
- Os `.so` ficam sem compressão no APK (`useLegacyPackaging = false`) e há duas ABIs, então um asset embutido conta duas vezes.
- O tema é CSS compilado no build (`lume-css` + `build.rs`), com lista fixa de seletores.

## Goals / Non-Goals

**Goals:**
- Dock com Magnification suave a 60 fps, sem quadros extras quando parado.
- Papel de parede nítido a 1080p (e portanto em 4K) dentro dos orçamentos de memória e APK.
- Remover o carrossel por completo, sem código morto.

**Non-Goals:**
- Troca ou escolha de papel de parede (spec futura). Só o ponto único de carregamento é deixado pronto.
- Fatores de Magnification configuráveis pelo tema (ficam no código).
- Animação do papel de parede.
- Alterar a lógica de catálogo, histórico de uso (`usage.rs` continua gravando) ou abertura de apps.

## Decisions

### 1. Papel de parede: base suave de 640x360 + estrelas em vetor
A aurora (cortinas, raios, brilho, fundo) é gerada por `tools/wallpaper/aurora06.py` (só biblioteca padrão do Python, determinística) em 640x360 RGB e nas posições de 420 estrelas (gerador pseudoaleatório fixo). O script grava `crates/lume-core/assets/wallpaper/aurora-640x360.rgb` e `aurora-stars.txt`, versionados no repositório; o `build.rs` os embute (`include_bytes!` e uma lista de estrelas em Rust). A base é enviada à GPU como uma textura RGB própria (675 KB), com filtro linear e `CLAMP_TO_EDGE`, e desenhada cobrindo a área visível (modo "cover") antes de qualquer outra coisa. As estrelas são desenhadas por cima como retângulos de 1 ou 2 pixels do buffer de renderização (`Painter::wallpaper`), com posições em coordenadas de 1920x1080 que acompanham o recorte do modo "cover".

Medido contra uma referência renderizada direto em 1920x1080: 640x360 dá erro médio de 0,93/255 e é indistinguível; 480x270 deixa a borda das cortinas visivelmente borrada; 384x216 (primeiro plano) é pior ainda. A borda inferior da cortina é suavizada para cerca de 3 texels na base, assim a textura a reproduz sem perda. O topo e a metade de baixo já nascem escuros na própria imagem (para a barra superior e o dock).

Alternativas descartadas:
- *Textura 1920x1080:* 8 MB de GPU, estoura o orçamento de 45 MB com o atlas.
- *Compressão ETC1:* blocos de 4x4 com 2 cores quebram gradientes suaves (banding e blocos).
- *Shader procedural por quadro:* caro numa GPU de TV fraca a cada quadro de animação de foco.
- *Gradientes empilhados:* só linear, 3 paradas, faixas visíveis.
- *Gerar em tempo de execução na CPU:* uns 50 a 150 ms em ARM de TV, ameaça os 300 ms de retorno a quente.

O custo no APK é de cerca de 675 KB por ABI 1,35 MB no total (por isso o limite de 1,5 MB na spec). Se a medição real do APK mostrar que isso aperta os 6 MB, o plano de recuo é mover os bytes para um asset do APK (comprimido uma vez, cerca de 157 KB) e entregá-los pela ponte, sem mudar a spec.

Ponto único de carregamento: um módulo `wallpaper` expõe `current() -> &'static Wallpaper` (bytes, dimensões e posições das estrelas). Uma futura troca só muda a fonte dessa função e a spec de configuração.

Com o papel de parede cobrindo toda a área visível, o preenchimento de fundo liso atual (`p.list.fill(visible, ... BG)`) sai de cada tela. Isso poupa um quadro cheio de fill. A cor `--bg` fica como reserva enquanto a textura não existe (primeiro quadro antes do upload).

O comando de imagem ganha um seletor de textura (`Icons` ou `Wallpaper`), e o lote quebra quando ele muda. Como o papel de parede é sempre o primeiro, há no máximo uma troca de textura por quadro.

### 2. Escurecimento nas telas de texto
O topo da imagem já nasce escuro (os 12% de cima), para o logo, as abas e o relógio. Nas telas com muito texto (Apps, Configurações, Sobre, edição), um retângulo `rgba(15, 23, 30, 0.78)` é desenhado sobre o papel de parede, para manter 4.5:1 em qualquer ponto da foto. Na tela Início o papel de parede aparece inteiro. O valor exato do escurecimento é ajustado numa medição de contraste (tarefa própria). Isso é um retângulo a mais num quadro cheio, e só nessas telas.

### 3. Ícones do dock: faixa quadrada no atlas
O atlas passa de 1024x2048 para 1024x2304 (cerca de +1 MB). A faixa nova (8 colunas x 2 linhas de 128x128, y de 2048 a 2304) guarda até 16 ícones de dock, mais que o limite de 12 favoritos. A plataforma ganha a chamada `dockIcon` (ao lado de `icon`), que devolve um ícone de 128x128 em quadrado arredondado, com a máscara aplicada no Java (o ícone adaptativo é desenhado por inteiro e recortado). Cada favorito tem, portanto, até duas células: a de banner/ícone da grade e a de dock, e o cache em disco passa a ter chave por tipo. O Rust desenha um retângulo arredondado na cor do app sob o ícone, igual ao que já faz com `Art::Icon`.

128 px é um pouco menos que o necessário no pico (ícone de 64 unidades x 1,6 x 1,5 de escala de 1080p é cerca de 154 px), com ampliação de cerca de 20% só no item focado. Se for visível, a faixa sobe para 160x160 (6 colunas x 2 linhas = 12 células, 1024x2368).

### 4. Dock: uma posição de foco animada e um núcleo de Magnification
Em vez de um progresso por item, o dock anima **uma posição de foco contínua** `p` (índice fracionário) e uma **amplitude** `a` (0 fora do dock, 1 com o foco nele), ambas com o `Anim` existente (150 ms, ease-out). A escala do item `i` é `1 + (pico - 1) * a * k(|i - p|)`, com `k` uma tabela fixa por distância (1,0; 0,45; 0,15; 0 em 3 ou mais) interpolada linearmente. Com pico 1,6 isso dá 1,6 / 1,27 / 1,09 / 1,0. Todas as constantes (pico, tabela, tamanho base de 64, espaço de 14, preenchimento de 16) ficam no código do dock.

O layout **empurra os vizinhos**, como o macOS: as posições vêm da soma cumulativa das larguras ampliadas mais o espaço, centralizadas, e a barra acompanha a largura total. A âncora é a base: os itens crescem para cima. Com 12 favoritos o dock mede cerca de 1040 unidades no pico, dentro da área segura. A spec permite sobreposição, mas o empurrão é mais fiel e evita sobreposição de cantos arredondados.

Os `Focusable` do dock usam os retângulos em repouso (sem ampliação), para que a navegação por vizinhança não oscile com a animação. `entry_down` da barra superior vai ao primeiro item, e `entry_up` do dock volta à aba ativa. O balão com o nome é uma caixa arredondada (`.dock-label`) acima do retângulo ampliado do item focado, deslocada para ficar dentro da área segura e com o nome truncado.

O dock (barra) é um retângulo arredondado com preenchimento translúcido `rgba(15, 23, 30, 0.55)` mais um anel de 1 px claro e sombra, sem blur, sobre o papel de parede. A estética vem do tema: seletores novos `.dock` e `.dock-label` (background, border-radius, padding, gap, box-shadow, cor e fonte), registrados em `lume-css`, `build.rs`, `theme.rs` e `docs/theme-format.md`.

Quadros: durante a animação o laço pede quadros; ao terminar, nenhum prazo permanece. O prazo do carrossel (`carousel_deadline`) é removido, então a tela Início ociosa volta a ter só o relógio.

### 5. Remoção do carrossel
Saem: `home/slides.rs`, `draw_hero`/`draw_slide`/`draw_pager`, o estado `Carousel`, `Anim::SlideFade`, `HERO_BUTTON`, o painel e as strings do carrossel no editor (`StringId` e `lume-strings`), os seletores `.hero`, `.hero-badge`, `.hero-title`, `.hero-subtitle` e `.pager-dot` (e o teste de `tint` do hero em `theme.rs`). **Fica** `.button`, que o editor usa. O histórico de uso (`usage.rs`) continua sendo gravado, pois `app-catalog` ainda o exige.

O editor deixa de listar seções: a tela principal passa a ser direto a edição de favoritos. É a maior mudança de UI fora do dock, porque `editor.rs` (cerca de 1300 linhas) monta o layout em torno das linhas de seção.

### 6. Configuração
`HomeConfig` perde `sections`, `carousel_apps` e `interval`. `from_kv` deixa de exigir `sections` e `carousel.apps` (hoje `kv.list(...)?` devolveria `None` e reiniciaria o usuário), lê só `favorites` e ignora o resto. `to_kv` grava só `favorites`. O nome `config.v1` não muda. Um lume antigo lendo um arquivo novo o trata como inutilizável e volta ao padrão (não há suporte a downgrade).

### 7. Abas e engrenagem
O sublinhado azul passa a ter um único alvo: a aba em foco quando o foco está numa aba, senão a aba ativa. A intensidade vem do `focus_progress` da aba, com a transição já existente. O anel de foco da aba (`topbar.rs`) e o `outline` e `border-radius` de `.tab:focus` saem; fica `color`. Em `.icon-button:focus` o `outline` passa a `3px solid var(--accent)`.

### 8. Grade de Apps
`COLUMNS` 6, `GAP` 24, tile 172x96,75 (a largura sai de `(1152 - 5*24) / 6`), `ROW_PITCH` cerca de 148,5 (altura do tile mais a faixa do nome, a mesma de hoje). O `.tile` do CSS (211x119) vira 172x97. A célula do atlas (256x144) continua cobrindo o tile em 1080p (172 x 1,5 = 258 px, ampliação de cerca de 1%).

## Risks / Trade-offs

- **Editor sem seções é uma reescrita parcial de `editor.rs`** → fazer em tarefa própria, antes do dock, com os testes de `tests.rs` do editor ajustados; manter o comportamento de favoritos, "Restaurar padrão" e persistência.
- **Memória:** atlas +1 MB e papel de parede +0,7 MB contra o orçamento de 45 MB de PSS → medir com `dumpsys meminfo` já na primeira tarefa de dock e na do papel de parede; se estourar, reduzir a faixa do dock (128 px é o piso) ou o atlas.
- **APK:** +1,35 MB com as duas ABIs → conferir com `scripts/verify-apk.py`; recuo no item 1.
- **Contraste sobre a aurora** nas telas de texto → escurecimento do item 2, validado por medição, e não por inspeção visual.
- **Ícones adaptativos do Android** variam muito (forma, fundo transparente) → a máscara quadrada arredondada e a cor do app por baixo cobrem os casos; testar com ícones de teste claros, escuros e transparentes.
- **Custo de GPU:** o papel de parede é um quadro cheio com amostragem linear a cada quadro, em vez do preenchimento liso → o preenchimento de fundo sai (item 1); medir a fluidez (`gfxinfo`) na navegação contínua do dock e da grade.
- **Sem blur:** o dock translúcido é só alfa e anel, menos "vidro" que o macOS → aceito; um blur real exigiria uma passada nova de renderização e fica fora do escopo.
