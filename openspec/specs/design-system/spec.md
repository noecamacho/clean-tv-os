# design-system Specification

## Purpose
Fixa o design system do frontend do Lume (tokens, tipografia, medidas, componentes, foco, movimento e regras de desenho) para que toda tela nova ou alterada siga as mesmas regras. A fonte dos valores é `themes/lume/theme.css` (cores, formas, texto e estados) e as constantes de layout do código; esta capacidade registra as regras que os dois devem obedecer.

## Variante tvOS 26 desta cópia
Nesta variante pessoal, o foco `--primary` é branco-gelo (`#EAF4FF`), o dock usa um gradiente translúcido com borda branca e refração de borda a partir de uma amostra do papel de parede, e os favoritos são quadrados arredondados por padrão. A barra superior revelada e o painel do Centro de controle também usam a textura do papel de parede sob uma camada translúcida e uma borda fina. O foco dura 180 ms com curva `cubic-bezier(0.2, 0, 0, 1)`. HOME não mostra logo nem marca na barra, nas telas internas ou na abertura. Essas escolhas substituem as referências antigas a azul, dock de fundo sólido, favoritos redondos, logo e foco de 150 ms nos requisitos abaixo. O efeito de vidro usa a textura do papel de parede; não desfoca nem refrata vídeo ou outros conteúdos.

## Requirements

### Requirement: Fonte única dos valores visuais
Todo valor visual de cor, forma, texto e estado SHALL vir de `themes/lume/theme.css` (variáveis e componentes), e nenhuma tela SHALL fixar no código uma cor, raio, peso ou tamanho de fonte que o tema já nomeia. Medidas de layout (posições, tamanhos de tile, dock, margens) SHALL ficar em constantes nomeadas no módulo da tela. Uma tela nova SHALL reaproveitar os componentes existentes (`.app`, `.topbar`, `.brand`, `.tab`, `.clock`, `.icon-button`, `.button`, `.button-secondary`, `.button-danger`, `.section-title`, `.tile`, `.tile-label`, `.dock`, `.dock-label`, `.editor-row`, `.chip`, `.panel`, `.breadcrumb`, `.settings-item`, `.menu`, `.menu-item`, `.menu-note`, `.menu-button`, `.menu-button-danger`, `.keycap`, `.toast`, `.page-title`, `.page-subtitle`, `.card-heading`, `.about-heading`, `.about-body`, `.about-license`) antes de criar outro; um componente novo SHALL ser registrado no tema e nesta spec no mesmo change.

#### Scenario: Cor nova
- **WHEN** uma tela precisa de uma cor que o tema não tem
- **THEN** a cor entra como variável em `:root` do tema, com o papel dela, e a tela usa a variável

#### Scenario: Componente existente
- **WHEN** uma tela precisa de um cartão, um botão redondo ou uma linha de lista
- **THEN** ela usa `.panel`, `.icon-button` ou `.settings-item`, em vez de desenhar caixas com valores próprios

#### Scenario: Lista de ações sobre um item
- **WHEN** uma tela precisa oferecer ações sobre o item em foco
- **THEN** ela usa o menu de contexto (`.menu`, `.menu-item`), em vez de criar outro painel de opções

### Requirement: Tokens de cor por papel
O design system SHALL nomear as cores por papel. Superfícies: `--bg` `#0F171E`, `--surface` `#121C25`, `--surface-row` `#17222C`, `--surface-focus` `#1F3040`, `--surface-icon` `#1A252F`, `--control` `#2A3A48`, `--control-disabled` `#1D2A35`, `--border` `#2F4252`. Texto: `--text` (igual a `--secondary`, `#F2F5F7`), `--text-soft` `#DDE3E8`, `--text-muted` `#AEB9C2`, `--text-disabled` `#59626A`. Papéis: `--primary` `#4DB3FF` (foco e controle), `--on-primary` `#0F171E`, `--secondary` `#F2F5F7` (neutra); `--accent` e `--on-accent` são apelidos de compatibilidade. Perigo: `--danger-bg` `#4A2630`, `--danger-fg` `#FFB4BE` (só sobre superfícies escuras). Cartões claros de sobreposição (o menu de contexto e, no mapeamento de controle, o modal de escuta de tecla e o diálogo de confirmação): `--menu-surface` `#F2F5F7`, `--menu-text` `#1B2630`, `--menu-text-muted` `#5E6A75` (contraste de 5,0:1 sobre o cartão; `--menu-text`, 14:1) e `--menu-danger` `#B3261E` (perigo sobre o cartão claro, contraste de 5,97:1). Cores de marca dos tiles (`--brand-*`) valem só para apps conhecidos. O texto sobre o papel de parede SHALL manter contraste de pelo menos 4,5:1.

#### Scenario: Cor de foco
- **WHEN** qualquer elemento recebe foco
- **THEN** o contorno, o sublinhado da barra ou o preenchimento de destaque usa `--primary`, e o nome do app em foco fica em `--text`

#### Scenario: Contraste
- **WHEN** o texto mudo (`--text-muted`) é desenhado sobre o ponto mais claro do papel de parede de uma tela
- **THEN** a razão de contraste é de pelo menos 4,5:1

