# remote-mapping Specification

## Purpose

Define a tela "Mapeamento de controle" e o sistema de personalização de teclas do controle remoto do Lume: proteção das teclas vitais de navegação e energia, adição e remoção individual de teclas em comandos do Lume, atalhos para configurações específicas da TV/Lume e para aplicativos, restauração pontual/total.

## Requirements

### Requirement: Acesso à tela de Mapeamento de controle
A tela "Mapeamento de controle" SHALL ser aberta pelo item "Mapeamento de controle" das Configurações do Lume, exibindo o caminho "Configurações › Mapeamento de controle", os três painéis ("Comandos do Lume", "Atalhos de configurações" e "Atalhos de aplicativos") e o botão "Restaurar padrão de fábrica" no rodapé. Voltar SHALL retornar às Configurações com foco em "Mapeamento de controle".

#### Scenario: Abrir pelas Configurações
- **WHEN** o usuário seleciona "Mapeamento de controle" na tela Configurações do Lume
- **THEN** a tela "Mapeamento de controle" é aberta com o caminho "Configurações › Mapeamento de controle" e o foco na primeira linha de "Comandos do Lume"

#### Scenario: Voltar para Configurações
- **WHEN** o usuário está na tela "Mapeamento de controle" (sem modal, menu ou diálogo aberto) e pressiona Voltar
- **THEN** a tela "Configurações" é exibida com o foco em "Mapeamento de controle", e todas as alterações de teclas já feitas permanecem salvas

#### Scenario: Foco só vertical
- **WHEN** o usuário navega pela tela "Mapeamento de controle"
- **THEN** cima e baixo percorrem, em ordem, as linhas dos três painéis (incluindo as linhas "Adicionar") e o botão "Restaurar padrão de fábrica", e esquerda e direita não mudam o foco

### Requirement: Proteção de teclas vitais do sistema
O Lume SHALL proibir a modificação, remoção ou reatribuição das teclas vitais de navegação e energia: Direcionais (`Up`, `Down`, `Left`, `Right`), Confirmar (`Ok`/`Enter`), Voltar (`Back`/`Escape`), Início (`Home`) e Desligar/Energia (`Power`, `TV Power`, `Sleep`, `Wakeup`). Essas teclas SHALL manter sempre seu comportamento nativo e NUNCA ser gravadas em nenhum alvo; no modal de captura, todas elas, exceto Voltar (que cancela), SHALL ser rejeitadas com um aviso.

#### Scenario: Tentativa de mapear seta direcional, OK ou energia
- **WHEN** o modal de escuta de tecla está aberto e o usuário pressiona uma seta direcional (ex.: Direita, `22`), OK (`23`), Home (`3`) ou Power (`26`)
- **THEN** o Lume rejeita a tecla, troca a dica do modal pelo aviso de tecla reservada, mantém o modal aberto aguardando uma tecla válida e não grava nada em `files/keys.v1`

### Requirement: Modal de escuta de tecla
Ao adicionar uma tecla, o Lume SHALL abrir o modal de escuta: um cartão claro, sem botão e sem foco, com "Pressione uma tecla", "Para: <alvo>" e a dica "Voltar para cancelar". Voltar SHALL ser a única forma de cancelar, sem alterar o mapeamento; qualquer tecla não reservada SHALL ser atribuída ao alvo, fechando o modal com um aviso de confirmação.

#### Scenario: Cancelar escuta com Voltar
- **WHEN** o modal de escuta de tecla está aberto e o usuário pressiona Voltar
- **THEN** o modal é fechado sem alterar o mapeamento da ação e o foco volta à linha de origem

#### Scenario: OK não cancela a escuta
- **WHEN** o modal de escuta de tecla está aberto e o usuário pressiona OK
- **THEN** o modal continua aberto e exibe o aviso de tecla reservada

#### Scenario: Escuta cancelada de um seletor
- **WHEN** o usuário escolhe um atalho ou um aplicativo num seletor e cancela a escuta com Voltar
- **THEN** nenhum atalho é criado, a linha correspondente não aparece e o foco volta à linha "Adicionar" de origem

### Requirement: Comandos do Lume e remoção individual de teclas
O painel "Comandos do Lume" SHALL exibir "Menu de contexto" e "Entradas da TV" como linhas com as teclas em keycaps à direita (nome amigável e código, ex.: `Menu 82`). OK numa linha com teclas SHALL abrir o menu de contexto da linha, com "Adicionar tecla", "Remover <tecla>" por tecla e "Restaurar padrão"; numa linha sem teclas, abrir direto o modal de escuta. Remover uma tecla SHALL gravar `files/keys.v1` na hora.

#### Scenario: Remover uma tecla invertida do Menu de contexto
- **WHEN** "Menu de contexto" possui as teclas padrão `Menu 82` e `Favorito 174`, o usuário pressiona OK na linha e escolhe "Remover Favorito 174"
- **THEN** o código `174` é removido de "Menu de contexto", restando apenas `Menu 82`, e a alteração é gravada em `files/keys.v1`

