# launcher-shell Specification

## Purpose
Define a casca do launcher lume: registro como tela inicial, barra superior, navegação entre abas, tela de Configurações do lume e comportamento dos botões do controle remoto.

## Requirements

### Requirement: Registro como tela inicial
O lume SHALL se declarar como atividade de tela inicial (HOME) e como app de TV (LEANBACK_LAUNCHER), para poder ser escolhido como launcher padrão e aparecer na lista de apps do sistema.

#### Scenario: Escolha como launcher padrão
- **WHEN** o usuário (ou um comando adb) define o lume como atividade HOME padrão
- **THEN** pressionar o botão Home do controle abre o lume na tela Início

#### Scenario: Instância única
- **WHEN** o botão Home é pressionado com o lume já aberto
- **THEN** o lume não cria uma segunda instância e volta para a aba Início, com o foco no primeiro elemento do conteúdo

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

### Requirement: Relógio
O relógio SHALL mostrar a hora local do dispositivo, respeitar a preferência de 12 h ou 24 h do sistema e se atualizar na virada de cada minuto.

#### Scenario: Virada de minuto
- **WHEN** o horário do sistema passa de 21:45:59 para 21:46:00 com o lume visível
- **THEN** o relógio exibe 21:46 em até 1 segundo

#### Scenario: Formato 12 h
- **WHEN** o sistema está configurado para 12 horas
- **THEN** o relógio exibe a hora no formato de 12 horas

### Requirement: Navegação por controle remoto
Toda a interface SHALL ser operável apenas com D-pad (cima, baixo, esquerda, direita), OK/Enter e Voltar, sem exigir toque ou mouse. Exatamente um elemento SHALL estar focado por vez, indicado pelo estilo de foco do tema.

#### Scenario: Ir do conteúdo para a barra superior
- **WHEN** o foco está na primeira linha de conteúdo e o usuário pressiona Cima
- **THEN** o foco vai para a aba ativa na barra superior

#### Scenario: Trocar de aba
- **WHEN** o foco está na aba "Início" e o usuário pressiona Direita e depois OK
- **THEN** a aba "Apps" é ativada e a grade de apps é exibida

### Requirement: Botão Voltar
Na raiz da tela Início, o botão Voltar SHALL devolver o foco ao primeiro item do dock, sem fechar o lume. Em telas secundárias (Apps, Configurações, edição, Sobre), Voltar SHALL retornar ao nível anterior.

#### Scenario: Voltar na raiz
- **WHEN** o usuário está na aba Início com o foco num item do dock e pressiona Voltar
- **THEN** o foco vai para o primeiro item do dock e o lume continua aberto

#### Scenario: Voltar na aba Apps
- **WHEN** o usuário está na aba Apps e pressiona Voltar
- **THEN** a aba Início volta a ficar ativa

### Requirement: Botão Menu
O Lume SHALL reconhecer o botão Menu do controle remoto e do teclado (`KEYCODE_MENU`). Com o foco num item que tem menu de contexto (um app da grade de Apps ou um favorito do dock), Menu SHALL abrir o menu de contexto desse item (capacidades `apps-screen` e `home-screen`). Em qualquer outro lugar (barra superior, tile "Lume", item "Adicionar favoritos", Configurações, Sobre, modo edição), Menu SHALL NOT fazer nada. Com um menu aberto, Menu ou Voltar SHALL fechá-lo sem executar nenhuma ação e devolver o foco ao item de origem; OK SHALL executar o item em foco e fechar o menu. Enquanto o menu estiver aberto, o foco SHALL ficar preso nele, e Home SHALL fechá-lo antes de seguir o comportamento normal de Home. O mesmo vale para o modo de mover um favorito (`home-screen`): ele prende as teclas, e Home o cancela antes de seguir.

#### Scenario: Abrir sobre um app
- **WHEN** o foco está num app da grade e o usuário pressiona Menu
- **THEN** o menu de contexto do app aparece junto do tile, com o foco no primeiro item

