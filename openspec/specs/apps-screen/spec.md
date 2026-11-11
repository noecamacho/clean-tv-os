# apps-screen Specification

## Purpose
Define a aba Apps do lume: uma grade com todos os apps instalados que podem ser abertos, conforme o mockup "Todos os apps".

## Requirements

### Requirement: Grade de todos os apps
A aba Apps SHALL exibir o título "Todos os apps" e uma grade de 6 colunas com todos os apps do catálogo, em ordem alfabética do nome de exibição (ignorando maiúsculas e acentos), sobre o papel de parede. Cada item SHALL mostrar o tile do app (o banner de TV ou, sem banner, o ícone sobre a cor do tile; o nome dentro do tile enquanto a imagem não está disponível), e o nome abaixo do tile. Os tiles SHALL ter cerca de 172x97 unidades lógicas (proporção 16:9, cerca de 18% menores que os 211x119 da versão anterior), de modo que as 6 colunas ocupem a largura da área de conteúdo, sem sobra lateral.

#### Scenario: Ordem alfabética
- **WHEN** estão instalados YouTube, Netflix e Globoplay
- **THEN** a grade os mostra na ordem Globoplay, Netflix, YouTube

#### Scenario: Ordem segue o nome mostrado
- **WHEN** o idioma do sistema muda e os nomes dos apps mudam com ele (por exemplo "Settings" e "Configurações")
- **THEN** a grade é reordenada alfabeticamente pelos nomes do idioma atual, sem diferenciar maiúsculas nem acentos

#### Scenario: Muitos apps
- **WHEN** existem mais apps do que cabem na tela
- **THEN** a grade rola verticalmente, mantendo a linha focada visível

#### Scenario: Tamanho dos tiles
- **WHEN** a aba Apps é exibida a 1280x720 lógico
- **THEN** a grade tem 6 colunas, cada tile mede cerca de 172x97 unidades lógicas, e as colunas ocupam toda a largura da área de conteúdo

### Requirement: Navegação na grade
Na grade, as setas SHALL mover o foco entre células vizinhas. Cima na primeira linha SHALL levar o foco à barra superior. Direita na última coluna SHALL NOT pular para a linha seguinte. OK SHALL abrir o app focado.

#### Scenario: Limite da linha
- **WHEN** o foco está na 6ª coluna e o usuário pressiona Direita
- **THEN** o foco permanece no mesmo tile

#### Scenario: Última linha incompleta
- **WHEN** o foco está na penúltima linha, na coluna 5, e a última linha tem só 2 itens
- **THEN** pressionar Baixo leva o foco ao último item da última linha

### Requirement: Tile do lume na grade
A grade SHALL incluir um tile do próprio lume, tratado como mais um app: ele SHALL ter o mesmo formato retangular dos demais, ocupar o seu lugar na ordem alfabética (o nome é "Lume" em qualquer idioma), mostrar o logo do lume (o L e o ponto) sobre a cor do logo e exibir o nome "Lume" sob o tile. OK sobre ele SHALL abrir a tela de Configurações do lume; as Configurações do Android continuam acessíveis por essa tela e pelo app de configurações do sistema, quando ele aparece na grade.

#### Scenario: Tile do lume
- **WHEN** a aba Apps é exibida
- **THEN** o tile "Lume" aparece entre os apps, em ordem alfabética, retangular como os outros, com o logo do lume no centro

#### Scenario: Abrir as configurações do lume
- **WHEN** o usuário seleciona o tile "Lume"
- **THEN** a tela de Configurações do lume é aberta e Voltar leva de volta à aba Apps

### Requirement: Nome do app em foco
O nome sob o tile em foco SHALL aparecer em branco (a cor do texto principal), como a aba ativa, e voltar ao cinza claro quando o foco sai, acompanhando a animação do foco.

#### Scenario: Nome em destaque
- **WHEN** o foco está num tile da grade
- **THEN** o nome sob esse tile é branco e os nomes dos demais são cinza claros

### Requirement: Foco nos tiles da grade
O tile em foco SHALL ter um contorno na cor primária do tema (o azul `#4DB3FF`, a mesma do sublinhado da aba) e leve aumento de escala, em vez do contorno branco da versão anterior.

#### Scenario: Contorno azul
- **WHEN** o foco está num tile da grade, inclusive de um app azul como o Prime Video
- **THEN** o tile mostra um contorno azul claro, visível sobre a cor do tile, e fica um pouco maior

### Requirement: Menu de contexto na grade
O menu de contexto de um app da grade (aberto pelo botão Menu, ver `launcher-shell`) SHALL ter, nesta ordem:

- "Adicionar ao Início", se o app não é favorito, ou "Remover do Início", se é. Com 12 favoritos, "Adicionar ao Início" SHALL aparecer desabilitado, com a indicação "Limite de 12 favoritos", e SHALL NOT receber foco.
- "Informações do app", que SHALL abrir a tela do app nas configurações do Android.
- "Desinstalar", apenas para apps que não são do sistema, que SHALL abrir o desinstalador do Android para o app.

Adicionar SHALL colocar o app no fim dos favoritos; remover SHALL tirá-lo dos favoritos. As duas mudanças SHALL ser gravadas como as do modo edição (`home-editor`) e o foco SHALL voltar ao tile do app. O tile "Lume" SHALL NOT ter menu de contexto. Quando um app desinstalado sai do catálogo, a grade SHALL ser atualizada e o foco SHALL ir para o tile que ocupar a mesma posição, ou para o último, se não houver.

#### Scenario: Adicionar ao Início
- **WHEN** o usuário abre o menu sobre o "Chrome", que não é favorito, e escolhe "Adicionar ao Início"
- **THEN** o Chrome entra no fim do dock, a mudança é gravada e o foco volta ao tile do Chrome

#### Scenario: Remover do Início
- **WHEN** o usuário abre o menu sobre o "YouTube", que é favorito
- **THEN** o primeiro item é "Remover do Início", e escolhê-lo tira o YouTube do dock

#### Scenario: Limite de favoritos
- **WHEN** já existem 12 favoritos e o usuário abre o menu sobre um app que não é favorito
- **THEN** "Adicionar ao Início" aparece desabilitado com "Limite de 12 favoritos", e o foco começa em "Informações do app"

#### Scenario: App do sistema
- **WHEN** o usuário abre o menu sobre um app do sistema (ex.: Configurações)
- **THEN** o menu não tem "Desinstalar"

#### Scenario: Desinstalar
- **WHEN** o usuário escolhe "Desinstalar" no menu de um app instalado pelo usuário e confirma no diálogo do Android
- **THEN** o app sai da grade e do dock, e o foco vai para o tile na mesma posição

#### Scenario: Tile do Lume
- **WHEN** o foco está no tile "Lume" e o usuário pressiona Menu
- **THEN** nenhum menu aparece