#### Scenario: Adicionar nova tecla a um comando
- **WHEN** o usuário escolhe "Adicionar tecla" no menu da linha "Entradas da TV" e pressiona a tecla `Entrada (178)` no controle remoto
- **THEN** o código `178` é adicionado a "Entradas da TV", o modal fecha e pressionar `178` passa a abrir o painel de entradas da TV

#### Scenario: Comando sem teclas
- **WHEN** todas as teclas de "Entradas da TV" foram removidas e o usuário pressiona OK na linha
- **THEN** o modal de escuta é aberto direto, sem passar pelo menu

### Requirement: Reset pontual por ação modificada
O menu de contexto de uma linha do painel "Comandos do Lume" SHALL ter o item "Restaurar padrão". Quando o conjunto de teclas da linha diverge do padrão da compilação (`DEFAULT_*`), o item SHALL estar habilitado e, ao ser escolhido, devolver apenas aquela ação ao seu conjunto padrão e gravar `files/keys.v1` imediatamente; quando a linha já é igual ao padrão, o item SHALL aparecer desabilitado, com a nota "Já está no padrão", e SHALL NOT receber foco.

#### Scenario: Reset pontual disponível só quando modificado
- **WHEN** "Menu de contexto" foi alterado (por exemplo, `174` foi removido ou `165` foi adicionado) e "Entradas da TV" permanece no padrão
- **THEN** o menu de "Menu de contexto" mostra "Restaurar padrão" habilitado, enquanto o de "Entradas da TV" o mostra desabilitado com a nota "Já está no padrão"

#### Scenario: Executar reset pontual de uma ação
- **WHEN** o usuário escolhe "Restaurar padrão" no menu da linha "Menu de contexto"
- **THEN** "Menu de contexto" volta a ter `Menu 82` e `Favorito 174` e os demais mapeamentos não conflitantes permanecem intactos

### Requirement: Atalhos de configurações específicas da TV e do Lume
O painel "Atalhos de configurações" SHALL listar como linhas só os atalhos com tecla e, por último, a linha "Adicionar atalho" enquanto restar algum sem tecla. OK nela SHALL abrir a tela "Escolher atalho", uma lista com rolagem de **todos** os atalhos sem tecla, e escolher um abre o modal de escuta. O menu da linha de um atalho SHALL ter "Adicionar tecla", "Remover <tecla>" e "Remover atalho".

#### Scenario: Mapear tecla para o Sleep Timer da TV
- **WHEN** o usuário escolhe "Adicionar atalho", depois "Timer / Sleep e Energia da TV" na lista "Escolher atalho", e pressiona uma tecla permitida no controle universal (ex.: `Favorito 174` ou `Azul 186`)
- **THEN** a linha "Timer / Sleep da TV" aparece na lista com a tecla associada e é persistida em `files/keys.v1`

#### Scenario: Atalhos disponíveis
- **WHEN** a lista "Escolher atalho" é aberta sem nenhum atalho com tecla
- **THEN** ela tem 20 atalhos: Timer / Sleep, Imagem, Som, Rede e Wi-Fi, Bluetooth, Aplicativos, Armazenamento, Idioma, Data e hora, Teclado, Legendas, Localização, Segurança e restrições, Opções do desenvolvedor, App de início padrão, Sobre a TV, Configurações do Android, Configurações do Lume, Editar Tela Início e Informações da TV

#### Scenario: Todos os atalhos alcançáveis
- **WHEN** nenhum atalho de configuração tem tecla e o usuário abre "Adicionar atalho"
- **THEN** a lista tem os 20 atalhos, todos recebem o foco, a lista rola para mostrar o que está em foco e cada nome aparece inteiro

#### Scenario: Voltar da escolha de atalho
- **WHEN** o usuário está em "Escolher atalho" e pressiona Voltar
- **THEN** a tela de mapeamento é exibida com o foco na linha "Adicionar atalho" e nenhum atalho é criado

#### Scenario: Acionar atalho de configuração pelo controle
- **WHEN** uma tecla está associada a "Timer / Sleep da TV" e o usuário pressiona essa tecla
- **THEN** o Lume solicita ao sistema a abertura direta da tela de Timer / Desligamento automático da TV

#### Scenario: Remover um atalho de configuração
- **WHEN** o usuário escolhe "Remover atalho" no menu da linha "Timer / Sleep da TV" (ou remove sua única tecla)
- **THEN** a linha sai da lista, o atalho volta a aparecer na lista "Escolher atalho" e `files/keys.v1` é atualizado

### Requirement: Cada atalho abre a sua tela do sistema
Cada atalho de configuração SHALL abrir a sua tela do sistema, ou as Configurações gerais onde o firmware não a tem, e SHALL valer com o Lume em primeiro plano. O Lume SHALL descartar telas stub vazias e abrir a tela real de Armazenamento e o seletor do App de início padrão nos firmwares suportados.