#### Scenario: Perigo sobre o cartão claro
- **WHEN** o aviso de tecla reservada (`--menu-danger`) ou o botão "Restaurar" do diálogo de confirmação é desenhado sobre `--menu-surface`
- **THEN** a razão de contraste é de pelo menos 4,5:1, conferida por teste do tema

### Requirement: Tipografia
O texto SHALL usar a família Plus Jakarta Sans embutida, nos pesos 500, 700 e 800, e nenhum outro. A escala de referência no canvas lógico SHALL ter 6 níveis, todos de tamanho inteiro e par (para cair em pixel inteiro a 1,5x): 40 (título de tela, peso 800), 28 (marca na barra, peso 800; relógio, peso 500), 22 (abas, títulos de seção, itens de Configurações, texto dos tiles, linhas do editor, títulos da tela Sobre), 20 (texto base e corpo da tela Sobre), 18 (botões, balão do dock, aviso, subtítulo, caminho de navegação, versão, rótulo de tile, chip, painel e valores do painel de estatísticas) e 16 (licenças de terceiros e cabeçalhos de cartão, estes em caixa alta com espaçamento de 1,2). Cada papel SHALL usar um só nível. Os tamanhos, pesos e espaçamentos de letras SHALL ser tokens do tema (`--font-title`, `--font-brand`, `--font-primary`, `--font-body`, `--font-label`, `--font-caption`) e nenhuma tela SHALL fixá-los no código. O texto SHALL NOT usar tamanho menor que 16 (`MIN_FONT_SIZE`). O nome do app sob um tile da grade de Apps SHALL seguir o comportamento da aba: sem foco, cor `--text-muted` e peso 500; com o foco, cor `--text` e peso 700, com a transição de 150 ms. Texto que não cabe SHALL terminar em reticências, exceto o nome no balão do dock, que SHALL aparecer inteiro enquanto couber na área segura.

#### Scenario: Peso fora do conjunto
- **WHEN** um componente pede peso 600
- **THEN** o build do tema avisa, e a tela não usa esse peso

#### Scenario: Tamanho fora da escala
- **WHEN** um componente do tema pede um tamanho que não é um dos 6 níveis (por exemplo 17)
- **THEN** o build do tema avisa, e a tela usa o nível mais próximo

### Requirement: Canvas, margens e grade
O layout SHALL ser desenhado no canvas lógico de 1280x720 (ver `display-adaptation`), com margem lateral de 64 e área segura de 32 na horizontal e 18 na vertical. A barra superior segue o requisito "Barra superior". Telas secundárias começam em x=64 com largura de 1152; nelas o caminho de navegação fica na linha-base 52 (a tela raiz, Configurações, mostra só "Configurações"), o título (40, peso 800) na 116 (64 abaixo do caminho) e, quando a tela possui subtítulo (18, linha-base 152), o conteúdo começa em y=212 (`page::CONTENT_TOP`). Nas telas secundárias sem subtítulo, o conteúdo começa em y=152 no editor (`editor::TOP`, 36 abaixo da linha-base do título) ou usa máscara de corte em y=152 com o texto começando em y=192 e passo de rolagem de 96 nas telas Sobre o Lume e Informações da TV. A versão instalada, quando mostrada, fica no canto inferior direito da área segura, em texto mudo de 18. A grade de Apps tem 6 colunas, espaço de 24 e tiles 16:9. A tela de Configurações divide a largura de 1152 em duas colunas iguais de 552, com 48 de vão: os itens (72 de altura, passo 88) na esquerda e o painel de estatísticas ("Status da TV", 312 de altura), alinhado ao primeiro item, na direita (x=664, cabeçalho `.card-heading` em y+24, três linhas de métricas em y+72+i·80 com rótulo 18/500 `--text-muted` à esquerda, valor 18/700 `--text` à direita e barra de 8 px de altura e raio 4 em top+36). No editor, o conteúdo em y=152 abre com o painel de aparência (`.panel`, enchimento lateral de 32 e vertical de 24, largura útil interna de 1088 em x=96), que reúne o cabeçalho "APARÊNCIA" em `--primary` à esquerda e o indicador `"1 / 6 ›"` (16/700, `--text-muted`) à direita, 32 acima da faixa horizontal de mini-cards de papel de parede (178x100, vão 16), um divisor de 1 px (`--surface-row`) 24 abaixo dos mini-cards e, 16 abaixo do divisor, a linha "Favoritos redondos" (`.editor-row`, 64 de altura, largura 1088, chave Liga/Desliga de 48x28 à direita com bolinha de 20). A 24 abaixo do painel de aparência fica o painel de favoritos (`.panel`, enchimento lateral de 32 e vertical de 24): cabeçalho "ORDEM DOS FAVORITOS" em `--primary` à esquerda e dica (16/500, `--text-muted`) à direita, linha de descrição (48 de altura, texto 16/500 `--text-muted`), cartões de 168x116 (56 de faixa de nome colorida + 60 de faixa de controles em `--surface-row` com 3 botões redondos de 44x44) com 16 de espaço (6 por linha, preenchendo os 1088 úteis), 20 entre os cartões e "Adicionar app", chips de 40 de altura (raio 20) com 16 de espaço entre linhas e 24 abaixo do painel até o botão secundário "Restaurar padrão" (48 de altura, em x=64, precedido por uma linha divisória de 2 px em y-12). O dock tem ícones de 64, espaço de 16, enchimento de 16 e pico de ampliação de 1,6. Valores que mudam SHALL ser alterados na constante do módulo, e esta spec SHALL ser atualizada junto.

