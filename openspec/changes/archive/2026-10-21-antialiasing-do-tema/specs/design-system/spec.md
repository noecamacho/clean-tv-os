## ADDED Requirements

### Requirement: Uma única borda suavizada por forma
Toda forma arredondada que leva uma imagem (favorito do dock, tile da grade com banner ou ícone) SHALL ser desenhada com uma única borda suavizada: a imagem é composta sobre a cor do item e o conjunto é recortado uma vez pela forma. Nenhum pixel da borda SHALL receber duas coberturas parciais da mesma forma, e a cor de fundo do item SHALL NOT aparecer como anel em volta da imagem.

#### Scenario: Favorito redondo com ícone
- **WHEN** o dock mostra um favorito redondo cujo ícone é branco e opaco, sobre a cor do app, em 1920x1080
- **THEN** nenhum pixel da borda tem a cor do app: a borda vai do branco do ícone ao papel de parede, sem anel entre eles

#### Scenario: Ícone ainda aparecendo
- **WHEN** o ícone de um favorito está no meio do fade de entrada, sobre a letra inicial
- **THEN** a cor do app continua visível por baixo do ícone e a borda da forma segue com uma única suavização

#### Scenario: Tile com banner
- **WHEN** a grade de Apps mostra um tile com banner
- **THEN** os cantos do banner são recortados pelo raio do tile no tamanho final em pixels, sem anel da cor do tile

### Requirement: Contorno de foco sem fresta
O contorno de foco de um tile com imagem SHALL ficar colado ao tile e passar por baixo da borda da imagem até onde ela é opaca, de modo que nenhum pixel entre o contorno e a imagem mostre o papel de parede ou o fundo do tile. Visto de fora, o contorno SHALL manter a largura de 4 px e a distância ao nome do app.

#### Scenario: Tile com foco em 1920x1080
- **WHEN** um tile com banner recebe o foco em 1920x1080
- **THEN** ao longo de qualquer linha que cruza o contorno e entra na imagem, pelos lados, pelo topo e pelo canto, nenhum pixel é mais escuro que o mais escuro entre o azul do contorno e o pixel da imagem logo depois dele (tolerância de 2 em 255)

#### Scenario: Tile com foco em 1280x720
- **WHEN** um tile com banner recebe o foco em 1280x720
- **THEN** o contorno azul encosta na imagem sem fio escuro, como em 1920x1080

### Requirement: Largura da suavização das bordas
A suavização das bordas de formas e recortes SHALL ter largura definida no tema (`--edge-softness`, em pixels do buffer de renderização, de 1 a 1,5) e igual em toda a interface. A transição entre o interior e o exterior de uma borda SHALL ocupar no máximo 2 px do buffer, em 1280x720 e em 1920x1080, inclusive durante as animações de foco e de Magnification.

#### Scenario: Borda de um favorito redondo
- **WHEN** um favorito redondo de cor sólida é desenhado sobre um fundo sólido, em repouso ou ampliado pela Magnification
- **THEN** a transição entre a cor do item e a do fundo ocupa de 1 a 2 px do buffer em qualquer direção

#### Scenario: Valor no tema
- **WHEN** `--edge-softness` muda no `themes/lume/theme.css`
- **THEN** todas as bordas suavizadas da interface usam a nova largura, sem valor fixo no código

### Requirement: Ícones da fonte com a mesma suavização
Os ícones da fonte de ícones (entradas, engrenagem e os demais Lucide) SHALL ter a borda suavizada com a mesma largura das formas (`--edge-softness`), aplicada uma vez quando o ícone é rasterizado no atlas, sem custo por quadro. O texto SHALL manter a borda da própria fonte.

#### Scenario: Engrenagem na barra superior
- **WHEN** a barra superior desenha a engrenagem e o botão de entradas em 1920x1080
- **THEN** a borda dos traços se espalha pela largura de `--edge-softness`, com a mesma tinta total do glifo da fonte

#### Scenario: Texto ao lado
- **WHEN** o relógio e as abas são desenhados na mesma barra
- **THEN** os glifos do texto têm o tamanho e a borda que a fonte rasteriza, sem a suavização dos ícones

### Requirement: Degradês sem faixas de cor
Os degradês desenhados pela interface (preenchimentos com degradê e o papel de parede, inclusive escurecido nas telas de texto) SHALL ser desenhados com dither de no máximo meio nível de cor (0,5 em 255), calculado na resolução do buffer de renderização, sem passada extra de tela cheia. O dither SHALL NOT ser perceptível como textura e SHALL NOT mudar a cor média.

#### Scenario: Papel de parede escurecido
- **WHEN** a aba Apps desenha o papel de parede escurecido sobre `--bg`
- **THEN** o degradê não forma faixas: a cor média de cada região de 16x16 px difere no máximo 1 em 255 da mesma região desenhada sem dither

#### Scenario: Custo do dither
- **WHEN** a navegação contínua pela grade de Apps é medida na TCL
- **THEN** a fração de quadros de 33 ms ou mais continua em no máximo 1%, como exige `resource-efficiency`

