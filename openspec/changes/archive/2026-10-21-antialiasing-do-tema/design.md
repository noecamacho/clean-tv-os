# Design

## Context

Ver `proposal.md` (Why) para os três defeitos. O estado do código que molda a solução:

- `render/shaders.rs`: `RECT_FRAGMENT` (formas por função de distância, cobertura `clamp(0.5 - d / unit)`, 1 px de rampa) e `IMAGE_FRAGMENT` (imagem pré-multiplicada vezes a cor do vértice, recorte opcional pelo mesmo cálculo). O papel de parede também passa pelo `IMAGE_FRAGMENT` (`ImageTexture::Wallpaper`), escurecido pela cor do vértice nas telas de texto.
- `app/dock.rs` (`draw_dock`): uma passada de `fill` na cor do app e outra de `image_rounded` com o mesmo raio; depois, a letra inicial enquanto o ícone não chega ou aparece.
- Grade de Apps (`app/mod.rs`, `draw_tile_image`): passadas de caixas (sombra, fundo e contorno via `draw_box`), de imagens (`list.image`, sem recorte, porque o banner já vem arredondado do Java) e de texto. A divisão em passadas existe para ter uma chamada de desenho por passada; alternar programas tile a tile custou cerca de 3 chamadas por tile na TV.
- `app_icons.rs`: atlas RGBA de 1024x2304, com células de tile de 256x144 e células do dock de 128x128. O tamanho da célula chega ao Java como parâmetro (`renderTile(width, height)`, `renderDockIcon(size)`).
- `LumeBridge.drawBanner`: desenha o banner num `sourceBitmap` e o copia arredondado para a célula com `BitmapShader` e `drawRoundRect`.

Medidas no canvas de 1280x720: tile de 211x118,7, raio 12, foco 1,05; favorito de 64 com pico de 1,6. Em 1080p a escala é 1,5.

## Goals / Non-Goals

**Goals:**
- Uma cobertura por borda, cantos no tamanho final, contorno sem fresta, rampa ajustável pelo tema, degradês com dither (ver specs).
- Manter uma chamada de desenho por passada, sem passada de tela cheia.

**Non-Goals:**
- Cobertura corrigida por gama (misturar em luz linear): exigiria converter cada fragmento e muda o peso visual de todo o texto e de todas as formas; a rampa mais larga resolve o que incomoda com custo zero.
- Antialiasing do texto: o atlas de glifos já é gerado no tamanho final. Os ícones da fonte de ícones são a exceção, pedida pelo autor ao ver a TCL: ganham a rampa de `--edge-softness` quando são rasterizados (filtro de 3 toques `[a, 1 − 2a, a]` nas duas direções, com `a = (s² − 1) / 24` para a rampa somada à do rasterizador dar `s`), sem custo por quadro.
- Máscara de ícone adaptativo do Android (`AdaptiveIconDrawable`): seria Java novo.

## Decisions

### 1. A imagem é composta sobre a cor do item no shader de imagem

O vértice de imagem ganha uma cor de fundo (`a_bg`, pré-multiplicada). O fragmento faz `cor = (tex * visível + bg * (1 - tex.a * visível)) * cobertura`, em que `visível` é o fade de entrada do ícone, aplicado só à imagem, e a `cobertura` é a do recorte. Com isso, a passada de `fill` do dock e o fundo do tile deixam de ser desenhados quando há imagem; continuam só para o item sem imagem (letra inicial, "Adicionar favoritos", tile sem arte).

Alternativas: desenhar o fundo 1 px menor (falha com ícones transparentes na borda, que mostrariam o papel de parede); desenhar o fundo e pular a imagem na borda (duas formas, mesmo problema). O fundo no vértice não muda o número de chamadas.

A forma recortada fica meia rampa (`--edge-softness / 2`, em px do buffer) para dentro do quad da imagem, para que a rampa inteira caia sobre pixels da própria imagem; sem isso, a metade de fora da rampa era cortada pelo quad nos pontos em que o círculo encosta nele. A alternativa testada, crescer o quad e prender a coordenada de textura na borda dentro do fragmento, foi descartada: conta sobre a coordenada no fragmento vira leitura dependente em fp16 na Mali-470 da TCL, com erro de cerca de 1 texel num atlas de 2600 linhas (até no desktop a amostra presa na borda puxava a célula vizinha). A forma fica cerca de 0,6 px do buffer menor de cada lado, o que não se percebe.

### 2. Cantos dos banners no shader

O `drawBanner` passa a desenhar o banner direto na célula, retangular (sai o `sourceBitmap`, o `BitmapShader` e o `drawRoundRect`; o `TILE_RADIUS_RATIO` também sai). O tile com banner usa `image_rounded` com o raio do tema (`--radius`), como o dock. A célula com ícone continua com o ícone centrado sobre transparência, agora composto sobre a cor do tile pela decisão 1.

É remoção de Java, autorizada pelo autor; nenhum método novo é chamado por JNI, então o `proguard-rules.pro` não muda.

### 3. Contorno de foco avança por baixo da imagem

