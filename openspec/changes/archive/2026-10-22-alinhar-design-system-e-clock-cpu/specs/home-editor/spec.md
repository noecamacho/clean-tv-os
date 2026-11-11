# Spec Delta

## MODIFIED Requirements

### Requirement: Acesso ao modo edição
O modo edição SHALL ser aberto pelo item "Tela Início" da tela Configurações do Lume, ou pelo item "Adicionar favoritos" do dock quando não há favoritos. A tela SHALL mostrar o caminho "Configurações › Tela Início", o título "Tela Início" (sem subtítulo), o painel de aparência (seletor de papel de parede e linha "Favoritos redondos"), o painel de favoritos e, sob o painel, o botão secundário "Restaurar padrão" à esquerda. A saída da tela de edição com a retenção de todas as alterações feitas SHALL ser realizada exclusivamente pelo botão Voltar do controle remoto.

#### Scenario: Entrar e sair
- **WHEN** o usuário abre "Editar tela Início" e depois pressiona o botão Voltar
- **THEN** a tela anterior volta a ser exibida, mantendo todas as alterações feitas

#### Scenario: Voltar equivale a Concluído
- **WHEN** o usuário pressiona Voltar no nível principal do modo edição aberto pelas Configurações
- **THEN** as alterações já feitas são mantidas e a tela "Configurações" do Lume é exibida, com o foco em "Tela Início"

#### Scenario: Voltar de quem veio do dock
- **WHEN** o modo edição foi aberto por "Adicionar favoritos" do dock e o usuário pressiona Voltar
- **THEN** as alterações são mantidas e a tela Início é exibida
