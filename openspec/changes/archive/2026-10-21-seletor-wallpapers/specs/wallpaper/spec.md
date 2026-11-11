# Spec Delta

## MODIFIED Requirements

### Requirement: Papel de parede de aurora boreal
O lume SHALL desenhar, atrás do conteúdo de todas as telas (Início, Apps, Configurações, Sobre e edição), o papel de parede selecionado pelo usuário a partir de um catálogo integrado de 10 opções (ou a Aurora boreal por padrão), cobrindo toda a área visível. A imagem SHALL respeitar a paleta do tema Lume (tons escuros de azul-petróleo, ardósia, índigo e ciano), com topo escuro sob a barra superior e metade inferior escura para acomodar o dock.

#### Scenario: Papel de parede na tela Início
- **WHEN** a tela Início é exibida
- **THEN** o papel de parede ativo aparece atrás da barra superior e do dock, ocupando a tela inteira, com a parte de baixo escura

#### Scenario: Papel de parede nas outras telas
- **WHEN** o usuário abre a aba Apps, as Configurações, a tela Sobre ou o modo edição
- **THEN** o mesmo papel de parede ativo aparece atrás do conteúdo

#### Scenario: Proporção diferente de 16:9
- **WHEN** a tela tem uma proporção diferente de 16:9
- **THEN** o papel de parede cobre toda a superfície visível, inclusive as sobras laterais ou verticais fora do canvas lógico, sem faixas vazias

## REMOVED Requirements

### Requirement: Papel de parede único e não configurável
**Reason**: O Lume passa a oferecer um catálogo de 10 papéis de parede configuráveis pelo usuário no modo edição.
**Migration**: Utilizar o seletor de papéis de parede em Configurações › Tela Início.

## ADDED Requirements

### Requirement: Catálogo de papéis de parede de alta definição
O lume SHALL oferecer um catálogo de 10 papéis de parede de alta definição embutidos em formato Raw 24-bit RGB linear, cada um em 1920x1080 (Full HD; ampliado pela GPU em telas 4K), com dithering nos degradês escuros para não formar faixas. O catálogo inclui a Aurora clássica, paisagens de realismo e natureza noturna, e composições abstratas minimalistas. Cada papel de parede SHALL possuir um identificador estável para persistência e um rótulo traduzido.

#### Scenario: Contraste do topo com os detalhes da imagem
- **WHEN** o papel de parede é mostrado sem escurecimento atrás da barra superior
- **THEN** o pixel mais claro atrás dos textos da barra mantém contraste de pelo menos 4,5:1 com o texto esmaecido, e não só a média da faixa

#### Scenario: Listagem de papéis de parede no editor
- **WHEN** o usuário abre o seletor de papéis de parede no modo edição
- **THEN** as 10 opções aparecem disponíveis com suas miniaturas de prévia e rótulos

### Requirement: Troca dinâmica de papel de parede
A seleção de papel de parede SHALL ser dinâmica: ao escolher um papel de parede no modo edição, o fundo do aplicativo SHALL ser atualizado imediatamente e a escolha SHALL ser persistida no arquivo de configuração do usuário em `config.v1`.

#### Scenario: Troca de papel de parede no modo edição
- **WHEN** o usuário seleciona um novo papel de parede no modo edição
- **THEN** o fundo da interface é atualizado imediatamente para a nova imagem e a preferência é gravada em `config.v1`

#### Scenario: Inicialização com papel de parede salvo
- **WHEN** o lume é iniciado e existe um papel de parede válido configurado em `config.v1`
- **THEN** o papel de parede configurado é carregado e exibido atrás de todas as telas

#### Scenario: Padrão no primeiro uso ou configuração ausente
- **WHEN** o lume é aberto pela primeira vez ou o arquivo de configuração não especifica o papel de parede
- **THEN** o papel de parede padrão Aurora é utilizado automaticamente
