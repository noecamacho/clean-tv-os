# Spec Delta

## MODIFIED Requirements

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
