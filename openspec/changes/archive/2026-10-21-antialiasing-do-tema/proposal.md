# Proposal

## Why

Na TCL (painel 4K, buffer do Lume em 1080p), os favoritos redondos do dock e os tiles da grade de Apps mostram bordas serrilhadas, que o scaler da TV realça ao ampliar 2x e aplicar nitidez. As formas já são suavizadas por função de distância nos shaders; o serrilhado vem de três defeitos pontuais que este change corrige:

- **Duas bordas suavizadas no mesmo pixel**: o fundo do item (cor do app ou do tile) e a imagem por cima são recortados pela mesma forma, cada um com a sua rampa; na borda, cerca de 25% da cor do fundo vaza e forma um anel fino (`app/dock.rs`, passadas de caixas e de imagens da grade).
- **Cantos dos banners embutidos no bitmap**: o `LumeBridge.drawBanner` arredonda os cantos numa célula de 256x144, que a GPU amplia 1,24x em repouso e 1,30x com foco em 1080p; a borda do canto fica borrada, desigual e treme na animação do foco. Isso contraria o requisito da `display-adaptation` de gerar formas no tamanho final.
- **Contorno de foco encostado na imagem**: o contorno de 4 px começa exatamente onde a borda do banner termina; as duas coberturas parciais deixam o papel de parede aparecer entre elas como um fio escuro.

## What Changes

- Imagens recortadas (ícones do dock, banners e ícones da grade) passam a ser compostas sobre a cor do item dentro do próprio shader, numa passada, com uma única cobertura de borda.
- Os cantos dos banners passam a ser recortados pelo shader do Lume (`image_rounded`) no tamanho final em pixels. O `LumeBridge.drawBanner` deixa de arredondar: sai o `drawRoundRect`, o `BitmapShader` e o segundo bitmap (`sourceBitmap`). É remoção de Java, autorizada pelo autor; nenhum Java novo entra.
- O contorno de foco passa a cobrir a borda da imagem até ela ficar opaca, sem pixel com duas bordas parciais.
- A rampa de suavização das bordas passa a ter cerca de 1,25 px de buffer (valor final calibrado na TCL), para resistir à nitidez do scaler da TV.
- Os ícones da fonte de ícones (entradas, engrenagem) ganham a mesma rampa ao serem rasterizados; o texto não muda (pedido do autor na implementação).
- Os shaders que desenham degradês (papel de parede e preenchimentos com degradê, como a barra do dock) ganham um dither de cerca de ±0,5 nível pela posição na tela, sem passada extra, para não formar faixas de cor quando a TV aumenta o contraste.
- Em segundo passo, condicionado à medida de memória: as células do atlas (ícones do dock e tiles) passam a ter o tamanho do item exibido no pico (foco ou Magnification) na escala atual, para a imagem não ser ampliada mais de cerca de 1,05x. O tamanho já vem do Rust (`renderTile`, `renderDockIcon`), sem Java.
- Fora de escopo: filtros de tela inteira (FXAA, SMAA, MSAA, nitidez, supersampling), que custam quadros na GPU da TV e borram texto e ícones.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `design-system`: novo requisito de antialiasing para o tema inteiro: uma única cobertura por borda, formas e recortes gerados no tamanho final em pixels, contorno de foco sem fresta, largura da rampa e dither nos degradês, com verificação por captura.
- `display-adaptation`: o requisito de gerar formas no tamanho final passa a valer também para o recorte de imagens, e as imagens de apps passam a ter limite de ampliação (cerca de 1,05x) em 720p e 1080p.

## Impact

- Rust: `render/shaders.rs` (composição sobre a cor do item, rampa, dither), `render/batch.rs` e `render/list.rs` (cor de fundo no vértice da imagem), `app/dock.rs` e `app/mod.rs` (uma passada por item, recorte dos banners, contorno de foco por baixo da imagem), `app_icons.rs` (tamanho das células, no segundo passo).
- Java: `LumeBridge.drawBanner` só perde código (sem `drawRoundRect`, `BitmapShader` e `sourceBitmap`).
- Tema: `themes/lume/theme.css`, `--edge-softness`.
- Testes: capturas no `lume-desktop` em 1280x720 e 1920x1080 de um tile com foco e de um favorito redondo, que reprovam transição de borda acima de 2 px e pixel de borda mais escuro que os dois lados.
- Orçamentos mantidos (`resource-efficiency`): até 1% de quadros de 33 ms ou mais na navegação e PSS até 64 MB; medir nas duas TVs, a TCL e a AIWA, inclusive a memória das células maiores antes de adotá-las.