#### Scenario: Botão novo na barra
- **WHEN** um botão é adicionado à barra superior
- **THEN** ele tem 48 de diâmetro, glifo de 36, fica 64 à esquerda do vizinho (16 de vão) e segue a tabela do requisito "Barra superior"

#### Scenario: Painel ao lado dos itens
- **WHEN** a tela de Configurações é exibida em 1280x720
- **THEN** a lista ocupa x de 64 a 616 e o painel de estatísticas ocupa x de 664 a 1216 com 312 de altura, em colunas iguais

#### Scenario: Seis cartões por linha no editor
- **WHEN** a tela Editar tela Início é exibida em 1280x720 com 6 favoritos
- **THEN** os 6 cartões de 168 com vão de 16 cabem na mesma linha, preenchendo os 1088 de largura útil do painel sem sobra à direita

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

- **Unidade de 4.** Toda posição, tamanho, enchimento, espaço, raio e linha-base SHALL ser múltiplo de 4. O espaço entre blocos de uma tela (seções, cartões, grupos de botões) SHALL ser múltiplo de 8. Série de espaços: 4, 8, 12, 16, 24, 32, 40, 48, 64. Exceções, registradas nesta spec: medida derivada de proporção ou de animação (tile 16:9 de 172x96,75, desenhado com 97; ampliação do dock), espessuras de linha (sublinhado de 2, divisor de 1 e 2, anéis de foco de 3 e 4 e seus afastamentos) e correção ótica pela tinta (marca em x=118, para a tinta ficar a 16 do logo). Os testes `the_layout_is_on_the_ruler` (um por módulo de tela) e `the_component_boxes_are_on_the_ruler` (tema) conferem a unidade.
- **Colunas.** O conteúdo vai de x=64 a x=1216 (1152 de largura). Divisões permitidas: 6 colunas de 172 com vão de 24 (grade de Apps), 2 colunas de 552 com vão de 48 (Configurações) e 2 colunas de ficha técnica em Informações da TV (rótulos de x=64 a x=416 e valores de x=448 a x=1216). Todo bloco SHALL começar e terminar numa borda dessas colunas; o texto e o ícone dentro de um bloco alinham à borda do bloco mais o enchimento do componente.
- **Linhas fixas.** Barra superior de 0 a 88, eixo 44, linha-base 52 (requisito "Barra superior"). Tela secundária com subtítulo: caminho na linha-base 52, título na 116, subtítulo na 152, conteúdo a partir de y=212. Tela secundária sem subtítulo: caminho na 52, título na 116, conteúdo a partir de y=152 (Editar tela Início) ou janela de corte em y=152 com texto a partir de y=192 (Sobre o Lume e Informações da TV). Tela principal sob a barra (Apps): título de seção na linha-base 116 e conteúdo a partir de y=144. O conteúdo termina pelo menos 40 acima do fim da área segura.
- **Tamanhos de componente.** Alturas: 40 (chip e keycap), 48 (botões e botões redondos), 64 (linha do editor `.editor-row`, passo 72), 72 (item de Configurações `.settings-item`, passo 88); ícones: glifo de 24 (setas e ícones dentro de componentes), 36 (barra superior) e 64 (dock); raio 12 (cartões, botões, itens) ou metade da altura (chip, keycap, botão redondo); enchimento lateral de 24 a 32 em componentes de texto e de 32 em cartões e painéis. Espaço entre ícone e texto num componente: 8; entre itens de um grupo de botões, botões redondos, keycaps ou chips: 16.
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

### Requirement: Botões
O design system SHALL ter três botões de 48 de altura, raio `--radius`, texto 18 peso 700 e enchimento horizontal de 28 para superfícies escuras, além dos equivalentes `.menu-button` e `.menu-button-danger` para cartões claros. O **primário** (`.button`) é a ação principal de uma tela e se comporta como uma aba: sem foco fica apagado (fundo `--control`, texto `--text-muted`) e, com o foco, acende, preenchido com `--primary` (a cor do logo do Lume) e texto `--on-primary`; como todo foco do Lume em superfície escura, o dele SHALL ser um anel azul `--primary` de 3 px, afastado do preenchimento por `outline-offset` 4 (o vão escuro deixa o anel visível) e com escala 1,04. O **secundário** (`.button-secondary`) é a alternativa discreta, com fundo `--surface`, contorno de 2 px `--border` e texto `--text`; em foco, fundo `--surface-focus`, anel `--primary` de 3 px e escala 1,04. O **de perigo** (`.button-danger`) confirma o que não se desfaz em superfície escura, com fundo `--danger-bg` e texto `--danger-fg`, e em foco o mesmo anel azul afastado do primário. Cada tela SHALL ter no máximo um botão primário visível; nas telas de configuração em que as alterações são salvas imediatamente (Editar tela Início e Mapeamento de controle), não há botão "Concluído" e o retorno é feito pelo botão Voltar. "Restaurar padrão", "Restaurar padrão de fábrica" e "Cancelar" (no diálogo escuro do editor) SHALL ser secundários (`.button-secondary`), e "Restaurar" (no diálogo escuro do editor, 640x280 em `--surface-row` com contorno de 2 px `--border` e escurecimento `rgba(0,0,0,170)` de tela cheia) SHALL ser de perigo (`.button-danger`). No diálogo claro de Mapeamento de controle, os botões usam `.menu-button` e `.menu-button-danger` (ver "Cartões claros de sobreposição"). Botões SHALL ser desenhados pelo componente do tema, nunca com cores no código.

