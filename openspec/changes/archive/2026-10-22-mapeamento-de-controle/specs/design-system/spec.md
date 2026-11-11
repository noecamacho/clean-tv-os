# Spec Delta

## MODIFIED Requirements

### Requirement: Item de Informações da TV nas Configurações
A tela de Configurações SHALL incluir, na coluna esquerda de itens, as opções "Tela Início", "Mapeamento de controle", "Informações da TV" e "Sobre o Lume", respeitando a altura de 72 px e o passo de 88 px na régua.

#### Scenario: Terceiro item de configurações
- **WHEN** a tela de Configurações é desenhada
- **THEN** a lista exibe os itens "Tela Início", "Mapeamento de controle", "Informações da TV" e "Sobre o Lume" alinhados à régua de 4 px

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

## ADDED Requirements

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
