# Spec Delta

## Purpose

Define o conteúdo da tela Início do lume na v0.1.0: o carrossel de destaque com os apps do usuário e a linha "Seus apps favoritos", conforme o mockup.

## ADDED Requirements

### Requirement: Seções da tela Início
A tela Início SHALL exibir, abaixo da barra superior, as seções habilitadas na configuração (carrossel de destaque e "Seus apps favoritos"), na ordem configurada. Quando o conteúdo passa da altura da tela, a tela SHALL rolar verticalmente para manter o elemento focado visível.

#### Scenario: Ordem padrão
- **WHEN** o lume é aberto pela primeira vez
- **THEN** a tela Início mostra o carrossel no topo e "Seus apps favoritos" logo abaixo

#### Scenario: Seção oculta
- **WHEN** o usuário ocultou o carrossel no modo edição
- **THEN** a tela Início começa pela linha de favoritos

### Requirement: Slides do carrossel
O carrossel SHALL exibir um slide por app selecionado em "Apps no carrossel" e que esteja instalado. Cada slide SHALL mostrar o fundo em gradiente na cor do app, a etiqueta com o nome do app, o nome do app como título, uma linha de contexto e o botão "Abrir". O carrossel SHALL NOT exibir logos, slogans ou imagens de terceiros.

#### Scenario: Slide de app
- **WHEN** o Netflix está selecionado no carrossel
- **THEN** existe um slide com o título "Netflix", o gradiente da cor do Netflix e o botão "Abrir"

#### Scenario: Abrir pelo carrossel
- **WHEN** o foco está no botão "Abrir" de um slide e o usuário pressiona OK
- **THEN** o app daquele slide é aberto

### Requirement: Linha de contexto do slide
A linha de contexto SHALL ser escolhida com base no histórico local, nesta ordem de prioridade: "Novo no seu TV" (app visto pela primeira vez há menos de 7 dias), "Mais usado da semana" (app com a maior contagem semanal, quando ela é maior que zero), "Aberto há <tempo>" (há uma última abertura) e, na falta de tudo isso, "Toque em Abrir para começar".

#### Scenario: App mais usado
- **WHEN** o YouTube tem a maior contagem de aberturas nos últimos 7 dias
- **THEN** o slide do YouTube mostra "Mais usado da semana"

#### Scenario: App aberto recentemente
- **WHEN** o Prime Video foi aberto há 2 horas e não se encaixa nos casos anteriores
- **THEN** o slide mostra "Aberto há 2 horas"

### Requirement: Slide de app recém-instalado
Quando existe um app instalado há menos de 7 dias que não está no carrossel, o carrossel SHALL incluir um slide "Novo: <nome do app>" para o app instalado mais recentemente.

#### Scenario: Nova instalação
- **WHEN** o usuário instalou o Globoplay ontem e ele não está no carrossel
- **THEN** o carrossel inclui um slide "Novo: Globoplay" com o botão "Abrir"

### Requirement: Rotação do carrossel
O carrossel SHALL avançar para o próximo slide no intervalo configurado (3 s, 5 s, 10 s, 30 s ou 1 min; padrão 5 s), voltando ao primeiro depois do último, e SHALL exibir indicadores de posição com o slide atual destacado. A rotação automática SHALL pausar enquanto o foco estiver no carrossel. Com o foco no carrossel, Esquerda/Direita SHALL trocar de slide manualmente.

#### Scenario: Rotação automática
- **WHEN** o intervalo é 5 s, o foco está nos favoritos e há 3 slides
- **THEN** o slide visível muda a cada 5 segundos, e o indicador acompanha a posição

#### Scenario: Pausa com foco
- **WHEN** o foco está no botão "Abrir" do carrossel
- **THEN** o slide não muda sozinho

#### Scenario: Nenhum app selecionado instalado
- **WHEN** nenhum app do carrossel está instalado e não há instalações recentes
- **THEN** o carrossel mostra um único slide de boas-vindas do lume, que leva à grade de Apps

### Requirement: Linha de favoritos
A seção "Seus apps favoritos" SHALL exibir os favoritos configurados, na ordem configurada, como tiles de app: o banner de TV do app (ou, sem banner, o ícone sobre a cor do tile). Quando o tile mostra só o ícone, o nome do app SHALL aparecer dentro do tile, sob o ícone. A linha SHALL rolar horizontalmente quando houver mais favoritos do que cabem na tela. OK sobre um tile SHALL abrir o app.

#### Scenario: Favoritos padrão no primeiro uso
- **WHEN** o lume é aberto pela primeira vez com Netflix, YouTube e Spotify instalados
- **THEN** os favoritos são os apps conhecidos instalados, na ordem Netflix, YouTube, Prime Video, Spotify, Disney+, Max, Globoplay, limitados a 5

#### Scenario: Foco no tile
- **WHEN** o foco chega a um tile de favorito
- **THEN** o tile ganha o estilo de foco do tema (contorno e escala), com animação

#### Scenario: Sem favoritos
- **WHEN** a lista de favoritos está vazia
- **THEN** a linha mostra um único tile "Adicionar favoritos" que abre o modo edição