#### Scenario: Concluído
- **WHEN** a tela Editar tela Início ou Mapeamento de controle é exibida
- **THEN** o rodapé exibe apenas o botão secundário de restauração à esquerda (`x = 64`), sem botão "Concluído", pois toda alteração é aplicada imediatamente e o retorno é pelo botão Voltar

#### Scenario: Restaurar padrão
- **WHEN** a tela Editar tela Início é exibida
- **THEN** "Restaurar padrão" aparece como botão secundário contornado, e em foco ganha anel azul e fundo de foco

#### Scenario: Confirmação
- **WHEN** o diálogo "Restaurar a tela Início padrão?" é exibido no editor
- **THEN** o cartão escuro de 640x280 aparece centralizado sobre a camada escura de tela cheia, com "Restaurar" como botão de perigo (`.button-danger`) e "Cancelar" como secundário (`.button-secondary`), com o foco inicial em "Cancelar"

### Requirement: Itens de configuração
Os itens da tela de Configurações (`.settings-item`) SHALL seguir o comportamento do botão primário: sem foco ficam apagados (fundo `--surface-row`, texto e seta `--text-muted`) e, com o foco, acendem, preenchidos com `--primary`, texto e seta `--on-primary` e anel azul de 3 px afastado de 4, sem ampliar a escala. A seta à direita SHALL acompanhar a cor do texto.

#### Scenario: Item em foco
- **WHEN** o foco chega a "Tela Início" em Configurações
- **THEN** o item acende em azul com texto escuro, e "Sobre o Lume" permanece apagado

### Requirement: Menu de contexto
O menu de contexto (botão Menu na tela Início e na grade de Apps, ou OK numa linha mapeada de Mapeamento de controle; ver `launcher-shell` e `remote-mapping`) SHALL ser um cartão claro (`.menu`: `--menu-surface`, raio 20, sombra próxima e em volta do cartão (0 4 24, preto a 45%), que o separa de um tile branco), como o do launcher do Android TV, para se destacar sobre as telas escuras. Ele SHALL ficar junto do item de origem, sem seta e sem escurecer a tela (sem camada translúcida de tela cheia): acima do favorito no dock, ao lado do tile na grade (à direita, ou à esquerda quando não couber) e sob a linha na tela de Mapeamento de controle (alinhado à borda direita da linha, ou acima quando não couber), sempre dentro da área segura.

Os itens (`.menu-item`) SHALL formar uma lista contínua: texto `--menu-text` (18, peso 500, o mesmo tamanho e peso dos nomes dos apps; em 20 e peso 700 o texto escuro sobre o cartão claro parecia pesado) sem foco e, com o foco, preenchimento `--primary` e texto `--on-primary`, sem anel de contorno. Só um item SHALL aparecer em foco por vez. Um item desabilitado SHALL usar `--menu-text-muted` com opacidade 0,6, mostrar o motivo embaixo em `.menu-note` (16, peso 500) e SHALL NOT receber foco. O menu SHALL aparecer e sumir com fade de 150 ms (`--focus-time`) e uma subida de 8 ao aparecer, aplicados só à área do cartão. Todos os textos SHALL vir de `strings.xml`.

Medidas, na régua de 4:

| Medida | Valor | Por quê |
|---|---|---|
| Margem interna do cartão | 8 | com o raio 20 do cartão e o 12 do item, os cantos ficam concêntricos (20 = 12 + 8) |
| Item | 56 de altura, enchimento lateral de 16 | o texto (18) fica a 24 da borda do cartão (pedido do autor: laterais enxutas) |
| Vão entre itens | 0 | itens de uma lista só; sem anel, não há o que afastar |
| Item desabilitado | 72 | nome e motivo centrados pela tinta |
| Largura | a do texto mais longo + 4 de folga, mínimo 200 | a folga evita reticências por arredondamento em 1,5x; o mínimo só vale para menus muito curtos, para que o texto decida a largura e as duas laterais fiquem iguais |
| Distância ao tile (grade) | 8 do anel de foco, com o topo do cartão no topo do anel | o menu fica junto do tile como ele aparece com o foco (ampliado 1,05, com o anel de 4), perto o bastante para se ler como dele |
| Distância ao nome (dock) | 4 acima da caixa do nome do favorito (~18 até as letras) | perto do app, e o nome continua visível e diz a qual app o menu se refere |
| Distância à linha (mapeamento) | 8 abaixo da linha (ou 8 acima se não couber), borda direita alinhada à da linha | fica colado aos keycaps da linha que o abriu, dentro da área segura |

