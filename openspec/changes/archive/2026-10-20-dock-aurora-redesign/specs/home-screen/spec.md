# Spec Delta

## ADDED Requirements

### Requirement: Conteúdo da tela Início
A tela Início SHALL exibir, sobre o papel de parede, a barra superior e, na parte de baixo, o dock com os apps favoritos. A tela Início SHALL NOT exibir carrossel, título de seção nem outros conteúdos além do dock.

#### Scenario: Primeiro uso
- **WHEN** o lume é aberto pela primeira vez
- **THEN** a tela Início mostra o papel de parede, a barra superior e, embaixo, o dock com os favoritos padrão

### Requirement: Dock de favoritos
Os favoritos configurados SHALL ser exibidos na ordem configurada num dock: uma barra translúcida de cantos arredondados, flutuante, centralizada horizontalmente e acima da borda inferior da área segura. Cada favorito SHALL aparecer como o ícone do app em formato quadrado de cantos arredondados, sobre a cor do app quando o ícone tem transparência. O dock SHALL mostrar todos os favoritos (até 12) lado a lado, sem rolagem horizontal, e SHALL ficar dentro da área segura. OK sobre um favorito SHALL abrir o app.

#### Scenario: Favoritos padrão no primeiro uso
- **WHEN** o lume é aberto pela primeira vez com Netflix, YouTube e Spotify instalados
- **THEN** o dock mostra os apps conhecidos instalados, na ordem Netflix, YouTube, Prime Video, Spotify, Disney+, Max, Globoplay, limitados a 5

#### Scenario: Doze favoritos
- **WHEN** existem 12 favoritos
- **THEN** todos aparecem no dock ao mesmo tempo, dentro da área segura e sem rolagem, mesmo com o item focado ampliado

#### Scenario: Abrir pelo dock
- **WHEN** o foco está num ícone do dock e o usuário pressiona OK
- **THEN** o app daquele ícone é aberto

### Requirement: Magnification do dock
O item do dock em foco SHALL ser ampliado, com a base alinhada à base do dock, e os itens vizinhos SHALL ser ampliados em proporção menor, decrescente com a distância ao item focado, até voltar ao tamanho normal. A ampliação SHALL ser animada ao mover o foco. Itens ampliados MAY se sobrepor aos vizinhos sem deslocá-los. O dock SHALL NOT desenhar contorno ao redor do item focado.

#### Scenario: Foco no item do meio
- **WHEN** o foco chega ao terceiro de cinco itens
- **THEN** o terceiro item fica o maior, o segundo e o quarto ficam menores que ele e maiores que o normal, e o primeiro e o quinto permanecem no tamanho normal

#### Scenario: Mover o foco
- **WHEN** o usuário pressiona Direita com o foco no terceiro item
- **THEN** a ampliação passa ao quarto item com animação, sem salto

#### Scenario: Sem contorno
- **WHEN** um item do dock está em foco
- **THEN** não há contorno nem anel ao redor do item, apenas a ampliação

#### Scenario: Foco fora do dock
- **WHEN** o foco está na barra superior
- **THEN** todos os itens do dock aparecem no tamanho normal

### Requirement: Nome do app em foco
Com um item do dock em foco, o lume SHALL exibir o nome do app num balão acima do item ampliado, centralizado nele e dentro da tela. O balão SHALL desaparecer quando o foco sai do dock.

#### Scenario: Balão com o nome
- **WHEN** o foco chega ao ícone do Spotify
- **THEN** um balão com "Spotify" aparece acima do ícone

#### Scenario: Item na borda do dock
- **WHEN** o foco está no primeiro item e o nome é longo
- **THEN** o balão fica inteiro dentro da área segura, com o nome truncado se necessário

### Requirement: Navegação no dock
Com o foco no dock, Esquerda e Direita SHALL mover o foco entre os itens, sem avançar além do primeiro ou do último. Cima SHALL levar o foco à aba ativa da barra superior. Baixo SHALL NOT mudar o foco. Ao descer da barra superior, o foco SHALL ir ao primeiro item do dock.

#### Scenario: Limite do dock
- **WHEN** o foco está no último favorito e o usuário pressiona Direita
- **THEN** o foco permanece no último favorito

#### Scenario: Ir à barra superior
- **WHEN** o foco está no dock e o usuário pressiona Cima
- **THEN** o foco vai para a aba ativa da barra superior

#### Scenario: Descer da barra superior
- **WHEN** o foco está na barra superior e o usuário pressiona Baixo
- **THEN** o foco vai ao primeiro item do dock

### Requirement: Dock sem favoritos
Quando a lista de favoritos está vazia, o dock SHALL mostrar um único item "Adicionar favoritos", cujo nome aparece no balão quando em foco e que abre o modo edição com OK.

#### Scenario: Sem favoritos
- **WHEN** a lista de favoritos está vazia
- **THEN** o dock mostra um único item "Adicionar favoritos" que abre o modo edição

## REMOVED Requirements

### Requirement: Seções da tela Início
**Reason**: A tela Início passa a ter só o dock sobre o papel de parede; não há mais seções a ordenar, ocultar ou rolar.
**Migration**: Substituído por "Conteúdo da tela Início" e "Dock de favoritos".

### Requirement: Slides do carrossel
**Reason**: O carrossel foi removido da tela Início.
**Migration**: Nenhuma. O acesso aos apps passa pelo dock e pela aba Apps.

### Requirement: Linha de contexto do slide
**Reason**: A linha de contexto só existia nos slides do carrossel.
**Migration**: Nenhuma. O histórico local de uso continua sendo gravado.

### Requirement: Slide de app recém-instalado
**Reason**: Depende do carrossel, que foi removido.
**Migration**: Nenhuma. Apps novos aparecem na aba Apps.

### Requirement: Rotação do carrossel
**Reason**: O carrossel foi removido; nenhuma rotação automática permanece na tela Início.
**Migration**: Nenhuma.

### Requirement: Linha de favoritos
**Reason**: A linha de tiles 16:9 com título foi substituída pelo dock.
**Migration**: Substituído por "Dock de favoritos", "Magnification do dock" e "Dock sem favoritos".
