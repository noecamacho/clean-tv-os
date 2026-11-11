# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

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
