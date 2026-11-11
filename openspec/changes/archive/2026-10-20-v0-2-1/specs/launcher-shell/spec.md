# Spec Delta

## MODIFIED Requirements

### Requirement: Barra superior
A barra superior SHALL exibir o logo do Lume, as abas "Início" e "Apps", o botão de entradas, o botão de Configurações (ícone de engrenagem) e o relógio, sobre o papel de parede. O botão de entradas SHALL ficar à esquerda da engrenagem, com o mesmo ícone do menu de entradas do launcher do Google TV, e SHALL abrir o menu de entradas da TV (ação `com.android.tv.action.VIEW_INPUTS`) ao receber OK. Se o sistema não resolver essa ação, o botão SHALL NOT ser exibido nem receber foco. A engrenagem SHALL abrir diretamente as configurações do Android ao receber OK. O indicador de uma aba SHALL ser um sublinhado sob o nome. Com o foco na barra superior (abas, entradas ou engrenagem), o sublinhado SHALL ser azul (a cor de destaque do tema); com o foco no conteúdo (dock ou grade), ele SHALL aparecer na aba ativa em branco, para indicar que o usuário não está controlando as abas. A mudança de cor SHALL acompanhar a animação do foco. Mover o foco para uma aba (Esquerda ou Direita na barra superior) SHALL exibir essa aba imediatamente, sem exigir OK, e o sublinhado SHALL aparecer sob ela; o foco SHALL permanecer na aba. OK sobre uma aba SHALL não mudar nada além do que o foco já mudou. As abas SHALL NOT ter contorno em volta do nome, nem em foco. A aba que não está ativa nem em foco SHALL aparecer em cinza claro. Os botões de entradas e de engrenagem em foco SHALL ter um contorno azul (a cor de destaque do tema).

#### Scenario: Aba ativa destacada
- **WHEN** a aba Apps está selecionada e o foco está na grade
- **THEN** o rótulo "Apps" aparece em destaque com o sublinhado azul, e "Início" aparece em cinza claro, sem contorno

#### Scenario: Sublinhado branco no conteúdo
- **WHEN** o foco está num item do dock ou da grade de Apps
- **THEN** o sublinhado da aba ativa é branco

#### Scenario: Sublinhado azul na barra
- **WHEN** o foco sobe para uma aba, para o botão de entradas ou para a engrenagem
- **THEN** o sublinhado fica azul, e volta a ficar branco quando o foco desce de novo

#### Scenario: Aba em foco sem contorno
- **WHEN** o foco chega a uma aba da barra superior
- **THEN** o nome da aba não tem contorno e o sublinhado azul aparece sob ele, sem precisar pressionar OK

#### Scenario: Trocar de aba só com o foco
- **WHEN** a aba Início está ativa, o foco está nela e o usuário pressiona Direita, chegando a "Apps"
- **THEN** a tela passa a mostrar a grade da aba Apps sem pressionar OK, o sublinhado azul fica sob "Apps" e o foco continua na aba

#### Scenario: Voltar à aba Início
- **WHEN** o foco está em "Apps" e o usuário pressiona Esquerda
- **THEN** a tela volta a mostrar a tela Início, com o sublinhado sob "Início"

#### Scenario: Passar pela aba a caminho da engrenagem
- **WHEN** o usuário pressiona Direita duas vezes a partir da aba Início
- **THEN** a aba Apps é exibida ao passar por ela, e o foco chega ao botão de entradas (o vizinho da engrenagem) com o sublinhado sob "Apps"

#### Scenario: Foco na engrenagem
- **WHEN** o foco chega ao botão de entradas ou ao de engrenagem
- **THEN** o botão ganha um contorno azul

#### Scenario: Abrir as entradas da TV
- **WHEN** o foco está no botão de entradas e o usuário pressiona OK numa TV que resolve `com.android.tv.action.VIEW_INPUTS`
- **THEN** o menu de entradas da TV é aberto

