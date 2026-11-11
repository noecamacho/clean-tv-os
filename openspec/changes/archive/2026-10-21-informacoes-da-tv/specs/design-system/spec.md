# design-system Specification Delta

## ADDED Requirements

### Requirement: Item de Informações da TV nas Configurações
A tela de Configurações SHALL incluir, na coluna esquerda de itens, a opção "Informações da TV" entre "Tela Início" e "Sobre o Lume", respeitando a altura de 72 px e o passo de 88 px na régua.

#### Scenario: Terceiro item de configurações
- **WHEN** a tela de Configurações é desenhada
- **THEN** a lista exibe três itens: "Tela Início", "Informações da TV" e "Sobre o Lume"

### Requirement: Layout vertical de Informações da TV
A tela Informações da TV SHALL utilizar a largura útil de 1152 px no canvas 1280x720, com títulos de seção na escala de 16 px e linhas de chave/valor na escala de 18 px, alinhadas à régua de 4 px, com máscara de corte abaixo do cabeçalho.

#### Scenario: Rótulo e valor na régua
- **WHEN** as especificações são renderizadas
- **THEN** as linhas mantêm altura uniforme, com rótulos em `--text-muted` e valores em `--text`
