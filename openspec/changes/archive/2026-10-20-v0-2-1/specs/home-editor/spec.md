# Spec Delta

## MODIFIED Requirements

### Requirement: Acesso ao modo edição
O modo edição SHALL ser aberto pelo item "Tela Início" da tela Configurações do lume (engrenagem), ou pelo item "Adicionar favoritos" do dock quando não há favoritos. A tela SHALL mostrar o caminho "Configurações › Tela Início", o título "Tela Início", uma instrução curta e, sob o painel de favoritos, os botões "Restaurar padrão" (à esquerda) e "Concluído" (à direita).

#### Scenario: Entrar e sair
- **WHEN** o usuário abre "Editar tela Início" e depois seleciona "Concluído"
- **THEN** a tela Início volta a ser exibida, já com as alterações feitas

#### Scenario: Voltar equivale a Concluído
- **WHEN** o usuário pressiona Voltar no nível principal do modo edição aberto pelas Configurações
- **THEN** as alterações já feitas são mantidas e a tela "Configurações" do Lume é exibida, com o foco em "Tela Início"

#### Scenario: Voltar de quem veio do dock
- **WHEN** o modo edição foi aberto por "Adicionar favoritos" do dock e o usuário pressiona Voltar
- **THEN** as alterações são mantidas e a tela Início é exibida

## ADDED Requirements

### Requirement: Botões do modo edição
Os botões "Restaurar padrão" (secundário) e "Concluído" (primário) SHALL ficar na mesma linha sob o painel de favoritos, em lados opostos da tela: "Restaurar padrão" na margem esquerda e "Concluído" na direita, ambos com 48 de altura. Ao abrir o modo edição o foco SHALL começar no primeiro favorito (ou no primeiro app a adicionar, se não há favoritos). Pressionar Baixo a partir dos favoritos e dos apps a adicionar SHALL levar o foco a "Concluído"; chegar a "Restaurar padrão" SHALL exigir Esquerda a partir de "Concluído", e Direita volta a ele. Cima a partir da linha de botões SHALL voltar aos apps a adicionar.

#### Scenario: Baixo vai para Concluído
- **WHEN** o foco está nos apps a adicionar e o usuário pressiona Baixo na última linha
- **THEN** o foco vai para "Concluído", e "Restaurar padrão" não recebe foco

#### Scenario: Restaurar pela esquerda
- **WHEN** o foco está em "Concluído" e o usuário pressiona Esquerda
- **THEN** o foco vai para "Restaurar padrão", e Direita o devolve a "Concluído"

#### Scenario: Foco inicial
- **WHEN** o modo edição é aberto com favoritos
- **THEN** o foco está no botão de mover à direita do primeiro favorito
