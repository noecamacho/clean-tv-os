# Spec Delta

## Purpose

Fixa o design system do frontend do Lume (tokens, tipografia, medidas, componentes, foco, movimento e regras de desenho) para que toda tela nova ou alterada siga as mesmas regras. A fonte dos valores é `themes/lume/theme.css` (cores, formas, texto e estados) e as constantes de layout do código; esta capacidade registra as regras que os dois devem obedecer.

## ADDED Requirements

### Requirement: Fonte única dos valores visuais
Todo valor visual de cor, forma, texto e estado SHALL vir de `themes/lume/theme.css` (variáveis e componentes), e nenhuma tela SHALL fixar no código uma cor, raio, peso ou tamanho de fonte que o tema já nomeia. Medidas de layout (posições, tamanhos de tile, dock, margens) SHALL ficar em constantes nomeadas no módulo da tela. Uma tela nova SHALL reaproveitar os componentes existentes (`.tile`, `.settings-item`, `.panel`, `.chip`, `.button`, `.button-secondary`, `.button-danger`, `.icon-button`, `.breadcrumb`, `.toast`) antes de criar outro; um componente novo SHALL ser registrado no tema e nesta spec no mesmo change.

#### Scenario: Cor nova
- **WHEN** uma tela precisa de uma cor que o tema não tem
- **THEN** a cor entra como variável em `:root` do tema, com o papel dela, e a tela usa a variável

#### Scenario: Componente existente
- **WHEN** uma tela precisa de um cartão, um botão redondo ou uma linha de lista
- **THEN** ela usa `.panel`, `.icon-button` ou `.settings-item`, em vez de desenhar caixas com valores próprios

### Requirement: Tokens de cor por papel
O design system SHALL nomear as cores por papel. Superfícies: `--bg` `#0F171E`, `--surface` `#121C25`, `--surface-row` `#17222C`, `--surface-focus` `#1F3040`, `--surface-icon` `#1A252F`, `--control` `#2A3A48`, `--control-disabled` `#1D2A35`, `--border` `#2F4252`. Texto: `--text` (igual a `--secondary`, `#F2F5F7`), `--text-soft` `#DDE3E8`, `--text-muted` `#AEB9C2`, `--text-disabled` `#59626A`. Papéis: `--primary` `#4DB3FF` (foco e controle), `--on-primary` `#0F171E`, `--secondary` `#F2F5F7` (neutra); `--accent` e `--on-accent` são apelidos de compatibilidade. Perigo: `--danger-bg` `#4A2630`, `--danger-fg` `#FFB4BE`. Cores de marca dos tiles (`--brand-*`) valem só para apps conhecidos. O texto sobre o papel de parede SHALL manter contraste de pelo menos 4,5:1.

#### Scenario: Cor de foco
- **WHEN** qualquer elemento recebe foco
- **THEN** o contorno, o sublinhado da barra ou o preenchimento de destaque usa `--primary`, e o nome do app em foco fica em `--text`

#### Scenario: Contraste
- **WHEN** o texto mudo (`--text-muted`) é desenhado sobre o ponto mais claro do papel de parede de uma tela
- **THEN** a razão de contraste é de pelo menos 4,5:1

### Requirement: Tipografia
O texto SHALL usar a família Plus Jakarta Sans embutida, nos pesos 500, 700 e 800, e nenhum outro. Os tamanhos de referência no canvas lógico SHALL ser: 28 (marca na barra, peso 800), 24 (linhas do editor), 22 (títulos de seção, itens de Configurações, texto dos tiles), 20 (abas, relógio, texto base), 18 (botões, balão do dock, aviso), 17 (caminho de navegação), 16 (rótulo de tile, chip, painel, rótulo e valor do painel de estatísticas) e 15 em caixa alta com espaçamento de 1,2 (cabeçalhos de cartão). O texto SHALL NOT usar tamanho menor que 14 (`MIN_FONT_SIZE`). O nome do app sob um tile da grade de Apps SHALL seguir o comportamento da aba: sem foco, cor `--text-muted` e peso 500; com o foco, cor `--text` e peso 700, com a transição de 150 ms. Texto que não cabe SHALL terminar em reticências, exceto o nome no balão do dock, que SHALL aparecer inteiro enquanto couber na área segura.

#### Scenario: Peso fora do conjunto
- **WHEN** um componente pede peso 600
- **THEN** o build do tema avisa, e a tela não usa esse peso

