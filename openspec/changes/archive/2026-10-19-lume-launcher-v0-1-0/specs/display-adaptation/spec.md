# Spec Delta

## Purpose

Garante que o lume apresente o mesmo layout, nítido e legível, em TVs de 30" a 60" com resolução de 720p a 4K, sem pagar memória de 4K.

## ADDED Requirements

### Requirement: Canvas lógico e escala uniforme
O layout SHALL ser definido em um canvas lógico de 1280x720 unidades e escalado uniformemente para a superfície de renderização pelo fator `min(largura/1280, altura/720)`. Em proporções diferentes de 16:9, o conteúdo SHALL ficar centralizado, com o fundo do tema preenchendo as sobras.

#### Scenario: 720p
- **WHEN** a superfície tem 1280x720 pixels
- **THEN** 1 unidade lógica equivale a 1 pixel

#### Scenario: 1080p
- **WHEN** a superfície tem 1920x1080 pixels
- **THEN** 1 unidade lógica equivale a 1,5 pixel, e a composição é idêntica à de 720p

### Requirement: Resolução máxima de renderização
O lume SHALL renderizar em, no máximo, 1920x1080 pixels. Em telas com resolução maior (ex.: 4K), o buffer de renderização SHALL ser limitado a 1920x1080 e ampliado pelo compositor/hardware do sistema.

#### Scenario: TV 4K
- **WHEN** a superfície de exibição informa 3840x2160
- **THEN** os buffers de renderização do lume têm 1920x1080, e a imagem ocupa a tela inteira

#### Scenario: TV 720p
- **WHEN** a superfície de exibição informa 1280x720
- **THEN** o lume renderiza nativamente em 1280x720

### Requirement: Texto e formas nítidos em qualquer escala
Texto, ícones e formas SHALL ser gerados no tamanho final em pixels da escala corrente, e não ampliados a partir de bitmaps de outro tamanho.

#### Scenario: Troca de resolução
- **WHEN** a resolução da superfície muda de 1280x720 para 1920x1080 com o lume aberto
- **THEN** o texto é regenerado para a nova escala e continua nítido, sem reiniciar o app

### Requirement: Área segura contra overscan
Todo conteúdo interativo ou textual SHALL ficar dentro de uma área segura com margens de pelo menos 32 unidades lógicas nas laterais e 18 em cima e embaixo. Somente fundos podem ultrapassá-la.

#### Scenario: TV com overscan de 5%
- **WHEN** a TV corta 5% de cada borda da imagem
- **THEN** o logo, o relógio e os tiles continuam inteiramente visíveis

### Requirement: Tamanho mínimo de texto
Nenhum texto da interface SHALL ter tamanho menor que 14 unidades lógicas.

#### Scenario: Legenda pequena
- **WHEN** o tema define o rótulo de tile com 13px
- **THEN** o build emite um aviso, e o texto é renderizado com 14 unidades
