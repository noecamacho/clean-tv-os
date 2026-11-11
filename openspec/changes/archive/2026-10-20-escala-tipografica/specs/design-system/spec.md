## MODIFIED Requirements

### Requirement: Tipografia
O texto SHALL usar a família Plus Jakarta Sans embutida, nos pesos 500, 700 e 800, e nenhum outro. A escala de referência no canvas lógico SHALL ter 6 níveis, todos de tamanho inteiro e par (para cair em pixel inteiro a 1,5x): 40 (título de tela, peso 800), 28 (marca na barra, peso 800; relógio, peso 500), 22 (abas, títulos de seção, itens de Configurações, texto dos tiles, linhas do editor, títulos da tela Sobre), 20 (texto base e corpo da tela Sobre), 18 (botões, balão do dock, aviso, subtítulo, caminho de navegação, versão, rótulo de tile, chip, painel e valores do painel de estatísticas) e 16 (licenças de terceiros e cabeçalhos de cartão, estes em caixa alta com espaçamento de 1,2). Cada papel SHALL usar um só nível. Os tamanhos, pesos e espaçamentos de letras SHALL ser tokens do tema (`--font-title`, `--font-brand`, `--font-primary`, `--font-body`, `--font-label`, `--font-caption`) e nenhuma tela SHALL fixá-los no código. O texto SHALL NOT usar tamanho menor que 16 (`MIN_FONT_SIZE`). O nome do app sob um tile da grade de Apps SHALL seguir o comportamento da aba: sem foco, cor `--text-muted` e peso 500; com o foco, cor `--text` e peso 700, com a transição de 150 ms. Texto que não cabe SHALL terminar em reticências, exceto o nome no balão do dock, que SHALL aparecer inteiro enquanto couber na área segura.

#### Scenario: Peso fora do conjunto
- **WHEN** um componente pede peso 600
- **THEN** o build do tema avisa, e a tela não usa esse peso

#### Scenario: Tamanho fora da escala
- **WHEN** um componente do tema pede um tamanho que não é um dos 6 níveis (por exemplo 17)
- **THEN** o build do tema avisa, e a tela usa o nível mais próximo


### Requirement: Canvas, margens e grade
O layout SHALL ser desenhado no canvas lógico de 1280x720 (ver `display-adaptation`), com margem lateral de 64 e área segura de 32 na horizontal e 18 na vertical. A barra superior segue o requisito "Barra superior". Telas secundárias começam em x=64 com largura de 1152; nelas o caminho de navegação fica na linha-base 52 (a tela raiz, Configurações, mostra só "Configurações"), o título (40, peso 800) na 116 (64 abaixo do caminho), o subtítulo (18) na 152 e o conteúdo começa em y=212 (`page::CONTENT_TOP`). A versão instalada, quando mostrada, fica no canto inferior direito da área segura, em texto mudo de 18. A grade de Apps tem 6 colunas, espaço de 24 e tiles 16:9. A tela de Configurações divide a largura de 1152 em duas colunas iguais de 552, com 48 de vão: os itens (72 de altura, passo 88) na esquerda e o painel de estatísticas, alinhado ao primeiro item, na direita (x=664). No editor, o painel de favoritos tem enchimento de 32, 48 entre o cabeçalho e os cartões, cartões de 168 com 24 de espaço, 32 entre os cartões e "Adicionar app", chips de 40 de altura (raio 20) com 16 de espaço entre linhas e 40 entre o painel e "Restaurar padrão". O dock tem ícones de 64, espaço de 16, enchimento de 16 e pico de ampliação de 1,6. Valores que mudam SHALL ser alterados na constante do módulo, e esta spec SHALL ser atualizada junto.

#### Scenario: Botão novo na barra
- **WHEN** um botão é adicionado à barra superior
- **THEN** ele tem 48 de diâmetro, glifo de 36, fica 64 à esquerda do vizinho (16 de vão) e segue a tabela do requisito "Barra superior"

#### Scenario: Painel ao lado dos itens
- **WHEN** a tela de Configurações é exibida em 1280x720
- **THEN** a lista ocupa x de 64 a 616 e o painel de 664 a 1216, em colunas iguais

### Requirement: Ícones, marca e texto de interface
Os ícones de interface SHALL vir do Lucide, desenhados pelo `scripts/subset-fonts.py` a partir dos SVGs com o traço `LUCIDE_STROKE` (1,75 na grade de 24, em vez dos 2 do Lucide), para que no glifo de 36 da barra o traço tenha a espessura do texto das abas e os ícones pareçam da mesma família da fonte; o tamanho é o do requisito "Régua de layout" (36 na barra superior, 24 dentro de componentes). Um ícone novo SHALL ser adicionado ao script e regerado com o mesmo traço, nunca desenhado à mão. O nome do produto SHALL ser escrito "Lume" em todo texto de interface, doc e spec; identificadores técnicos (pacote, crate, biblioteca) ficam em minúsculas. Todo texto de interface SHALL estar em `strings.xml` (en e pt-BR), nunca no código.