No modo de mover um favorito (`home-screen`), os vizinhos se afastam 24 de cada lado e as setas (chevrons Lucide de 24, cor `--text`) ficam no centro desses vãos.

#### Scenario: Menu sobre o dock
- **WHEN** o menu de contexto de um favorito é aberto
- **THEN** o painel aparece acima do favorito, dentro da área segura, sem escurecer o resto da tela

#### Scenario: Menu na última coluna
- **WHEN** o menu de contexto é aberto sobre um app da sexta coluna da grade
- **THEN** o painel aparece à esquerda do tile

#### Scenario: Menu sob linha de mapeamento
- **WHEN** o menu de contexto é aberto sobre uma linha com teclas em Mapeamento de controle
- **THEN** o painel aparece 8 px abaixo da linha (ou 8 px acima quando perto do rodapé), alinhado à borda direita da linha

#### Scenario: Um item em foco por vez
- **WHEN** o usuário abre o menu, leva o foco ao último item, fecha e abre de novo
- **THEN** só o primeiro item aparece em foco

#### Scenario: Item desabilitado
- **WHEN** o menu mostra "Adicionar ao Início" com o limite atingido
- **THEN** o item aparece apagado, com "Limite de 12 favoritos" em texto menor, e o foco o pula

### Requirement: Foco e movimento
O foco SHALL ser sempre visível e azul primário: contorno de 4 px (`--focus-ring`) em tiles, itens e linhas; 3 px em chips, botões e botões redondos, com `outline-offset` de 2 a 3; os botões preenchidos afastam o anel do fundo. A única exceção ao anel é o item do menu de contexto, em que o preenchimento `--primary` sobre o cartão claro basta (decisão do autor). Tiles ampliam 1,05 e botões redondos 1,08, com `--focus-shadow` nos tiles. As abas não têm contorno: o foco é o sublinhado de 2 px. As transições de foco SHALL durar 150 ms com curva ease-out (`--focus-time`); a troca de aba SHALL ter fade de 180 ms; o dock SHALL deslizar com a animação de Magnification; quando a ordem dos favoritos muda (mover, remover, desinstalar), cada ícone SHALL deslizar do lugar e do tamanho antigos para os novos em 240 ms (ease-in-out), e o removido encolhe e some no lugar; o menu de contexto SHALL aparecer com fade de 150 ms e subida de 8. Um elemento sem ação (como o painel de estatísticas) SHALL NOT receber foco nem ter animação. Uma animação cujo valor de repouso é 0 SHALL receber o valor inicial em `animate_to`.

#### Scenario: Foco em botão redondo
- **WHEN** o foco chega ao botão de entradas ou à engrenagem
- **THEN** o botão ganha contorno azul de 3 px e cresce 1,08 em 150 ms

#### Scenario: Cartão sem foco
- **WHEN** o usuário navega pela tela de Configurações
- **THEN** o foco passa só pelos itens da lista, nunca pelo cartão de estatísticas

### Requirement: Superfícies e papel de parede
O fundo de toda tela SHALL ser o papel de parede de aurora (ver `wallpaper`) sobre `--bg`; as telas de texto o desenham escurecido no próprio desenho do papel (`Painter::wallpaper(true)`). Cartões e painéis SHALL usar `.panel` (`--surface`, raio `--radius` de 12). A exceção é o menu de contexto (`.menu`): um cartão claro (`--menu-surface`, raio 20, sombra próxima e em volta do cartão (0 4 24, preto a 45%), que o separa de um tile branco), o único elemento claro da interface, para se destacar como camada temporária sobre as telas escuras. O dock é translúcido (`rgba(15,23,30,.55)`, raio 28, contorno de 1 px a 14% de branco), e o nome do favorito em foco, acima dele, tem fundo transparente: só o texto (`--text`, 18, peso 700), sem balão nem sombra. O desenho SHALL NOT usar nenhuma camada translúcida de tela cheia, exceto o escurecimento do diálogo de confirmação, que só existe enquanto o diálogo está aberto.

#### Scenario: Escurecimento
- **WHEN** a tela de Configurações, Sobre ou Editar é desenhada
- **THEN** o escurecimento vem do desenho do papel de parede, sem camada translúcida por cima

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

### Requirement: Mudança visual segue o design system
Todo change que cria ou altera tela, componente ou token SHALL ler esta spec antes de implementar, SHALL respeitar os requisitos acima e, se precisar divergir ou acrescentar, SHALL incluir a atualização desta spec no mesmo change. Uma captura da tela por `lume-desktop --screenshot` em 1280x720 SHALL ser conferida contra estas regras antes de dar a tarefa visual por concluída.

