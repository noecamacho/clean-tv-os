## ADDED Requirements

### Requirement: Menu de contexto no dock
O menu de contexto de um favorito do dock (aberto pelo botão Menu, ver `launcher-shell`) SHALL ter, nesta ordem: "Mover" (só com dois favoritos ou mais), "Remover do Início" e "Informações do app" (abre a tela do app nas configurações do Android). O item "Adicionar favoritos" SHALL NOT ter menu de contexto. Toda mudança SHALL ser gravada como as do modo edição (`home-editor`).

"Mover" SHALL pôr o favorito em modo de mover, como no launcher do Android TV: os vizinhos se afastam e aparece uma seta de cada lado dele (nenhuma na ponta do dock). Esquerda e Direita SHALL levar o favorito uma posição por vez, com o foco junto; Cima, Baixo e Menu SHALL NOT fazer nada. OK SHALL manter a nova posição e gravá-la; Voltar ou Home SHALL devolvê-lo à posição de antes, sem gravar nada novo.

Remover SHALL tirar o favorito do dock e levar o foco ao favorito que ocupar a mesma posição, ou ao anterior, se era o último; sem favoritos, o dock SHALL mostrar o item "Adicionar favoritos".

Toda mudança de posição no dock (mover, remover, desinstalar) SHALL ser animada: cada ícone desliza da posição e do tamanho antigos para os novos em 240 ms, a barra acompanha os ícones, e o ícone removido encolhe e some no lugar em que estava.

#### Scenario: Mover para a direita
- **WHEN** o usuário escolhe "Mover" no favorito da posição 2, pressiona Direita duas vezes e OK
- **THEN** o favorito desliza até a posição 4, os que estavam no caminho voltam uma posição cada um, o foco segue o favorito movido e a nova ordem é gravada

#### Scenario: Cancelar o movimento
- **WHEN** o usuário está movendo um favorito e pressiona Voltar
- **THEN** o favorito volta à posição de antes, com o foco nele, e nada é gravado de novo

#### Scenario: Pontas do dock
- **WHEN** o favorito em movimento é o primeiro do dock
- **THEN** só aparece a seta da direita, e Esquerda não faz nada

#### Scenario: Remover com transição
- **WHEN** o usuário escolhe "Remover do Início" no favorito da posição 2
- **THEN** o ícone encolhe e some no lugar, os favoritos à direita deslizam para fechar o espaço e o foco fica no que ocupou a posição 2

#### Scenario: Remover o último favorito
- **WHEN** o dock tem um único favorito e o usuário escolhe "Remover do Início"
- **THEN** o dock passa a mostrar "Adicionar favoritos", com o foco nele
