## ADDED Requirements

### Requirement: Menu de contexto
O menu de contexto (botão Menu, ver `launcher-shell`) SHALL ser um cartão claro (`.menu`: `--menu-surface`, raio 20, sombra como a do dock), como o do launcher do Android TV, para se destacar sobre as telas escuras. Ele SHALL ficar junto do item de origem, sem seta e sem escurecer a tela (sem camada translúcida de tela cheia): acima do favorito no dock e ao lado do tile na grade (à direita, ou à esquerda quando não couber), sempre dentro da área segura.

Os itens (`.menu-item`) SHALL formar uma lista contínua: texto `--menu-text` (20, peso 700) sem foco e, com o foco, preenchimento `--primary` e texto `--on-primary`, sem anel de contorno. Só um item SHALL aparecer em foco por vez. Um item desabilitado SHALL usar `--menu-text-muted` com opacidade 0,6, mostrar o motivo embaixo em `.menu-note` (16, peso 500) e SHALL NOT receber foco. O menu SHALL aparecer e sumir com fade de 150 ms (`--focus-time`) e uma subida de 8 ao aparecer, aplicados só à área do cartão. Todos os textos SHALL vir de `strings.xml`.

Medidas, na régua de 4:

| Medida | Valor | Por quê |
|---|---|---|
| Margem interna do cartão | 8 | com o raio 20 do cartão e o 12 do item, os cantos ficam concêntricos (20 = 12 + 8) |
| Item | 56 de altura, enchimento lateral de 24 | o texto fica a 32 da borda do cartão, perto dos ~28 do topo |
| Vão entre itens | 0 | itens de uma lista só; sem anel, não há o que afastar |
| Item desabilitado | 72 | nome e motivo centrados pela tinta |
| Largura | a do texto mais longo + 4 de folga, mínimo 240 | a folga evita reticências por arredondamento em 1,5x |
| Distância ao tile (grade) | 24 | o tile em foco cresce 1,05 e tem anel de 4: sobram ~16 visíveis |
| Distância ao nome (dock) | 16 acima do nome do favorito | o nome continua visível e diz a qual app o menu se refere |

No modo de mover um favorito (`home-screen`), os vizinhos se afastam 24 de cada lado e as setas (chevrons Lucide de 24, cor `--text`) ficam no centro desses vãos.

#### Scenario: Menu sobre o dock
- **WHEN** o menu de contexto de um favorito é aberto
- **THEN** o painel aparece acima do favorito, dentro da área segura, sem escurecer o resto da tela

#### Scenario: Menu na última coluna
- **WHEN** o menu de contexto é aberto sobre um app da sexta coluna da grade
- **THEN** o painel aparece à esquerda do tile

#### Scenario: Um item em foco por vez
- **WHEN** o usuário abre o menu, leva o foco ao último item, fecha e abre de novo
- **THEN** só o primeiro item aparece em foco

#### Scenario: Item desabilitado
- **WHEN** o menu mostra "Adicionar ao Início" com o limite atingido
- **THEN** o item aparece apagado, com "Limite de 12 favoritos" em texto menor, e o foco o pula

## MODIFIED Requirements

### Requirement: Fonte única dos valores visuais
Todo valor visual de cor, forma, texto e estado SHALL vir de `themes/lume/theme.css` (variáveis e componentes), e nenhuma tela SHALL fixar no código uma cor, raio, peso ou tamanho de fonte que o tema já nomeia. Medidas de layout (posições, tamanhos de tile, dock, margens) SHALL ficar em constantes nomeadas no módulo da tela. Uma tela nova SHALL reaproveitar os componentes existentes (`.tile`, `.settings-item`, `.panel`, `.menu`, `.menu-item`, `.menu-note`, `.chip`, `.button`, `.button-secondary`, `.button-danger`, `.icon-button`, `.breadcrumb`, `.toast`) antes de criar outro; um componente novo SHALL ser registrado no tema e nesta spec no mesmo change.

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
O design system SHALL nomear as cores por papel. Superfícies: `--bg` `#0F171E`, `--surface` `#121C25`, `--surface-row` `#17222C`, `--surface-focus` `#1F3040`, `--surface-icon` `#1A252F`, `--control` `#2A3A48`, `--control-disabled` `#1D2A35`, `--border` `#2F4252`. Texto: `--text` (igual a `--secondary`, `#F2F5F7`), `--text-soft` `#DDE3E8`, `--text-muted` `#AEB9C2`, `--text-disabled` `#59626A`. Papéis: `--primary` `#4DB3FF` (foco e controle), `--on-primary` `#0F171E`, `--secondary` `#F2F5F7` (neutra); `--accent` e `--on-accent` são apelidos de compatibilidade. Perigo: `--danger-bg` `#4A2630`, `--danger-fg` `#FFB4BE`. Menu de contexto (o único cartão claro): `--menu-surface` `#F2F5F7`, `--menu-text` `#1B2630`, `--menu-text-muted` `#5E6A75` (contraste de 5,0:1 sobre o cartão; `--menu-text`, 14:1). Cores de marca dos tiles (`--brand-*`) valem só para apps conhecidos. O texto sobre o papel de parede SHALL manter contraste de pelo menos 4,5:1.

