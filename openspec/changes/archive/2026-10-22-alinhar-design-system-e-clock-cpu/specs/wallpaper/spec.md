# Spec Delta

## MODIFIED Requirements

### Requirement: Papel de parede de aurora boreal
O lume SHALL desenhar, atrás do conteúdo de todas as telas (Início, Apps, Configurações, Sobre e edição), o papel de parede selecionado pelo usuário a partir de um catálogo integrado de 6 opções (ou a Aurora boreal por padrão), cobrindo toda a área visível. A imagem SHALL respeitar a paleta do tema Lume (tons escuros de azul-petróleo, ardósia, índigo e ciano), com topo escuro sob a barra superior e metade inferior escura para acomodar o dock.

#### Scenario: Papel de parede na tela Início
- **WHEN** a tela Início é exibida
- **THEN** o papel de parede ativo aparece atrás da barra superior e do dock, ocupando a tela inteira, com a parte de baixo escura

#### Scenario: Papel de parede nas outras telas
- **WHEN** o usuário abre a aba Apps, as Configurações, a tela Sobre ou o modo edição
- **THEN** o mesmo papel de parede ativo aparece atrás do conteúdo

#### Scenario: Proporção diferente de 16:9
- **WHEN** a tela tem uma proporção diferente de 16:9
- **THEN** o papel de parede cobre toda a superfície visível, inclusive as sobras laterais ou verticais fora do canvas lógico, sem faixas vazias
