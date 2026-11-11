## ADDED Requirements

### Requirement: Splash na abertura e no retorno
O objetivo do splash é a experiência do usuário: a abertura do Lume e a passagem entre outro app e o Lume SHALL ser suaves e breves. Ao ser aberto (a frio) ou trazido de volta ao primeiro plano (a quente), o Lume SHALL mostrar o splash: o logo do Lume centralizado sobre o fundo `--bg`. O splash SHALL ser desenhado pelo próprio Lume: a janela inicial do sistema mostra só a cor `--bg`, e o primeiro quadro do Lume mostra o logo sobre a mesma cor, com fade de entrada, de modo que a passagem não tenha salto. O Lume SHALL manter o splash até que as imagens das duas abas estejam carregadas (os ícones do dock e os tiles que a grade de Apps mostra: os visíveis na rolagem restaurada, quando ela é a tela de destino, ou os da primeira tela, para que a troca de aba logo depois da revelação também não mostre ícones chegando), por no máximo 600 ms a partir do primeiro quadro, e SHALL então revelar a interface. Um ícone que chegar com a interface já na tela (depois do teto, ou abaixo da primeira tela da grade) SHALL entrar com fade de 150 ms sobre a cor e a inicial do app, que somem sob ele, e não aparecer de uma vez. O splash SHALL NOT ter tempo mínimo e SHALL NOT mostrar a versão. Nesta versão, o splash SHALL NOT mostrar barra, porcentagem ou outro indicador de carregamento. O Lume SHALL, porém, acompanhar durante o splash quantas das imagens esperadas já estão prontas, de modo que um indicador futuro possa usar essa informação sem mudar o restante do splash. Uma tecla pressionada durante o splash, exceto Home, SHALL encerrá-lo imediatamente (a interface aparece sem a revelação) e SHALL ser aplicada à interface. Home SHALL seguir as regras de Home e de retorno sem encerrar o splash, porque é a tecla que traz o Lume de volta: ela só define a tela que o splash revela. A tela de destino SHALL ser a definida pelas regras de Home e de retorno depois de abrir um app.

#### Scenario: Abertura a frio
- **WHEN** o processo do Lume não existe e o usuário pressiona Home
- **THEN** a tela passa do fundo da janela do sistema ao logo do Lume, sobre a mesma cor, e a tela Início aparece já com os ícones do dock, sem que eles surjam depois

#### Scenario: Retorno a quente
- **WHEN** o Lume está em segundo plano e o usuário pressiona Home
- **THEN** o logo aparece e a tela Início é revelada assim que os ícones do dock estão carregados

#### Scenario: Retorno para a grade
- **WHEN** o usuário volta ao Lume depois de abrir um app pela grade de Apps
- **THEN** o splash segura até os tiles visíveis estarem carregados e revela a grade com o foco e a rolagem restaurados

#### Scenario: Ícone que não carrega
- **WHEN** um ícone do dock não termina de carregar em 600 ms
- **THEN** a interface é revelada mesmo assim, e o item mostra a cor e a inicial até o ícone chegar, quando ele entra com fade

#### Scenario: Troca para Apps logo depois da revelação
- **WHEN** o Lume volta para a tela Início e o usuário vai para a aba Apps assim que a interface aparece
- **THEN** os tiles da primeira tela da grade já estão carregados

#### Scenario: Sem indicador de carregamento
- **WHEN** o splash está na tela esperando os ícones
- **THEN** só o logo aparece, sem barra nem porcentagem, embora o Lume saiba quantas imagens faltam

#### Scenario: Home durante o splash
- **WHEN** o Home que trouxe o Lume de volta chega enquanto o splash está na tela
- **THEN** o splash continua e revela a tela definida pelas regras de Home e de retorno

#### Scenario: Tecla durante o splash
- **WHEN** o usuário pressiona Direita enquanto o splash está na tela
- **THEN** o splash acaba na hora e o foco se move para a direita na interface