#### Scenario: Cor de foco
- **WHEN** qualquer elemento recebe foco
- **THEN** o contorno, o sublinhado da barra ou o preenchimento de destaque usa `--primary`, e o nome do app em foco fica em `--text`

#### Scenario: Contraste
- **WHEN** o texto mudo (`--text-muted`) é desenhado sobre o ponto mais claro do papel de parede de uma tela
- **THEN** a razão de contraste é de pelo menos 4,5:1

### Requirement: Foco e movimento
O foco SHALL ser sempre visível e azul primário: contorno de 4 px (`--focus-ring`) em tiles, itens e linhas; 3 px em chips, botões e botões redondos, com `outline-offset` de 2 a 3; os botões preenchidos afastam o anel do fundo. A única exceção ao anel é o item do menu de contexto, em que o preenchimento `--primary` sobre o cartão claro basta (decisão do autor). Tiles ampliam 1,05 e botões redondos 1,08, com `--focus-shadow` nos tiles. As abas não têm contorno: o foco é o sublinhado de 2 px. As transições de foco SHALL durar 150 ms com curva ease-out (`--focus-time`); a troca de aba SHALL ter fade de 180 ms; o dock SHALL deslizar com a animação de Magnification; quando a ordem dos favoritos muda (mover, remover, desinstalar), cada ícone SHALL deslizar do lugar e do tamanho antigos para os novos em 240 ms (ease-in-out), e o removido encolhe e some no lugar; o menu de contexto SHALL aparecer com fade de 150 ms e subida de 8. Um elemento sem ação (como o painel de estatísticas) SHALL NOT receber foco nem ter animação. Uma animação cujo valor de repouso é 0 SHALL receber o valor inicial em `animate_to`.

#### Scenario: Foco em botão redondo
- **WHEN** o foco chega ao botão de entradas ou à engrenagem
- **THEN** o botão ganha contorno azul de 3 px e cresce 1,08 em 150 ms

#### Scenario: Cartão sem foco
- **WHEN** o usuário navega pela tela de Configurações
- **THEN** o foco passa só pelos itens da lista, nunca pelo cartão de estatísticas

### Requirement: Superfícies e papel de parede
O fundo de toda tela SHALL ser o papel de parede de aurora (ver `wallpaper`) sobre `--bg`; as telas de texto o desenham escurecido no próprio desenho do papel (`Painter::wallpaper(true)`). Cartões e painéis SHALL usar `.panel` (`--surface`, raio `--radius` de 12). A exceção é o menu de contexto (`.menu`): um cartão claro (`--menu-surface`, raio 20, sombra como a do dock), o único elemento claro da interface, para se destacar como camada temporária sobre as telas escuras. O dock é translúcido (`rgba(15,23,30,.55)`, raio 28, contorno de 1 px a 14% de branco), e o nome do favorito em foco, acima dele, tem fundo transparente: só o texto (`--text`, 18, peso 700), sem balão nem sombra. O desenho SHALL NOT usar nenhuma camada translúcida de tela cheia, exceto o escurecimento do diálogo de confirmação, que só existe enquanto o diálogo está aberto.

#### Scenario: Escurecimento
- **WHEN** a tela de Configurações, Sobre ou Editar é desenhada
- **THEN** o escurecimento vem do desenho do papel de parede, sem camada translúcida por cima