### Requirement: Canvas, margens e grade
O layout SHALL ser desenhado no canvas lógico de 1280x720 (ver `display-adaptation`), com margem lateral de 64 e área segura de 32 na horizontal e 18 na vertical. A barra superior tem 88 de altura. Botões redondos da barra (entradas e engrenagem) têm 48 de diâmetro, **fundo transparente** (só o ícone aparece; o anel azul surge em foco), com 8 de espaço entre si, e o relógio termina em x=1216. Telas secundárias começam em x=64 com largura de 1152; nelas o caminho de navegação fica na linha-base 52 (a tela raiz, Configurações, mostra só "Configurações"), o título (40, peso 800) na 116 (64 abaixo do caminho), o subtítulo (18) na 154 e o conteúdo começa em y=212 (`page::CONTENT_TOP`). A versão instalada, quando mostrada, fica no canto inferior direito da área segura, em texto mudo de 17. A grade de Apps tem 6 colunas, espaço de 24 e tiles 16:9. A tela de Configurações divide a largura de 1152 em duas colunas iguais de 552, com 48 de vão: os itens (72 de altura, passo 88) na esquerda e o painel de estatísticas, alinhado ao primeiro item, na direita (x=664). No editor, o painel de favoritos tem enchimento de 32, 48 entre o cabeçalho e os cartões, cartões de 168 com 24 de espaço, 32 entre os cartões e "Adicionar app", chips com 16 de espaço entre linhas e 40 entre o painel e "Restaurar padrão". O dock tem ícones de 64, espaço de 14, enchimento de 16 e pico de ampliação de 1,6. Valores que mudam SHALL ser alterados na constante do módulo, e esta spec SHALL ser atualizada junto.

#### Scenario: Botão novo na barra
- **WHEN** um botão é adicionado à barra superior
- **THEN** ele tem 48 de diâmetro, fica 56 à esquerda do vizinho e não desloca o relógio

#### Scenario: Painel ao lado dos itens
- **WHEN** a tela de Configurações é exibida em 1280x720
- **THEN** a lista ocupa x de 64 a 616 e o painel de 664 a 1216, em colunas iguais

### Requirement: Botões
O design system SHALL ter três botões de 48 de altura, raio `--radius`, texto 18 peso 700 e enchimento horizontal de 28. O **primário** (`.button`) é a ação principal da tela e se comporta como uma aba: sem foco fica apagado (fundo `--control`, texto `--text-muted`) e, com o foco, acende, preenchido com `--primary` (a cor do logo do Lume) e texto `--on-primary`; como todo foco do Lume, o dele SHALL ser um anel azul `--primary` de 3 px, afastado do preenchimento por `outline-offset` 4 (o vão escuro deixa o anel visível) e com escala 1,04. O **secundário** (`.button-secondary`) é a alternativa discreta, com fundo `--surface`, contorno de 2 px `--border` e texto `--text`; em foco, fundo `--surface-focus`, anel `--primary` de 3 px e escala 1,04. O **de perigo** (`.button-danger`) confirma o que não se desfaz, com fundo `--danger-bg` e texto `--danger-fg`, e em foco o mesmo anel azul afastado do primário. Cada tela SHALL ter no máximo um botão primário visível. "Concluído" SHALL ser primário, "Restaurar padrão" e "Cancelar" secundários e "Restaurar" (na confirmação) de perigo. Botões SHALL ser desenhados pelo componente do tema, nunca com cores no código.

#### Scenario: Concluído
- **WHEN** a tela Editar tela Início é exibida
- **THEN** "Concluído" aparece apagado (cinza-azulado com texto mudo) enquanto o foco está em outro lugar e, quando recebe o foco, acende com o azul do logo (`#4DB3FF`), texto escuro e o anel azul afastado de 4 px

#### Scenario: Restaurar padrão
- **WHEN** a tela Editar tela Início é exibida
- **THEN** "Restaurar padrão" aparece como botão secundário contornado, e em foco ganha anel azul e fundo de foco

#### Scenario: Confirmação
- **WHEN** o diálogo "Restaurar a tela Início padrão?" é exibido
- **THEN** "Restaurar" é o botão de perigo e "Cancelar" o secundário, com o foco inicial em "Cancelar"

### Requirement: Itens de configuração
Os itens da tela de Configurações (`.settings-item`) SHALL seguir o comportamento do botão primário: sem foco ficam apagados (fundo `--surface-row`, texto e seta `--text-muted`) e, com o foco, acendem, preenchidos com `--primary`, texto e seta `--on-primary` e anel azul de 3 px afastado de 4, sem ampliar a escala. A seta à direita SHALL acompanhar a cor do texto.