#### Scenario: Cada atalho abre a sua tela
- **WHEN** uma tecla está associada a "Idioma" (ou a "Data e hora", "Armazenamento", "Teclado", "Legendas", "Localização", "Segurança e restrições", "Opções do desenvolvedor", "App de início padrão", "Sobre a TV") e o usuário a pressiona
- **THEN** o Lume abre a tela correspondente do sistema e não as Configurações gerais, onde o firmware a tem (conferido na AIWA e na TCL)

#### Scenario: Armazenamento não abre redefinição de fábrica
- **WHEN** uma tecla está associada a "Armazenamento" e o usuário a pressiona na TV TCL
- **THEN** o Lume abre o resumo de armazenamento (`StorageSummaryActivity`) e não a tela de redefinição de dados de fábrica (`StorageResetActivity`)

#### Scenario: App de início padrão ignora stub vazio
- **WHEN** uma tecla está associada a "App de início padrão" e o usuário a pressiona na TV TCL
- **THEN** o Lume ignora o `EmptyStubActivity` e abre o seletor de aplicativo padrão do sistema (`permissioncontroller`)

#### Scenario: Sleep abre tela com temporizador de desligamento
- **WHEN** uma tecla está associada a "Timer / Sleep da TV" e o usuário a pressiona na TV TCL
- **THEN** o Lume abre a tela de descanso do sistema (`DaydreamActivity`) com opções de Temporizador de desligamento e Desligar a tela

### Requirement: Atalhos de aplicativos
O painel "Atalhos de aplicativos" SHALL listar como linhas os apps com tecla e, por último, a linha "Adicionar aplicativo" enquanto restar algum sem atalho. OK nela SHALL abrir a tela "Escolher aplicativo" (grade de tiles da aba Apps, só com apps sem atalho); escolher um tile abre o modal de escuta para o app, e Voltar retorna com o foco em "Adicionar aplicativo". O menu da linha de um app SHALL ter "Adicionar tecla", "Remover <tecla>" e "Remover atalho".

#### Scenario: Mapear tecla para abrir um aplicativo
- **WHEN** o usuário escolhe "Adicionar aplicativo", seleciona o tile do YouTube e pressiona a tecla `Vermelho 183` no controle
- **THEN** a linha "YouTube" é adicionada com o keycap `Vermelho 183`, e pressionar `Vermelho (183)` abre o YouTube diretamente

#### Scenario: Qualquer aplicativo é alcançável
- **WHEN** o catálogo tem mais apps do que cabem numa linha e o usuário abre "Escolher aplicativo"
- **THEN** a grade rola e todo app sem atalho recebe foco, não só os primeiros em ordem alfabética

#### Scenario: Voltar da escolha de aplicativo
- **WHEN** o usuário está em "Escolher aplicativo" e pressiona Voltar
- **THEN** a tela de mapeamento é exibida com o foco na linha "Adicionar aplicativo" e nenhum atalho é criado

#### Scenario: Remover atalho de aplicativo
- **WHEN** o usuário escolhe "Remover atalho" no menu da linha "YouTube"
- **THEN** todas as teclas associadas ao YouTube são desvinculadas, a linha é removida e o YouTube volta a aparecer na grade de "Escolher aplicativo"

### Requirement: Resolução automática de conflito entre teclas
Cada código físico de tecla (`i32`) SHALL estar associado a no máximo uma ação, configuração ou aplicativo por vez. Quando o usuário atribuir uma tecla que já estava mapeada em outro comando, configuração ou aplicativo, o Lume SHALL remover automaticamente esse código do mapeamento anterior e atribuí-lo ao novo alvo.

#### Scenario: Reatribuir tecla já usada em outra função
- **WHEN** a tecla `Favorito (174)` está em "Menu de contexto" e o usuário a pressiona ao adicionar uma tecla para "Timer / Sleep da TV"
- **THEN** `174` é removido de "Menu de contexto" (cujo menu passa a ter "Restaurar padrão" habilitado) e atribuído a "Timer / Sleep da TV"

### Requirement: Restaurar mapeamento de fábrica
No final da tela "Mapeamento de controle", o botão secundário "Restaurar padrão de fábrica" SHALL abrir um diálogo de confirmação em cartão claro ("Restaurar o padrão de fábrica?", com os botões "Cancelar" e "Restaurar", este de perigo, e o foco inicial em "Cancelar"). Ao confirmar, o Lume SHALL restaurar todos os Comandos do Lume para `DEFAULT_*`, limpar todos os atalhos de configurações e de aplicativos e regravar `files/keys.v1`.

#### Scenario: Confirmar restauração de fábrica
- **WHEN** o usuário seleciona "Restaurar padrão de fábrica" e confirma em "Restaurar"
- **THEN** todos os comandos voltam ao padrão da compilação, todos os atalhos de configurações e aplicativos são limpos e `files/keys.v1` é atualizado

#### Scenario: Cancelar restauração de fábrica
- **WHEN** o usuário seleciona "Restaurar padrão de fábrica" e escolhe "Cancelar" (ou pressiona Voltar)
- **THEN** o diálogo fecha e todos os mapeamentos customizados são preservados
