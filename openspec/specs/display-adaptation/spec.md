# display-adaptation Specification

## Purpose
Garante que o lume apresente o mesmo layout, nítido e legível, em TVs de 30" a 60" com resolução de 720p a 4K, sem pagar memória de 4K.

## Variante HOME para a TCL 55Q5K
Nesta variante pessoal, o buffer de renderização volta a 1920×1080 (limite máximo). A GPU da TV (PowerVR BXE-4-32) não preenche um quadro inteiro de 1080p com vidro a cada 16 ms (mediana de ~37 ms), por isso cada quadro redesenha apenas o que mudou: a lista de desenho é comparada com a do quadro anterior, o retângulo da diferença (unido aos dos quadros que a idade do buffer, `EGL_EXT_buffer_age`, pede) vira o `glScissor` e a região de `EGL_KHR_partial_update`. Trocas de textura (ícones, papel de parede, atlas refeito) e buffers de idade desconhecida forçam o quadro inteiro. Medido na TCL navegando na primeira fileira: mediana 16,7 ms, p95 17,0 ms em 1920×1080.

## Requirements

### Requirement: Canvas lógico e escala uniforme
O layout SHALL ser definido em um canvas lógico de 1280x720 unidades e escalado uniformemente para a superfície de renderização pelo fator `min(largura/1280, altura/720)`. Em proporções diferentes de 16:9, o conteúdo SHALL ficar centralizado, com o fundo do tema preenchendo as sobras.

#### Scenario: 720p
- **WHEN** a superfície tem 1280x720 pixels
- **THEN** 1 unidade lógica equivale a 1 pixel

#### Scenario: 1080p
- **WHEN** a superfície tem 1920x1080 pixels
- **THEN** o buffer usa 1920×1080, com 1,5 pixel por unidade lógica

### Requirement: Resolução máxima de renderização
HOME SHALL renderizar em, no máximo, 1920×1080 pixels. Em telas com resolução maior, o buffer SHALL ser ampliado pelo compositor/hardware do sistema.

#### Scenario: TV 4K
- **WHEN** a superfície de exibição informa 3840x2160
- **THEN** os buffers de renderização de HOME têm 1920×1080, e a imagem ocupa a tela inteira

#### Scenario: TV 720p
- **WHEN** a superfície de exibição informa 1280x720
- **THEN** o lume renderiza nativamente em 1280x720

### Requirement: Texto e formas nítidos em qualquer escala
Texto, ícones e formas SHALL ser gerados no tamanho final em pixels da escala corrente, e não ampliados a partir de bitmaps de outro tamanho. O recorte de imagens (cantos de banners, favoritos redondos) SHALL ser feito no tamanho final em pixels, e não embutido na imagem. As imagens de apps (banners e ícones da grade e do dock) SHALL ser guardadas em tamanho que não as obrigue a ampliar mais de 1,05x nem a reduzir abaixo de 0,5x no maior tamanho em que aparecem (foco ou pico da Magnification) na escala corrente.

#### Scenario: Troca de resolução
- **WHEN** a resolução da superfície muda de 1280x720 para 1920x1080 com o lume aberto
- **THEN** o texto é regenerado para a nova escala e continua nítido, sem reiniciar o app

#### Scenario: Tile com foco em 1920x1080
- **WHEN** um tile com banner recebe o foco e é ampliado 1,05x em 1920x1080
- **THEN** o banner é desenhado a partir de uma imagem ampliada no máximo 1,05x, e os cantos são recortados no tamanho final

#### Scenario: Favorito no pico da Magnification
- **WHEN** um favorito do dock está no pico da Magnification em 1920x1080 ou em 1280x720
- **THEN** o ícone é desenhado a partir de uma imagem ampliada no máximo 1,05x

#### Scenario: Favorito em repouso em 1280x720
- **WHEN** um favorito do dock está em repouso em 1280x720
- **THEN** o ícone é desenhado a partir de uma imagem reduzida no máximo 2x, sem tremer quando a Magnification passa por ele

### Requirement: Área segura contra overscan
Todo conteúdo interativo ou textual SHALL ficar dentro de uma área segura com margens de pelo menos 32 unidades lógicas nas laterais e 18 em cima e embaixo. Somente fundos podem ultrapassá-la.

#### Scenario: TV com overscan de 5%
- **WHEN** a TV corta 5% de cada borda da imagem
- **THEN** o logo, o relógio e os tiles continuam inteiramente visíveis

### Requirement: Tamanho mínimo de texto
Nenhum texto da interface SHALL ter tamanho menor que 16 unidades lógicas. A escala uniforme (canvas lógico × `min(largura/1280, altura/720)`) SHALL manter a mesma composição em qualquer resolução 16:9, e em outras proporções o conteúdo SHALL ficar centralizado dentro da área segura, sem texto cortado nem novas reticências.

#### Scenario: Legenda pequena
- **WHEN** o tema define o rótulo de tile com 13px
- **THEN** o build emite um aviso, e o texto é renderizado com 16 unidades

#### Scenario: Mesma composição em 720p, 1080p e 4K
- **WHEN** a mesma tela é exibida em 1280x720, 1920x1080 e 3840x2160 (limitado a 1920x1080)
- **THEN** as posições e a quebra de linhas em unidades lógicas são idênticas, e só o fator de escala muda

#### Scenario: Proporção diferente de 16:9
- **WHEN** a superfície é 1024x768 (4:3) ou 2560x1080 (21:9)
- **THEN** o conteúdo fica centralizado, com o fundo do tema nas sobras, e todo texto continua dentro da área segura
