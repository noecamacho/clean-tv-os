# Spec Delta

## Purpose

Define o papel de parede do lume: uma imagem de aurora boreal nas cores da marca, única nesta versão, desenhada atrás de todas as telas, sem depender de imagens de terceiros, nítida em qualquer resolução de TV e preparada para uma futura opção de troca.

## ADDED Requirements

### Requirement: Papel de parede de aurora boreal
O lume SHALL desenhar, atrás do conteúdo de todas as telas (Início, Apps, Configurações, Sobre e edição), um papel de parede de aurora boreal que cobre toda a área visível. A aurora SHALL usar as cores da marca (o azul de destaque do tema, passando ao índigo no alto) sobre o fundo azul-petróleo, com estrelas discretas, e a metade inferior da tela SHALL ser escura para acomodar o dock. A imagem SHALL ser criada pelo próprio projeto, sem uso de imagens, fotos ou arte de terceiros, SHALL poder ser regerada pelo script `tools/wallpaper/aurora06.py` e SHALL ser embutida no aplicativo.

#### Scenario: Papel de parede na tela Início
- **WHEN** a tela Início é exibida
- **THEN** a aurora azul aparece atrás da barra superior e do dock, ocupando a tela inteira, com a parte de baixo escura

#### Scenario: Papel de parede nas outras telas
- **WHEN** o usuário abre a aba Apps, as Configurações, a tela Sobre ou o modo edição
- **THEN** o mesmo papel de parede aparece atrás do conteúdo

#### Scenario: Proporção diferente de 16:9
- **WHEN** a tela tem uma proporção diferente de 16:9
- **THEN** o papel de parede cobre toda a superfície visível, inclusive as sobras laterais ou verticais fora do canvas lógico, sem faixas vazias

### Requirement: Definição em qualquer resolução
O papel de parede SHALL ter a mesma nitidez do resto da interface em qualquer resolução de TV. Como o lume renderiza no máximo em 1920x1080 (ver `display-adaptation`), em telas 4K ele SHALL ficar tão nítido quanto a interface em 1080p. As partes suaves (cortinas, brilhos e fundo) SHALL vir de uma imagem de pelo menos 640x360 pixels, ampliada com filtragem linear, e as partes de detalhe fino (as estrelas) SHALL ser desenhadas na resolução de renderização, e não ampliadas de uma imagem menor. O papel de parede SHALL NOT mostrar faixas de cor visíveis (banding) nem blocos.

#### Scenario: TV 4K
- **WHEN** a superfície de exibição informa 3840x2160 e o lume renderiza em 1920x1080
- **THEN** as estrelas têm 1 ou 2 pixels do buffer de renderização, com bordas nítidas, e as cortinas não mostram degraus nem blocos

#### Scenario: Fidelidade à referência
- **WHEN** o papel de parede é renderizado a 1920x1080 e comparado com a imagem de referência do projeto (`mockup/wallpaper-aurora-lume@1080p.png`)
- **THEN** a diferença média absoluta por canal é de no máximo 2 em 255

#### Scenario: TV 720p
- **WHEN** a superfície tem 1280x720 pixels
- **THEN** o papel de parede é renderizado sem perda perceptível em relação ao de 1080p, com as estrelas em 1 ou 2 pixels

### Requirement: Legibilidade sobre o papel de parede
O papel de parede SHALL ser escurecido sob a barra superior, de modo que o logo, as abas, a engrenagem e o relógio mantenham contraste de pelo menos 4.5:1 com o fundo. O papel de parede SHALL NOT reduzir a legibilidade dos textos de nenhuma tela.

#### Scenario: Barra superior legível
- **WHEN** a parte mais clara da aurora está sob a barra superior
- **THEN** o texto das abas e o relógio continuam com contraste de pelo menos 4.5:1

### Requirement: Escurecimento nas telas de texto
Nas telas com muito texto (Apps, Configurações, Sobre e modo edição), o papel de parede SHALL ser escurecido (cerca de 78% em direção à cor de fundo do tema), sem uma camada translúcida de tela cheia por cima (o próprio desenho do papel de parede já sai escurecido, para não custar quadros na TV), de modo que todo texto sobre ele tenha contraste de pelo menos 4.5:1 em qualquer ponto da aurora. Na tela Início o papel de parede SHALL aparecer sem esse escurecimento.

#### Scenario: Aba Apps
- **WHEN** a aba Apps é exibida e o nome de um app fica sobre a parte mais clara da aurora
- **THEN** o texto do nome tem contraste de pelo menos 4.5:1 com o fundo

#### Scenario: Tela Início
- **WHEN** a tela Início é exibida
- **THEN** a aurora aparece sem o escurecimento das telas de texto

### Requirement: Papel de parede único e não configurável
Nesta versão o lume SHALL ter um único papel de parede, sem opção na interface para trocá-lo, desativá-lo ou carregar uma imagem do usuário. O papel de parede SHALL ser obtido por um único ponto de carregamento, de modo que uma versão futura possa oferecer a troca sem mudar o desenho das telas.

#### Scenario: Sem opção de troca
- **WHEN** o usuário percorre as Configurações do lume e o modo edição
- **THEN** não existe opção para trocar o papel de parede

### Requirement: Custo de recursos do papel de parede
O papel de parede SHALL ocupar no máximo 1 MB de memória gráfica e SHALL adicionar no máximo 1,5 MB ao tamanho do APK de release, somadas as duas arquiteturas. A textura SHALL ser criada só com o lume em primeiro plano e liberada junto com o resto da memória gráfica ao ir para segundo plano. O papel de parede SHALL NOT manter animação contínua nem provocar quadros adicionais com a tela ociosa.

#### Scenario: Segundo plano
- **WHEN** outro app passa ao primeiro plano
- **THEN** a textura do papel de parede é liberada com o contexto gráfico

#### Scenario: Retorno
- **WHEN** o usuário pressiona Home com o lume em segundo plano
- **THEN** o papel de parede é recriado e aparece já no primeiro quadro completo

#### Scenario: Tela ociosa
- **WHEN** a tela Início está visível e sem interação
- **THEN** o papel de parede não provoca nenhum quadro além do relógio