#### Scenario: TV sem menu de entradas
- **WHEN** o sistema não resolve `com.android.tv.action.VIEW_INPUTS`
- **THEN** a barra superior não exibe o botão de entradas, e a engrenagem fica na mesma posição

#### Scenario: Engrenagem abre o Android
- **WHEN** o foco está na engrenagem e o usuário pressiona OK
- **THEN** a tela de configurações do Android é aberta, e não as configurações do Lume

## REMOVED Requirements

### Requirement: Tela de Configurações do lume
**Reason**: a engrenagem passa a abrir as configurações do Android, e a tela do Lume ganha outra porta de entrada (o tile "Lume") e outro conjunto de itens.
**Migration**: substituída por "Tela de Configurações do Lume".

## ADDED Requirements

### Requirement: Tela de Configurações do Lume
A tela "Configurações" do Lume SHALL ser aberta pelo tile "Lume" da grade de Apps, com os itens "Tela Início" (abre o modo edição) e "Sobre o Lume". Ela SHALL NOT ter item para as configurações do Android, nem ser aberta pela engrenagem. A tela SHALL exibir, à direita dos itens, o painel de estatísticas da TV (capacidade `device-stats`). A tela SHALL exibir o caminho "Configurações" e, no canto inferior direito, a versão instalada (ex.: "Versão 0.2.1"). As telas abaixo dela SHALL exibir o caminho de navegação a partir de "Configurações" (ex.: "Configurações › Tela Início"). Voltar SHALL retraçar o caminho de entrada, um passo por vez: da tela de edição para a tela "Configurações" (com o foco em "Tela Início") e dela para o tile "Lume" da aba Apps. Voltar da tela SHALL devolver o foco ao tile "Lume" na aba Apps.

#### Scenario: Abrir pelo tile
- **WHEN** o foco está no tile "Lume" da grade de Apps e o usuário pressiona OK
- **THEN** a tela "Configurações" do Lume é exibida com o foco no primeiro item

#### Scenario: Painel de estatísticas visível
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** o painel de estatísticas aparece à direita dos itens, sem receber foco

#### Scenario: Caminho e versão
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** o caminho "Configurações" aparece no topo, e a versão instalada no canto inferior direito

#### Scenario: Sem item do Android
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** ela lista apenas "Tela Início" e "Sobre o Lume"

#### Scenario: Voltar ao tile
- **WHEN** o usuário pressiona Voltar na tela "Configurações" do Lume
- **THEN** a aba Apps é exibida com o foco no tile "Lume"

#### Scenario: Ir para a edição da tela Início
- **WHEN** o usuário seleciona "Tela Início" em Configurações
- **THEN** o modo edição é aberto com o caminho "Configurações › Tela Início" e o título "Tela Início"

#### Scenario: Voltar passo a passo
- **WHEN** o usuário está no modo edição aberto pelas Configurações e pressiona Voltar duas vezes
- **THEN** a primeira volta à tela "Configurações" com o foco em "Tela Início", e a segunda à aba Apps com o foco no tile "Lume"

#### Scenario: Tela Sobre
- **WHEN** o usuário escolhe "Sobre o Lume"
- **THEN** é exibida uma tela com a versão do app, a licença (GPL-3.0-or-later), o endereço do código-fonte e as licenças de terceiros (fonte, ícones e bibliotecas)

### Requirement: Nome do app com L maiúsculo
O nome do app SHALL ser escrito "Lume" em todo texto que o usuário lê: o rótulo do app no sistema, a barra superior, o tile da grade, a tela Sobre e as telas de configurações, em todos os idiomas fornecidos. Identificadores técnicos (pacote, biblioteca nativa, tag de log) SHALL NOT mudar.

#### Scenario: Rótulo no sistema
- **WHEN** o usuário vê o Lume na lista de apps ou nas configurações de aplicativos do Android
- **THEN** o nome exibido é "Lume"

#### Scenario: Tela Sobre
- **WHEN** a tela Sobre é exibida em português ou em inglês
- **THEN** o título é "Sobre o Lume" ou "About Lume"