Em vez de mudar a ordem das passadas (o contorno depois das imagens custaria uma chamada a mais e cobriria a imagem por cima), o anel do tile com imagem (`DisplayList::draw_box_under`) vai de 4 px para fora da borda até duas rampas (`2 × --edge-softness` px do buffer, `/ escala` no canvas) para dentro. A imagem continua desenhada depois; ela é recortada meia rampa para dentro (decisão 1) e fica opaca uma rampa depois, e o anel tem a sua meia rampa interna, então duas rampas garantem que a borda parcial da imagem mistura com o azul opaco do anel, e não com o papel de parede. A medida visual do anel não muda, porque a parte de dentro fica sob a imagem.

Durante a implementação o autor experimentou o anel afastado 3 px do tile (para não se misturar com tiles azuis) e o nome 4 px mais abaixo, viu na TCL e voltou ao anel colado e ao nome a 8 do tile.

### 4. Rampa no tema

Novo token `--edge-softness` (px do buffer, valor inicial 1.25, calibrado olhando na TCL), lido pelo tema e passado aos shaders de forma e de imagem como uniforme. A cobertura vira `clamp(0.5 - d / (unit * suavidade), 0, 1)`. O mesmo valor serve para formas, anéis e recortes, para que todas as bordas da interface tenham a mesma maciez.

### 5. Dither ordenado nos degradês

**Revisto na medida da TCL.** A versão abaixo, com o cálculo por pixel no `IMAGE_FRAGMENT`, levou a grade de Apps a ~60% de quadros de 33 ms ou mais e o dock a 44% (base: 1,5% e 0%): o papel de parede é um quad de tela cheia a cada quadro, e a Mali-470 não tem ALU de fragmento para isso. Ficou assim: o papel de parede tem programa próprio (`WALLPAPER_VERTEX`/`WALLPAPER_FRAGMENT`, tão curto quanto antes do change), e o dither dele é uma textura Bayer 4x4 em `REPEAT`/`NEAREST` lida com coordenadas que o vértice calcula (`posição / 4`), uma leitura direta sem conta no fragmento. Não há chamada a mais: o papel de parede já era um lote à parte. No `RECT_FRAGMENT`, o cálculo por pixel ficou só para formas com degradê, que são pequenas.

Dither ordenado 4x4 (Bayer) calculado com `floor`/`mod` sobre `gl_FragCoord`, com amplitude de ±0,5/255, somado antes de escrever o fragmento: no `RECT_FRAGMENT` só quando a forma tem degradê (`v_misc.w`), e no `IMAGE_FRAGMENT` só para o papel de parede (um parâmetro do vértice o marca). Evita a função `fract(sin(...))`, que perde precisão em `mediump` (o `IMAGE_FRAGMENT` é `mediump`) e geraria padrões em GPUs como a da AIWA. O Bayer em ±0,5 nível não se percebe como textura e tem média zero. Importa sobretudo no papel de parede escurecido: misturar 22% de imagem com `--bg` reduz os degradês a poucas dezenas de níveis.

### 6. Células do atlas no tamanho do pico

O lado da célula passa a ser calculado pela escala corrente, arredondado para cima em múltiplo de 4:

| | 1280x720 | 1920x1080 |
|---|---|---|
| Célula do dock (pico 64 × 1,6) | 104x104 | 156x156 |
| Célula do tile (172 × 1,05, 16:9) | 184x104 | 272x153 |
| Atlas (4 x 14 tiles + 16 do dock) | 736x1768 (5,2 MB) | 1088x2610 (11,4 MB) |
| Antes | 1024x2304 (9,4 MB) | 1024x2304 (9,4 MB) |

Correção feita na implementação: a primeira versão desta tabela usava um tile de 211 de largura (medida antiga); o tile da grade tem 172 (`apps.rs`), e os tamanhos saem do código (`AtlasLayout::for_scale`, com `app::image_peaks`), não de números fixos. O ícone em repouso fica reduzido no máximo cerca de 0,62x (96 de 156), dentro do que o filtro bilinear faz bem sem mipmap. Em 720p o atlas encolhe 4,2 MB; em 1080p cresce 2 MB de memória gráfica. Como o atlas já é liberado ao dormir e refeito ao acordar, a troca de resolução recria o atlas com o novo tamanho. Este passo entra por último e só fica se a PSS medida na TCL continuar dentro dos 64 MB com folga (tarefa própria).

## Risks / Trade-offs

- [O fundo no vértice da imagem aumenta o vértice e a banda] → São 16 bytes a mais por vértice de imagem, só nos quads de imagem; medir os quadros perdidos na grade antes e depois.
- [O anel por baixo da imagem aparece quando o ícone tem transparência na borda] → O ícone já é composto sobre a cor do item (decisão 1), que é opaca, então o azul só aparece na faixa de cobertura parcial, onde é o efeito desejado.
- [Banners de apps que desenham a própria borda arredondada] → Continuam recortados pelo raio do tile; se o canto da arte for maior, não aparece diferença; se for menor, o recorte do Lume prevalece, como hoje.
- [Atlas maior em 1080p] → Decisão 6 condicionada à medida; sem folga, volta ao tamanho atual e o limite de 1,05x fica só no dock, com a spec ajustada no mesmo change.
- [Rampa mais larga deixa as bordas macias demais] → Valor no tema, calibrado na TCL entre 1 e 1,5.
- [A `lume-desktop` não carrega ícones de apps] → Os testes usam imagens sintéticas (ícone branco opaco, banner de cor sólida) injetadas pelo demo, só no desktop.

## Migration Plan

Sem migração de dados: a configuração e os arquivos do usuário não mudam. Reverter o change é reverter o commit; o Java removido volta junto.
