# Spec Delta

## MODIFIED Requirements

### Requirement: Barra superior
A barra superior SHALL exibir o logo do Lume, as abas "Início" e "Apps", o botão de entradas, o botão de Configurações (ícone de engrenagem) e o relógio, sobre o papel de parede. O botão de entradas SHALL ficar à esquerda da engrenagem, com o mesmo ícone do menu de entradas do launcher do Google TV, e SHALL abrir o menu de entradas da TV (ação `com.android.tv.action.VIEW_INPUTS`) ao receber OK. Se o sistema não resolver essa ação, o botão SHALL NOT ser exibido nem receber foco. A engrenagem SHALL abrir diretamente as configurações do Android ao receber OK. O indicador de uma aba SHALL ser um sublinhado sob o nome. Com o foco na barra superior (abas, entradas ou engrenagem), o sublinhado SHALL ser azul (a cor de destaque do tema); com o foco no conteúdo (dock ou grade), ele SHALL aparecer na aba ativa em branco, para indicar que o usuário não está controlando as abas. A mudança de cor SHALL acompanhar a animação do foco. Mover o foco para uma aba (Esquerda ou Direita na barra superior) SHALL exibir essa aba imediatamente, sem exigir OK, e o sublinhado SHALL aparecer sob ela; o foco SHALL permanecer na aba. OK sobre uma aba SHALL não mudar nada além do que o foco já mudou. As abas SHALL NOT ter contorno em volta do nome, nem em foco. A aba que não está ativa nem em foco SHALL aparecer em cinza claro. Os botões de entradas e de engrenagem em foco SHALL ter um contorno azul (a cor de destaque do tema).

#### Scenario: Aba ativa destacada
- **WHEN** a aba Apps está selecionada e o foco está na grade
- **THEN** o rótulo "Apps" aparece em destaque com o sublinhado branco, e "Início" aparece em cinza claro, sem contorno

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