#### Scenario: Ícone novo
- **WHEN** uma tela precisa de um ícone que o subconjunto não tem
- **THEN** o nome do ícone do Lucide entra em `ICONS` de `subset-fonts.py`, o script é rodado (`--icons-only` para não regerar a fonte de texto) e a constante nasce em `icons.rs`

#### Scenario: Peso do ícone
- **WHEN** a barra superior é medida na captura em 1920x1080
- **THEN** o traço dos ícones e o do relógio têm a mesma espessura de tinta que o texto das abas (4 px), com 1 px de tolerância

#### Scenario: Texto novo
- **WHEN** uma tela ganha um rótulo
- **THEN** ele entra em `values/strings.xml` e `values-pt-rBR/strings.xml`, e a tela o lê pelo tradutor

## ADDED Requirements

### Requirement: Barra superior
A barra superior SHALL seguir estas métricas (canvas lógico; "tinta" é o desenho visível, medido na captura, e "caixa" é o retângulo de layout):

| Item | Caixa | Tinta medida | Regra |
|---|---|---|---|
| Barra | 0 a 88, margens de 64 | — | eixo central em y=44 para todos os itens |
| Logo | 40x40 em x=64, y de 24 a 64 | 64 a 103, altura 40 | âncora da ponta esquerda |
| Marca "Lume" | x=118, 28 peso 800, linha-base 56 | começa a 16 do logo | logo e marca formam um bloco só |
| Abas | a primeira em x=236, 32 entre os textos; 22, linha-base 52 | 48 da marca à primeira aba | menores que as pontas: o sublinhado as marca |
| Botões (entradas, engrenagem) | 48 de diâmetro, y de 20 a 68, 16 de vão | glifo de 36 (Lucide redesenhado com traço 1,75), desenho de 30 a 32 (cerca de 80% do logo), centro 44, traço de tinta igual ao das abas (4 px a 1,5x) | o ícone tem o peso do texto das abas; desenho de traço parece maior que o logo cheio, por isso fica menor que ele |
| Relógio | 28 peso 500, linha-base 56, termina em x=1216 | altura de tinta e linha-base da marca; traço igual ao do texto das abas (4 px a 1,5x) | algarismos da fonte padrão (escolha do autor) |
| Engrenagem → relógio | 24 da caixa da engrenagem ao relógio | 34 de tinta, igual ao vão entre os ícones, em qualquer hora | os botões acompanham a largura do relógio |

O relógio SHALL usar a fonte padrão com os algarismos dela, alinhado à direita em x=1216. Os botões SHALL ser posicionados a partir da largura da hora mostrada, para que o vão de tinta entre a engrenagem e o relógio seja sempre o mesmo que entre os dois ícones; eles se movem só quando a largura da hora muda, e nunca chegam às abas. O anel de foco de um botão (3 px, afastado 2, botão ampliado 1,08) SHALL caber na barra e não tocar a tinta do vizinho. Item novo na barra SHALL usar estas métricas, sem tamanho, espaço ou linha-base próprios; se precisar de um, esta tabela é atualizada no mesmo change.

#### Scenario: Mesma escala na barra
- **WHEN** a barra superior é exibida em 1280x720 e medida com `scripts/measure-ink.py`
- **THEN** logo, ícones, marca e relógio têm o centro de tinta em y=44 (1 px de tolerância), os ícones têm de 30 a 32 de altura e traço igual ao das abas, o relógio tem a altura e a linha-base da marca e o traço das abas, e os vãos de tinta são 16 (logo → marca), 48 (marca → abas, 2 px de tolerância pela folga das letras), 34 (ícone → ícone) e 34 (engrenagem → relógio, em qualquer hora)

#### Scenario: Hora mais larga
- **WHEN** o relógio passa de "11:11" para "00:00", ou o formato é de 12 h
- **THEN** o relógio cabe no espaço reservado, nada encosta na engrenagem, e os botões ficam no mesmo lugar

### Requirement: Régua de layout
Toda tela, existente ou nova, SHALL ser montada sobre uma régua única no canvas lógico de 1280x720, para que espaços, alinhamentos e tamanhos sejam os mesmos em todas as telas e uma tela nova saia alinhada de primeira:

- **Unidade de 4.** Toda posição, tamanho, enchimento, espaço, raio e linha-base SHALL ser múltiplo de 4. O espaço entre blocos de uma tela (seções, cartões, grupos de botões) SHALL ser múltiplo de 8. Série de espaços: 4, 8, 12, 16, 24, 32, 40, 48, 64. Exceções, registradas nesta spec: medida derivada de proporção ou de animação (tile 16:9 de 172x96,75, desenhado com 97; ampliação do dock), espessuras de linha (sublinhado de 2, anéis de foco de 3 e 4 e seus afastamentos) e correção ótica pela tinta (marca em x=118, para a tinta ficar a 16 do logo). Os testes `the_layout_is_on_the_ruler` (um por módulo de tela) e `the_component_boxes_are_on_the_ruler` (tema) conferem a unidade.
- **Colunas.** O conteúdo vai de x=64 a x=1216 (1152 de largura). Divisões permitidas: 6 colunas de 172 com vão de 24 (grade de Apps) e 2 colunas de 552 com vão de 48 (Configurações). Todo bloco SHALL começar e terminar numa borda dessas colunas; o texto e o ícone dentro de um bloco alinham à borda do bloco mais o enchimento do componente.
- **Linhas fixas.** Barra superior de 0 a 88, eixo 44, linha-base 52 (requisito "Barra superior"). Tela secundária: caminho na linha-base 52, título na 116, subtítulo na 152, conteúdo a partir de y=212. Tela principal sob a barra (Apps): título de seção na linha-base 116 e conteúdo a partir de y=144. O conteúdo termina pelo menos 40 acima do fim da área segura.
- **Tamanhos de componente.** Alturas: 40 (chip), 48 (botões e botões redondos), 72 (item de Configurações e linha do editor), 88 (passo da lista); ícones: glifo de 24 (setas e ícones dentro de componentes), 36 (barra superior) e 64 (dock); raio 12 (cartões, botões, itens) ou metade da altura (chip, botão redondo); enchimento lateral de 24 a 32 em componentes de texto e de 32 em cartões e painéis. Espaço entre ícone e texto num componente: 8; entre itens de um grupo de botões, botões redondos ou chips: 16.
- **Texto corrido.** A altura de linha SHALL ser múltipla de 4: 32 para a legenda (16), 40 para corpo e rótulo (18 e 20), 44 para o principal (22).
- **Alinhamento.** Itens lado a lado numa linha compartilham o eixo central (ícone, botão, chip) ou a linha-base (textos); textos empilhados numa coluna compartilham a borda esquerda; valores numéricos de uma lista alinham à direita; um cartão ao lado de uma lista alinha o topo ao do primeiro item.
- **Caixa e tinta.** Um espaço é especificado na caixa (o que o código posiciona), mas o que o olho vê é a tinta; quando a folga da fonte ou a margem do glifo faz vãos iguais parecerem diferentes, a caixa é corrigida para que a tinta fique igual (como os ícones da barra, que têm 16 de vão na caixa e 34 na tinta por causa da margem do glifo). Textos lado a lado alinham pela linha-base; ícones e blocos, pelo centro da tinta.
- **Verificação.** Toda tela nova ou alterada SHALL ser medida na captura de 1280x720 com `scripts/measure-ink.py` (`columns` dá os vãos de uma faixa, `rows` dá altura e centro de cada item), e os números conferidos contra esta régua e contra a tabela do requisito da tela.
- **Receita de tela nova.** Escolher o modelo (principal, sob a barra, ou secundária, com cabeçalho); dividir a largura com uma das divisões de colunas; empilhar os blocos a partir da linha de conteúdo com espaços da série; usar só componentes e tokens existentes. Medida nova SHALL entrar nesta spec e numa constante do módulo, no mesmo change, antes da implementação da tela.

#### Scenario: Medida fora da régua
- **WHEN** uma tela usa um espaço de 14, uma altura de 71 ou uma linha-base de 119
- **THEN** a revisão a recusa, e o valor vira o múltiplo de 4 da série mais próximo (16, 72, 116) ou é registrado aqui como exceção derivada

#### Scenario: Tela nova alinhada de primeira
- **WHEN** um change cria uma tela
- **THEN** o design do change escreve a tela em termos desta régua, numa tabela de métricas como a da barra superior (item, caixa, tinta esperada, regra), antes do código, e a medição da captura em 1280x720 confere com a tabela

#### Scenario: Mesma medida em telas diferentes
- **WHEN** o mesmo papel aparece em duas telas (título, conteúdo, cartão, botão)
- **THEN** ele tem a mesma posição, o mesmo tamanho e o mesmo espaço nas duas