#### Scenario: Tela nova
- **WHEN** um change adiciona uma tela
- **THEN** as tarefas do change citam os componentes e tokens desta spec, e a captura de 1280x720 é anexada à verificação

#### Scenario: Divergência
- **WHEN** a implementação precisa de um valor que contraria esta spec
- **THEN** o change atualiza esta spec com a decisão, em vez de deixar o código divergir em silêncio

### Requirement: Splash
O splash SHALL ser só o logo do Lume (o quadrado arredondado com o L e o ponto, nas proporções de `logo.rs`) centralizado sobre `--bg`, com tamanho definido no tema, sem texto. O tema SHALL reservar abaixo do logo uma faixa para um indicador de carregamento futuro (posição e altura em tokens), que nesta versão fica vazia e não desloca o logo. O logo SHALL entrar com fade de 300 ms (ease-out) e crescimento de 0,96 a 1 no primeiro quadro do splash, sobre o mesmo `--bg` da janela inicial do sistema, e SHALL ficar à vista por no mínimo 600 ms (abertura a frio) ou 300 ms (retorno a quente) desde o primeiro quadro antes da revelação. A revelação SHALL ser um progresso linear dividido em etapas que se sobrepõem, cada uma com curva ease-in-out, de modo que logo e interface fiquem parcialmente visíveis ao mesmo tempo, sem um momento vazio entre eles: o papel de parede passa da intensidade 0 (só `--bg`) à intensidade normal, pela própria cor com que é desenhado; o logo some nos primeiros 60% da revelação, com fade e encolhendo até 0,92 do tamanho, aplicado só à área do logo; e a interface entra de 30% a 100% da revelação, com fade pela opacidade de cada elemento (como o fade das abas), com o conteúdo (dock ou grade) subindo 16 unidades e a barra superior parada; o papel de parede sobe de 0 a 80%. A revelação SHALL durar 600 ms na abertura a frio e 360 ms no retorno a quente. Um ícone que chegar com a interface na tela SHALL entrar com fade de 150 ms, ease-out (`--icon-in`), enquanto a cor e a inicial do app saem sob ele. Nenhuma etapa SHALL usar camada translúcida de tela cheia.

#### Scenario: Entrada do logo
- **WHEN** a janela do sistema é substituída pelo primeiro quadro do Lume
- **THEN** o fundo continua da mesma cor e o logo surge com fade, sem saltar

#### Scenario: Revelação sem camada de tela cheia
- **WHEN** a interface é revelada
- **THEN** o papel de parede ganha intensidade pelo próprio desenho, e só a área do logo é desenhada com transparência

### Requirement: Item de Informações da TV nas Configurações
A tela de Configurações SHALL incluir, na coluna esquerda de itens, as opções "Tela Início", "Mapeamento de controle", "Informações da TV" e "Sobre o Lume", respeitando a altura de 72 px e o passo de 88 px na régua.

#### Scenario: Terceiro item de configurações
- **WHEN** a tela de Configurações é desenhada
- **THEN** a lista exibe os itens "Tela Início", "Mapeamento de controle", "Informações da TV" e "Sobre o Lume" alinhados à régua de 4 px

### Requirement: Layout vertical de Informações da TV
A tela Informações da TV SHALL utilizar a largura útil de 1152 px no canvas 1280x720 (`LEFT = 64`), sem subtítulo no cabeçalho, com máscara de corte a partir de `y = 152` (`TV_VIEW_TOP`), texto começando em `y = 192` (`TV_TEXT_TOP`) e passo de rolagem vertical de `96 px` (`TV_STEP`). Cada seção SHALL exibir o título em `.card-heading` (16 px, peso 800, `--text-muted`, altura `32 px`), seguido de linhas de chave/valor com `40 px` de altura (`TV_ROW_HEIGHT`) divididas em duas colunas: rótulo à esquerda (`x = 64`, largura `352 px`, em `.breadcrumb` 18 px peso 500 `--text-muted`) e valor à direita (`x = 448`, `TV_VALUE_X = LEFT + 384`, largura `768 px`, 18 px peso 700 `--text`), com vão de `32 px` (`TV_SECTION_GAP`) entre seções, todas alinhadas à régua de 4 px.

#### Scenario: Rótulo e valor na régua
- **WHEN** as especificações são renderizadas em 1280x720
- **THEN** os títulos de seção têm 32 px de altura, as linhas mantêm 40 px de altura com rótulos em `x = 64` (`--text-muted`, 18/500) e valores em `x = 448` (`--text`, 18/700), e a rolagem corta o texto em `y = 152`

