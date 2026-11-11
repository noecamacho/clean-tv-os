## ADDED Requirements

### Requirement: Splash
O splash SHALL ser só o logo do Lume (o quadrado arredondado com o L e o ponto, nas proporções de `logo.rs`) centralizado sobre `--bg`, com tamanho definido no tema, sem texto. O tema SHALL reservar abaixo do logo uma faixa para um indicador de carregamento futuro (posição e altura em tokens), que nesta versão fica vazia e não desloca o logo. O logo SHALL entrar com fade de 120 ms (ease-out) no primeiro quadro do splash, sobre o mesmo `--bg` da janela inicial do sistema. A revelação SHALL acontecer ao mesmo tempo em três partes: o papel de parede passa da intensidade 0 (só `--bg`) à intensidade normal, pela própria cor com que é desenhado; o logo some com fade, aplicado só à área do logo; e a interface entra com fade, pela opacidade de cada elemento (como o fade das abas), com o conteúdo (dock ou grade) subindo 24 unidades e a barra superior parada. A revelação SHALL durar 300 ms na abertura a frio e 220 ms no retorno a quente, com curva ease-out. Um ícone que chegar com a interface na tela SHALL entrar com fade de 150 ms, ease-out (`--icon-in`), enquanto a cor e a inicial do app saem sob ele. Nenhuma etapa SHALL usar camada translúcida de tela cheia.

#### Scenario: Entrada do logo
- **WHEN** a janela do sistema é substituída pelo primeiro quadro do Lume
- **THEN** o fundo continua da mesma cor e o logo surge com fade, sem saltar

#### Scenario: Revelação sem camada de tela cheia
- **WHEN** a interface é revelada
- **THEN** o papel de parede ganha intensidade pelo próprio desenho, e só a área do logo é desenhada com transparência

## MODIFIED Requirements

### Requirement: Menu de contexto
O menu de contexto (botão Menu, ver `launcher-shell`) SHALL ser um cartão claro (`.menu`: `--menu-surface`, raio 20, sombra próxima e em volta do cartão (0 4 24, preto a 45%), que o separa de um tile branco), como o do launcher do Android TV, para se destacar sobre as telas escuras. Ele SHALL ficar junto do item de origem, sem seta e sem escurecer a tela (sem camada translúcida de tela cheia): acima do favorito no dock e ao lado do tile na grade (à direita, ou à esquerda quando não couber), sempre dentro da área segura.

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

### Requirement: Superfícies e papel de parede
O fundo de toda tela SHALL ser o papel de parede de aurora (ver `wallpaper`) sobre `--bg`; as telas de texto o desenham escurecido no próprio desenho do papel (`Painter::wallpaper(true)`). Cartões e painéis SHALL usar `.panel` (`--surface`, raio `--radius` de 12). A exceção é o menu de contexto (`.menu`): um cartão claro (`--menu-surface`, raio 20, sombra próxima e em volta do cartão (0 4 24, preto a 45%), que o separa de um tile branco), o único elemento claro da interface, para se destacar como camada temporária sobre as telas escuras. O dock é translúcido (`rgba(15,23,30,.55)`, raio 28, contorno de 1 px a 14% de branco), e o nome do favorito em foco, acima dele, tem fundo transparente: só o texto (`--text`, 18, peso 700), sem balão nem sombra. O desenho SHALL NOT usar nenhuma camada translúcida de tela cheia, exceto o escurecimento do diálogo de confirmação, que só existe enquanto o diálogo está aberto.

#### Scenario: Escurecimento
- **WHEN** a tela de Configurações, Sobre ou Editar é desenhada
- **THEN** o escurecimento vem do desenho do papel de parede, sem camada translúcida por cima