#### Scenario: Sem menu na barra superior
- **WHEN** o foco está na engrenagem e o usuário pressiona Menu
- **THEN** nada muda na tela

#### Scenario: Fechar sem ação
- **WHEN** o menu de contexto está aberto e o usuário pressiona Voltar
- **THEN** o menu fecha, nada é alterado e o foco volta ao item de origem

#### Scenario: Foco preso
- **WHEN** o menu está aberto com o foco no último item e o usuário pressiona Baixo
- **THEN** o foco permanece no menu

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

### Requirement: Tela de Configurações do Lume
A tela "Configurações" do Lume SHALL ser aberta pelo tile "Lume" da grade de Apps, com os itens "Tela Início", "Mapeamento de controle", "Informações da TV" e "Sobre o Lume", sem item do Android e com o painel de estatísticas à direita (`device-stats`), caminho "Configurações" no topo e versão instalada (`platform-compatibility`) no canto inferior direito. Voltar SHALL retraçar o caminho de entrada passo a passo até o tile "Lume".

#### Scenario: Abrir pelo tile
- **WHEN** o foco está no tile "Lume" da grade de Apps e o usuário pressiona OK
- **THEN** a tela "Configurações" do Lume é exibida com o foco no primeiro item

#### Scenario: Painel de estatísticas visível
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** o painel de estatísticas aparece à direita dos itens, sem receber foco

#### Scenario: Caminho e versão
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** o caminho "Configurações" aparece no topo, e a versão instalada no canto inferior direito

#### Scenario: Versão de release com "v"
- **WHEN** o build instalado veio da tag `v0.3.0` e a tela "Configurações" ou a tela Sobre é exibida
- **THEN** a versão aparece como "v0.3.0", com um "v" só

#### Scenario: Versão de build local
- **WHEN** o build instalado foi gerado localmente no commit `4d8438b` com mudanças não commitadas
- **THEN** a versão aparece como "4d8438b-dirty", sem "v"

#### Scenario: Sem item do Android
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** ela lista apenas "Tela Início", "Mapeamento de controle", "Informações da TV" e "Sobre o Lume"

#### Scenario: Voltar ao tile
- **WHEN** o usuário pressiona Voltar na tela "Configurações" do Lume
- **THEN** a aba Apps é exibida com o foco no tile "Lume"

#### Scenario: Ir para a edição da tela Início
- **WHEN** o usuário seleciona "Tela Início" em Configurações
- **THEN** o modo edição é aberto com o caminho "Configurações › Tela Início" e o título "Tela Início"

#### Scenario: Ir para o Mapeamento de controle
- **WHEN** o usuário seleciona "Mapeamento de controle" em Configurações
- **THEN** a tela de mapeamento é aberta com o caminho "Configurações › Mapeamento de controle" e o título "Mapeamento de controle"

#### Scenario: Voltar passo a passo
- **WHEN** o usuário está no modo edição ou no mapeamento de controle aberto pelas Configurações e pressiona Voltar duas vezes
- **THEN** a primeira volta à tela "Configurações" com o foco no item de origem ("Tela Início" ou "Mapeamento de controle"), e a segunda à aba Apps com o foco no tile "Lume"

#### Scenario: Tela Sobre
- **WHEN** o usuário escolhe "Sobre o Lume"
- **THEN** é exibida uma tela com a versão do build (a mesma do canto da tela "Configurações"), a licença (GPL-3.0-or-later), o endereço do código-fonte `https://github.com/douglaspands/lume-tv-launcher` e as licenças de terceiros (fonte, ícones, papéis de parede e bibliotecas)

### Requirement: Nome do app com L maiúsculo
O nome do app SHALL ser escrito "Lume" em todo texto que o usuário lê: o rótulo do app no sistema, a barra superior, o tile da grade, a tela Sobre e as telas de configurações, em todos os idiomas fornecidos. Identificadores técnicos (pacote, biblioteca nativa, tag de log) SHALL NOT mudar.