### Requirement: Mini-cards de papel de parede no editor
Os mini-cards de papel de parede no modo edição SHALL ser desenhados dentro do painel de aparência (`.panel` em `x = 64`, largura `1152 px`, largura útil interna `1088 px` a partir de `inner_left = 96 px`) no canvas lógico 1280x720, com proporção aproximada de 16:9, medindo `178 px` de largura por `100 px` de altura, raio de cantos de `12 px` (`--radius`) e vão horizontal de `16 px` entre eles. A faixa SHALL rolar horizontalmente (`Anim::WallpaperScrollX`, transição `--focus-time`) para manter o mini-card focado visível dentro dos `1088 px` úteis do painel, exibindo scrims laterais de `48 px` e setas (`20 px` em `--text-muted`) quando houver conteúdo fora da vista, além do indicador `"<i> / <total>"` no cabeçalho do painel. Cada mini-card SHALL desenhar a miniatura real com scrim escuro nos `36 px` inferiores para legibilidade do título (16 px em `x + 12`, `bottom - 12`); quando ativo, SHALL exibir anel de `2 px` `--primary` e um selo circular de `18x18 px` (`--primary` com ponto branco de `8x8 px`) no canto superior direito.

#### Scenario: Mini-card em repouso
- **WHEN** o mini-card está sem foco e não é o papel de parede ativo
- **THEN** ele exibe a miniatura com anel de 1 px `--control` e o nome em `--text-muted` na escala `--font-caption` (16 px, peso 500)

#### Scenario: Mini-card em foco
- **WHEN** um mini-card de papel de parede recebe o foco do D-Pad
- **THEN** ele é ampliado em `1.05x` com sombra, contornado pelo anel de foco azul `--primary` de `3 px`, e exibe o nome em `--text` com 16 px e peso 700

#### Scenario: Alinhamento horizontal na régua
- **WHEN** o painel de aparência é exibido em 1280x720
- **THEN** os mini-cards começam em `x = 96 px` com passo horizontal de `194 px` (`178 + 16`) dentro da janela útil de `1088 px` do painel e rolam suavemente ao navegar até o último item

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

### Requirement: Layout da tela Mapeamento de controle na régua
A tela "Mapeamento de controle" SHALL ser uma tela secundária no canvas lógico de 1280x720 (`LEFT = 64`, `WIDTH = 1152`): três painéis `.panel` com vão de 24, o primeiro em `y = 212`, e o botão secundário "Restaurar padrão de fábrica" (48) 24 abaixo do último, com as medidas da tabela da decisão 5 do design do change. O foco na tela SHALL ser só vertical.

#### Scenario: Tela Mapeamento de controle na régua
- **WHEN** a tela "Mapeamento de controle" é renderizada em 1280x720 e verificada pelo teste `the_layout_is_on_the_ruler`
- **THEN** todas as posições, larguras, alturas, vãos e enchimentos são múltiplos de 4 px e permanecem dentro da largura útil de 1152 px

#### Scenario: Painéis sem atalhos
- **WHEN** a tela é exibida sem atalhos de configuração nem de aplicativo
- **THEN** os painéis 2 e 3 têm 144 de altura, cada um com só a linha "Adicionar", e o botão "Restaurar padrão de fábrica" termina em `y = 836`

#### Scenario: Todos os atalhos mapeados
- **WHEN** os 20 atalhos de configuração têm tecla
- **THEN** a linha "Adicionar atalho" não é desenhada e o painel tem `72 + 72·20` de altura

#### Scenario: Estrutura dos painéis
- **WHEN** a tela é exibida
- **THEN** o primeiro painel começa em `y = 212` e tem 216 de altura, cada painel tem cabeçalho `.card-heading` em `--primary` (32 de altura, a 12 do topo, centrado entre o topo do painel e a primeira linha) e linhas `.editor-row` de 64 (passo 72, a primeira a 56 do topo, em `x = 96` com 1088 de largura)

#### Scenario: Altura dos painéis de atalhos
- **WHEN** um painel de atalhos tem `n` linhas mapeadas
- **THEN** a altura é `144 + 72·n` com a linha "Adicionar" ou `72 + 72·n` sem ela

### Requirement: Linha de mapeamento e keycap
Uma linha de mapeamento (`.editor-row`) SHALL mostrar o rótulo do alvo à esquerda (22/700, `--text`) e as teclas à direita como `.keycap`s, o último terminando em `x = 1160`. O `.keycap` é um componente do tema, não focável, com largura medida com `p.measure` e texto nunca truncado. Uma ação sobre a linha (adicionar tecla, remover tecla, restaurar padrão, remover atalho) SHALL ser oferecida pelo menu de contexto, sem chips ou botões dentro da linha.

#### Scenario: Keycaps de larguras diferentes
- **WHEN** a linha "Menu de contexto" mostra `Menu 82` e `Favorito 174`
- **THEN** cada keycap tem a largura do seu texto mais 32 de enchimento, com 16 entre eles, e o último termina em `x = 1160`

#### Scenario: Linha em foco
- **WHEN** o foco chega a uma linha de mapeamento
- **THEN** a linha ganha `--surface-focus` e o anel azul de `.editor-row:focus`, o rótulo continua em `--text` e os keycaps não mudam

#### Scenario: Medidas do keycap
- **WHEN** um keycap é desenhado
- **THEN** ele tem 40 de altura, raio 20, enchimento de 16 e fundo `--control`, com o nome da tecla em 18/700 `--text` e o código em 16/500 `--text-muted`, 8 entre os dois, e 16 de vão entre keycaps

