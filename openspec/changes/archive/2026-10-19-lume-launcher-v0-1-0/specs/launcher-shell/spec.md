# Spec Delta

## Purpose

Define a casca do launcher lume: registro como tela inicial, barra superior, navegação entre abas, tela de Configurações do lume e comportamento dos botões do controle remoto.

## ADDED Requirements

### Requirement: Registro como tela inicial
O lume SHALL se declarar como atividade de tela inicial (HOME) e como app de TV (LEANBACK_LAUNCHER), para poder ser escolhido como launcher padrão e aparecer na lista de apps do sistema.

#### Scenario: Escolha como launcher padrão
- **WHEN** o usuário (ou um comando adb) define o lume como atividade HOME padrão
- **THEN** pressionar o botão Home do controle abre o lume na tela Início

#### Scenario: Instância única
- **WHEN** o botão Home é pressionado com o lume já aberto
- **THEN** o lume não cria uma segunda instância e volta para a aba Início, com o foco no primeiro elemento do conteúdo

### Requirement: Barra superior
A barra superior SHALL exibir o logo do lume, as abas "Início" e "Apps" (com indicador visual na aba ativa), o botão de Configurações (ícone de engrenagem) e o relógio, conforme o mockup.

#### Scenario: Aba ativa destacada
- **WHEN** a aba Apps está selecionada
- **THEN** o rótulo "Apps" aparece em destaque com o sublinhado de aba ativa, e "Início" aparece no estilo inativo

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
Na raiz da tela Início, o botão Voltar SHALL rolar a tela de volta ao topo e devolver o foco ao primeiro elemento, sem fechar o lume. Em telas secundárias (Apps, Configurações, edição, Sobre), Voltar SHALL retornar ao nível anterior.

#### Scenario: Voltar na raiz
- **WHEN** o usuário está na aba Início com o foco na linha de favoritos e pressiona Voltar
- **THEN** a tela rola até o topo, o foco vai para o carrossel e o lume continua aberto

#### Scenario: Voltar na aba Apps
- **WHEN** o usuário está na aba Apps e pressiona Voltar
- **THEN** a aba Início volta a ficar ativa

### Requirement: Tela de Configurações do lume
O botão de engrenagem da barra superior SHALL abrir a tela "Configurações" do lume, com os itens "Tela Início" (abre o modo edição), "Configurações do Android" e "Sobre o lume". As telas abaixo dela SHALL exibir o caminho de navegação a partir de "Configurações" (ex.: "Configurações › Tela Início › Editar").

#### Scenario: Abrir pela engrenagem
- **WHEN** o foco está na engrenagem da barra superior e o usuário pressiona OK
- **THEN** a tela "Configurações" do lume é exibida com o foco no primeiro item

#### Scenario: Ir para a edição da tela Início
- **WHEN** o usuário seleciona "Tela Início" em Configurações
- **THEN** o modo edição é aberto com o caminho "Configurações › Tela Início › Editar"

#### Scenario: Abrir configurações do sistema
- **WHEN** o usuário escolhe "Configurações do Android"
- **THEN** a tela de configurações do sistema é aberta

#### Scenario: Tela Sobre
- **WHEN** o usuário escolhe "Sobre o lume"
- **THEN** é exibida uma tela com a versão do app, a licença (GPL-3.0-or-later), o endereço do código-fonte e as licenças de terceiros (fonte, ícones e bibliotecas)