#### Scenario: Rótulo no sistema
- **WHEN** o usuário vê o Lume na lista de apps ou nas configurações de aplicativos do Android
- **THEN** o nome exibido é "Lume"

#### Scenario: Tela Sobre
- **WHEN** a tela Sobre é exibida em português ou em inglês
- **THEN** o título é "Sobre o Lume" ou "About Lume"

### Requirement: Splash na abertura e no retorno
O objetivo do splash é a experiência do usuário: a abertura do Lume e a passagem entre outro app e o Lume SHALL ser suaves e breves. Ao ser aberto (a frio) ou trazido de volta ao primeiro plano (a quente), o Lume SHALL mostrar o splash: o logo do Lume centralizado sobre o fundo `--bg`. O splash SHALL ser desenhado pelo próprio Lume: a janela inicial do sistema mostra só a cor `--bg`, e o primeiro quadro do Lume mostra o logo sobre a mesma cor, com fade de entrada, de modo que a passagem não tenha salto. O Lume SHALL manter o splash até que as imagens das duas abas estejam carregadas (os ícones do dock e os tiles que a grade de Apps mostra: os visíveis na rolagem restaurada, quando ela é a tela de destino, ou os da primeira tela, para que a troca de aba logo depois da revelação também não mostre ícones chegando), por no máximo 600 ms a partir do primeiro quadro, e SHALL então revelar a interface. Um ícone que chegar com a interface já na tela (depois do teto, ou abaixo da primeira tela da grade) SHALL entrar com fade de 150 ms sobre a cor e a inicial do app, que somem sob ele, e não aparecer de uma vez. O Lume SHALL manter o logo à vista por um tempo mínimo desde o primeiro quadro, para que o usuário o veja: 600 ms na abertura a frio e 300 ms no retorno a quente. A revelação só começa depois desse tempo e de as imagens estarem prontas (ou do teto de 600 ms). O splash SHALL NOT mostrar a versão. Nesta versão, o splash SHALL NOT mostrar barra, porcentagem ou outro indicador de carregamento. O Lume SHALL, porém, acompanhar durante o splash quantas das imagens esperadas já estão prontas, de modo que um indicador futuro possa usar essa informação sem mudar o restante do splash. Uma tecla pressionada durante o splash, exceto Home, SHALL encerrá-lo imediatamente (a interface aparece sem a revelação) e SHALL ser aplicada à interface. Home SHALL seguir as regras de Home e de retorno sem encerrar o splash, porque é a tecla que traz o Lume de volta: ela só define a tela que o splash revela. A tela de destino SHALL ser a definida pelas regras de Home e de retorno depois de abrir um app.

#### Scenario: Abertura a frio
- **WHEN** o processo do Lume não existe e o usuário pressiona Home
- **THEN** a tela passa do fundo da janela do sistema ao logo do Lume, sobre a mesma cor, e a tela Início aparece já com os ícones do dock, sem que eles surjam depois

#### Scenario: Retorno a quente
- **WHEN** o Lume está em segundo plano e o usuário pressiona Home
- **THEN** o logo aparece e a tela Início é revelada quando os ícones do dock estão carregados e o tempo mínimo de 300 ms do logo passou

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

### Requirement: Mapeamento centralizado e persistente de teclas
O Lume SHALL centralizar as teclas lógicas, constantes `KEYCODE_*`, padrões `DEFAULT_*` e atalhos (`cfg.<alvo>` e `app.<pacote>`) em `lume_core::keys`, persistindo o mapeamento ativo em `files/keys.v1`. Em `Shell::start`, se `keys.v1` não existir, o Lume SHALL gravar `KeyMap::default()`; se existir e for válido, SHALL carregá-lo sem regravar. Chaves vazias SHALL desabilitar a tecla customizável, e arquivos corrompidos SHALL ser autocurados.