#### Scenario: Item em foco
- **WHEN** o foco chega a "Tela Início" em Configurações
- **THEN** o item acende em azul com texto escuro, e "Sobre o Lume" permanece apagado

### Requirement: Foco e movimento
O foco SHALL ser sempre visível e azul primário (sem exceção, inclusive nos botões preenchidos, que afastam o anel do fundo): contorno de 4 px (`--focus-ring`) em tiles, itens e linhas; 3 px em chips, botões e botões redondos, com `outline-offset` de 2 a 3. Tiles ampliam 1,05 e botões redondos 1,08, com `--focus-shadow` nos tiles. As abas não têm contorno: o foco é o sublinhado de 2 px. As transições de foco SHALL durar 150 ms com curva ease-out (`--focus-time`); a troca de aba SHALL ter fade de 180 ms; o dock SHALL deslizar com a animação de Magnification. Um elemento sem ação (como o painel de estatísticas) SHALL NOT receber foco nem ter animação. Uma animação cujo valor de repouso é 0 SHALL receber o valor inicial em `animate_to`.

#### Scenario: Foco em botão redondo
- **WHEN** o foco chega ao botão de entradas ou à engrenagem
- **THEN** o botão ganha contorno azul de 3 px e cresce 1,08 em 150 ms

#### Scenario: Cartão sem foco
- **WHEN** o usuário navega pela tela de Configurações
- **THEN** o foco passa só pelos itens da lista, nunca pelo cartão de estatísticas

### Requirement: Superfícies e papel de parede
O fundo de toda tela SHALL ser o papel de parede de aurora (ver `wallpaper`) sobre `--bg`; as telas de texto o desenham escurecido no próprio desenho do papel (`Painter::wallpaper(true)`). Cartões e painéis SHALL usar `.panel` (`--surface`, raio `--radius` de 12). O dock é translúcido (`rgba(15,23,30,.55)`, raio 28, contorno de 1 px a 14% de branco), e o nome do favorito em foco, acima dele, tem fundo transparente: só o texto (`--text`, 18, peso 700), sem balão nem sombra. O desenho SHALL NOT usar nenhuma camada translúcida de tela cheia, exceto o escurecimento do diálogo de confirmação, que só existe enquanto o diálogo está aberto.

#### Scenario: Escurecimento
- **WHEN** a tela de Configurações, Sobre ou Editar é desenhada
- **THEN** o escurecimento vem do desenho do papel de parede, sem camada translúcida por cima

### Requirement: Ícones, marca e texto de interface
Os ícones de interface SHALL vir do subconjunto Lucide embutido (`scripts/subset-fonts.py`), com 27 de tamanho nos botões redondos e 24 nos indicadores de lista; um ícone novo SHALL ser adicionado ao script e regerado, nunca desenhado à mão. O nome do produto SHALL ser escrito "Lume" em todo texto de interface, doc e spec; identificadores técnicos (pacote, crate, biblioteca) ficam em minúsculas. Todo texto de interface SHALL estar em `strings.xml` (en e pt-BR), nunca no código.

#### Scenario: Ícone novo
- **WHEN** uma tela precisa de um ícone que o subconjunto não tem
- **THEN** o nome do glifo Lucide entra em `ICONS` de `subset-fonts.py`, o script é rodado e a constante nasce em `icons.rs`

#### Scenario: Texto novo
- **WHEN** uma tela ganha um rótulo
- **THEN** ele entra em `values/strings.xml` e `values-pt-rBR/strings.xml`, e a tela o lê pelo tradutor

### Requirement: Mudança visual segue o design system
Todo change que cria ou altera tela, componente ou token SHALL ler esta spec antes de implementar, SHALL respeitar os requisitos acima e, se precisar divergir ou acrescentar, SHALL incluir a atualização desta spec no mesmo change. Uma captura da tela por `lume-desktop --screenshot` em 1280x720 SHALL ser conferida contra estas regras antes de dar a tarefa visual por concluída.

#### Scenario: Tela nova
- **WHEN** um change adiciona uma tela
- **THEN** as tarefas do change citam os componentes e tokens desta spec, e a captura de 1280x720 é anexada à verificação

#### Scenario: Divergência
- **WHEN** a implementação precisa de um valor que contraria esta spec
- **THEN** o change atualiza esta spec com a decisão, em vez de deixar o código divergir em silêncio