#### Scenario: Linha "Adicionar"
- **WHEN** a linha "Adicionar atalho" ou "Adicionar aplicativo" é desenhada
- **THEN** ela tem a caixa de uma linha de mapeamento (64), contorno de 2 `--border`, sem fundo, glifo "+" de 24 e texto 18/700 com 8 de vão, e em foco ganha `--surface-focus` e o anel de `.editor-row:focus`

### Requirement: Cartões claros de sobreposição
O modal de escuta de tecla e o diálogo de confirmação da tela "Mapeamento de controle" SHALL ser cartões claros como o menu de contexto (`--menu-surface`, raio 20, sombra `0 4 24` preto a 45%, texto `--menu-text` e `--menu-text-muted`), centrados no canvas, **sem camada escura de tela cheia** e sem animação contínua, com enchimento de 32 e medidas múltiplas de 4.

#### Scenario: Modal de escuta sobre a tela
- **WHEN** o modal de escuta de tecla é aberto
- **THEN** o cartão claro de 560x188 aparece no centro com sombra, a tela atrás não escurece, e não há botão nem item em foco

#### Scenario: Tecla reservada no modal
- **WHEN** o modal de escuta está aberto e o usuário pressiona uma tecla reservada
- **THEN** a faixa de 116 a 156 mostra o aviso em `--menu-danger`, e a altura e a posição do cartão não mudam

#### Scenario: Diálogo de confirmação claro
- **WHEN** o diálogo "Restaurar o padrão de fábrica?" é exibido
- **THEN** o cartão claro de 560x260 aparece sem camada escura, "Cancelar" está aceso em `--primary` com texto `--on-primary`, e "Restaurar" tem contorno e texto em `--menu-danger`

#### Scenario: Medidas do modal de escuta
- **WHEN** o modal de escuta de tecla é aberto
- **THEN** o cartão claro de 560x188 aparece em `x = 360, y = 264` com sombra, a tela atrás não escurece, e não há botão nem item em foco

#### Scenario: Faixa de dica e aviso do modal
- **WHEN** o modal está aberto
- **THEN** o título "Pressione uma tecla" (28/800) ocupa `y 32 a 76`, o subtítulo "Para: <alvo>" (18/500) `y 76 a 116` e a faixa de `y 116 a 156` mostra a dica "Voltar para cancelar" (18/500, `--menu-text-muted`), que ao pressionar uma tecla reservada troca para o aviso (18/700, `--menu-danger`) sem mudar a altura do cartão

#### Scenario: Medidas do diálogo de confirmação
- **WHEN** o diálogo "Restaurar o padrão de fábrica?" é exibido
- **THEN** o cartão claro de 560x260 aparece em `x = 360, y = 232` sem camada escura, com título (28/800, `y 32 a 76`), corpo de até 2 linhas de 40 (18/500, `y 76 a 156`) e dois botões de 48 (`y 180 a 228`, vão 16, alinhados à direita)

#### Scenario: Botões do diálogo claro
- **WHEN** os botões do diálogo de confirmação são desenhados
- **THEN** "Cancelar" tem contorno de 2 `--menu-text-muted` e texto `--menu-text`, "Restaurar" tem contorno de 2 e texto `--menu-danger`, e o botão em foco tem fundo `--primary`, texto `--on-primary` e nenhum anel, com o foco inicial em "Cancelar"

### Requirement: Tela Escolher aplicativo na régua
A tela "Escolher aplicativo" (`Configurações › Mapeamento de controle › Escolher aplicativo`) SHALL ser uma tela secundária no canvas de 1280x720: caminho na linha-base 52, título "Escolher aplicativo" (40/800) na 116, subtítulo (18) na 152 e a grade de tiles da aba Apps (6 colunas de 172, vão 24, tiles 16:9, ordem alfabética pelo nome mostrado) a partir de `y = 212`, com a mesma rolagem e o mesmo foco da aba Apps, listando só os aplicativos sem atalho.

#### Scenario: Grade de escolha
- **WHEN** a tela "Escolher aplicativo" é exibida em 1280x720
- **THEN** a primeira linha de tiles começa em `y = 212`, os 6 tiles de 172 com vão de 24 ocupam os 1152 da largura útil e o foco está no primeiro tile

### Requirement: Tela Escolher atalho na régua
A tela "Escolher atalho" (`Configurações › Mapeamento de controle › Escolher atalho`) SHALL ser uma tela secundária no canvas de 1280x720, com o cabeçalho das outras telas e, a partir de `y = 212`, uma lista de linhas `.editor-row` de 64 (passo 72, `x = 64`, largura 1152) só com os atalhos sem tecla. O foco SHALL ser só vertical e a lista SHALL rolar para manter a linha em foco visível.

#### Scenario: Lista de escolha
- **WHEN** a tela "Escolher atalho" é exibida em 1280x720
- **THEN** a primeira linha começa em `y = 212`, tem 64 de altura e 1152 de largura, as seguintes vêm a cada 72, e o foco está na primeira

#### Scenario: Lista maior que a tela
- **WHEN** há mais atalhos do que cabem na tela e o foco desce até o último
- **THEN** a lista rola e a linha em foco fica inteira dentro da área visível