#### Scenario: Primeiro uso cria o arquivo de mapeamento
- **WHEN** o Lume é iniciado pela primeira vez e `files/keys.v1` não existe
- **THEN** o Lume carrega as constantes padrão embutidas na compilação e grava `files/keys.v1` nos dados do app com todas as teclas padrão

#### Scenario: Uso do mapeamento salvo nos dados do app
- **WHEN** `files/keys.v1` existe com um código customizado (por exemplo, `menu=165`) e o Lume é iniciado
- **THEN** o Lume passa a reconhecer o código `165` como a tecla Menu sem reescrever `files/keys.v1`

#### Scenario: Persistência de atalhos de configurações e aplicativos
- **WHEN** o usuário associa a tecla `174` ao atalho de configuração `sleep` e a tecla `183` ao aplicativo `com.google.android.youtube.tv`
- **THEN** `files/keys.v1` persiste `cfg.sleep=174` e `app.com.google.android.youtube.tv=183` e recarrega esses atalhos no próximo início

#### Scenario: Uma tecla acionada por múltiplas teclas do controle remoto
- **WHEN** `files/keys.v1` associa múltiplos códigos a uma mesma tecla (por exemplo, `menu=82,174,165` ou `ok=23,66,160,96`)
- **THEN** o Lume reconhece qualquer um dos códigos físicos enviados pelo controle remoto (`82`, `174` ou `165`) como a ação correspondente (Menu), mantendo o arquivo válido sem reescrita

#### Scenario: Desabilitar uma tecla com valor vazio
- **WHEN** `files/keys.v1` define uma tecla com valor vazio (por exemplo, `menu=`)
- **THEN** nenhum código aciona aquela tecla e o arquivo é mantido como válido

#### Scenario: Autocura de arquivo corrompido por queda de energia
- **WHEN** `files/keys.v1` está truncado (sem `#end`) ou com cabeçalho inválido ao iniciar o Lume
- **THEN** o Lume restaura todas as teclas para o padrão da compilação e regrava imediatamente um `files/keys.v1` íntegro no disco

#### Scenario: Autocura de linha inválida ou tecla ausente
- **WHEN** `files/keys.v1` tem estrutura válida e customiza `menu=165`, mas a linha de `ok` contém um valor não numérico (`ok=23,abc`) ou a chave `home` está ausente
- **THEN** o Lume preserva `menu=165`, restaura `ok` e `home` para seus padrões da compilação e regrava o `files/keys.v1` corrigido no disco

### Requirement: Perfil de teclas embutido na compilação e limpeza de dados
O build do Lume SHALL aceitar opcionalmente a variável `LUME_KEYS_FILE` apontando para um arquivo no formato `lume=1 ... #end`. Quando informada, o build SHALL validar o arquivo em tempo de compilação e embutir seus valores como os padrões `DEFAULT_*` do binário; quando omitida, SHALL usar as constantes originais do Lume. Ao limpar os dados do app no Android, o próximo início SHALL recriar `files/keys.v1` a partir dos `DEFAULT_*` do binário.

#### Scenario: Limpar dados do app em compilação padrão
- **WHEN** o usuário limpa os dados do Lume nas configurações do Android em uma compilação padrão (sem `LUME_KEYS_FILE`) e abre o Lume novamente
- **THEN** o arquivo antigo some na limpeza e o Lume recria `files/keys.v1` com as constantes padrão originais do Lume

#### Scenario: Compilação com arquivo embutido e limpeza de dados
- **WHEN** o Lume foi compilado com `LUME_KEYS_FILE` definindo um mapeamento específico para uma TV, o usuário altera `files/keys.v1` e depois limpa os dados do app no Android
- **THEN** ao abrir o Lume novamente, `files/keys.v1` é recriado com o mapeamento que foi embutido na compilação via `LUME_KEYS_FILE`
