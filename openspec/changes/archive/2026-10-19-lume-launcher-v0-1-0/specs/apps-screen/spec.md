# Spec Delta

## Purpose

Define a aba Apps do lume: uma grade com todos os apps instalados que podem ser abertos, conforme o mockup "Todos os apps".

## ADDED Requirements

### Requirement: Grade de todos os apps
A aba Apps SHALL exibir o título "Todos os apps" e uma grade de 5 colunas com todos os apps do catálogo, em ordem alfabética do nome de exibição (ignorando maiúsculas e acentos). Cada item SHALL mostrar o tile do app (o banner de TV ou, sem banner, o ícone sobre a cor do tile; o nome dentro do tile enquanto a imagem não está disponível), e o nome abaixo do tile.

#### Scenario: Ordem alfabética
- **WHEN** estão instalados YouTube, Netflix e Globoplay
- **THEN** a grade os mostra na ordem Globoplay, Netflix, YouTube

#### Scenario: Muitos apps
- **WHEN** existem mais apps do que cabem na tela
- **THEN** a grade rola verticalmente, mantendo a linha focada visível

### Requirement: Navegação na grade
Na grade, as setas SHALL mover o foco entre células vizinhas. Cima na primeira linha SHALL levar o foco à barra superior. Direita na última coluna SHALL NOT pular para a linha seguinte. OK SHALL abrir o app focado.

#### Scenario: Limite da linha
- **WHEN** o foco está na 5ª coluna e o usuário pressiona Direita
- **THEN** o foco permanece no mesmo tile

#### Scenario: Última linha incompleta
- **WHEN** o foco está na penúltima linha, na coluna 4, e a última linha tem só 2 itens
- **THEN** pressionar Baixo leva o foco ao último item da última linha

### Requirement: Atalho de configurações do sistema
A grade SHALL incluir, como último item, um tile "Configurações" que abre as configurações do Android, mesmo que o app de configurações não tenha atividade de entrada própria.

#### Scenario: Abrir configurações pela grade
- **WHEN** o usuário seleciona o tile "Configurações"
- **THEN** as configurações do Android são abertas
