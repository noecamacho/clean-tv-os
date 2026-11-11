# Spec Delta

## ADDED Requirements

### Requirement: Retorno depois de abrir um app
Quando o usuário abre um app a partir do lume (OK num item do dock, num tile da grade de Apps ou nos itens que abrem as Configurações do Android) e volta ao lume com o botão Home, o lume SHALL reaparecer na mesma tela, com o foco no mesmo elemento e com a rolagem de antes, em vez de ir à tela Início. Esse retorno SHALL valer uma única vez e SHALL ser descartado assim que o usuário pressionar qualquer outra tecla no lume, ou quando o app não puder ser aberto. O botão Home pressionado com o lume já visível na tela (ou mais de 2 segundos depois de ele voltar) SHALL sempre levar à aba Início, com o foco no primeiro item do dock. O estado SHALL ficar só na memória: se o sistema encerrar o processo do lume, ele abre na tela Início.

#### Scenario: Voltar à aba Apps no mesmo ícone
- **WHEN** o usuário está na aba Apps, move o foco até o YouTube, pressiona OK e depois volta ao lume pelo botão Home
- **THEN** a aba Apps é exibida com o foco no ícone do YouTube

#### Scenario: Voltar ao dock no mesmo item
- **WHEN** o usuário abre o quarto item do dock e volta ao lume pelo botão Home
- **THEN** a tela Início é exibida com o foco no quarto item, e não no primeiro

#### Scenario: Grade rolada
- **WHEN** a grade de Apps estava rolada até uma linha distante, e o app dessa linha foi aberto
- **THEN** ao voltar, a grade mostra a mesma linha, com o foco no mesmo app

#### Scenario: Home com o lume visível
- **WHEN** o lume está na tela, em qualquer aba ou tela, mesmo depois de o usuário ter aberto um app, e o usuário pressiona Home
- **THEN** a aba Início é exibida, com o foco no primeiro item do dock

#### Scenario: Retorno usado uma vez
- **WHEN** o usuário volta de um app pelo Home e pressiona Home de novo com o lume aberto
- **THEN** o segundo Home leva à aba Início, com o foco no primeiro item do dock

#### Scenario: Tecla pressionada antes do Home
- **WHEN** o usuário abre um app, volta ao lume pelo botão Voltar do app, move o foco e depois pressiona Home
- **THEN** o lume vai à aba Início, com o foco no primeiro item do dock

#### Scenario: App que não abre
- **WHEN** o lume não consegue abrir o app escolhido (por exemplo, foi desinstalado)
- **THEN** nada é guardado para o retorno, e o próximo Home leva à aba Início

### Requirement: Suavização entre as abas
Ao trocar de aba (Início e Apps), o conteúdo da aba nova SHALL aparecer com um fade de cerca de 180 ms, em vez de surgir de uma vez; a barra superior e o papel de parede SHALL permanecer fixos. Terminado o fade, o laço SHALL deixar de pedir quadros.

#### Scenario: Trocar de aba
- **WHEN** o usuário move o foco de "Início" para "Apps"
- **THEN** a grade de Apps surge gradualmente em cerca de 180 ms, enquanto a barra superior e o papel de parede ficam parados

#### Scenario: Depois do fade
- **WHEN** o fade termina
- **THEN** a tela volta a pedir quadros só para o relógio

## MODIFIED Requirements

### Requirement: Barra superior
A barra superior SHALL exibir o logo do lume, as abas "Início" e "Apps", o botão de Configurações (ícone de engrenagem) e o relógio, sobre o papel de parede. O indicador de uma aba SHALL ser um sublinhado sob o nome. Com o foco na barra superior (abas ou engrenagem), o sublinhado SHALL ser azul (a cor de destaque do tema); com o foco no conteúdo (dock ou grade), ele SHALL aparecer na aba ativa em branco, para indicar que o usuário não está controlando as abas. A mudança de cor SHALL acompanhar a animação do foco. Mover o foco para uma aba (Esquerda ou Direita na barra superior) SHALL exibir essa aba imediatamente, sem exigir OK, e o sublinhado SHALL aparecer sob ela; o foco SHALL permanecer na aba. OK sobre uma aba SHALL não mudar nada além do que o foco já mudou. As abas SHALL NOT ter contorno em volta do nome, nem em foco. A aba que não está ativa nem em foco SHALL aparecer em cinza claro. O botão de engrenagem em foco SHALL ter um contorno azul (a cor de destaque do tema).

#### Scenario: Aba ativa destacada
- **WHEN** a aba Apps está selecionada e o foco está na grade
- **THEN** o rótulo "Apps" aparece em destaque com o sublinhado azul, e "Início" aparece em cinza claro, sem contorno

#### Scenario: Sublinhado branco no conteúdo
- **WHEN** o foco está num item do dock ou da grade de Apps
- **THEN** o sublinhado da aba ativa é branco

#### Scenario: Sublinhado azul na barra
- **WHEN** o foco sobe para uma aba ou para a engrenagem
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
- **THEN** a aba Apps é exibida ao passar por ela, e o foco chega à engrenagem com o sublinhado sob "Apps"

#### Scenario: Foco na engrenagem
- **WHEN** o foco chega ao botão de engrenagem
- **THEN** o botão ganha um contorno azul

### Requirement: Botão Voltar
Na raiz da tela Início, o botão Voltar SHALL devolver o foco ao primeiro item do dock, sem fechar o lume. Em telas secundárias (Apps, Configurações, edição, Sobre), Voltar SHALL retornar ao nível anterior.

#### Scenario: Voltar na raiz
- **WHEN** o usuário está na aba Início com o foco num item do dock e pressiona Voltar
- **THEN** o foco vai para o primeiro item do dock e o lume continua aberto

#### Scenario: Voltar na aba Apps
- **WHEN** o usuário está na aba Apps e pressiona Voltar
- **THEN** a aba Início volta a ficar ativa
