## MODIFIED Requirements

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
