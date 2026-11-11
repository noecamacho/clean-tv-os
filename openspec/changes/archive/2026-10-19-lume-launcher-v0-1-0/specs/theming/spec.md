# Spec Delta

## Purpose

Define o contrato de tema do lume como um subconjunto de CSS, sem scripts, e o tema padrão "lume" derivado do mockup, compilado no app nesta versão.

## ADDED Requirements

### Requirement: Tema como subconjunto de CSS
O tema SHALL ser descrito em um arquivo CSS que aceita apenas: variáveis em `:root` (`--nome: valor`) e `var()`; seletores de componente fixos (`.app`, `.topbar`, `.tab`, `.clock`, `.icon-button`, `.hero`, `.hero-badge`, `.hero-title`, `.hero-subtitle`, `.button`, `.pager-dot`, `.section-title`, `.tile`, `.tile-label`, `.editor-row`, `.chip`, `.panel`, `.breadcrumb`, `.settings-item`, `.toast`) com as pseudo-classes `:focus`, `:active` (selecionado) e `:disabled`; e as propriedades `background` (cor ou `linear-gradient`), `color`, `opacity`, `border-radius`, `outline` / `outline-offset`, `box-shadow` (uma sombra), `font-size`, `font-weight`, `letter-spacing`, `padding`, `gap`, `width`, `height`, `transform: scale()` e `transition` (duração e curva). Valores de comprimento SHALL estar em `px` do canvas lógico (ver `display-adaptation`).

#### Scenario: Tema válido
- **WHEN** o tema usa apenas a sintaxe e as propriedades suportadas
- **THEN** o build termina sem avisos de tema

#### Scenario: Propriedade não suportada
- **WHEN** o tema contém `filter: blur(4px)` em `.tile`
- **THEN** o build emite um aviso identificando arquivo, linha e propriedade, e a propriedade é ignorada

#### Scenario: Erro de sintaxe
- **WHEN** o tema tem um bloco sem `}` de fechamento
- **THEN** o build falha com uma mensagem indicando arquivo e linha

### Requirement: Sem código executável em temas
O formato de tema SHALL NOT aceitar scripts, `@import`, URLs externas ou qualquer construção que execute código ou acesse a rede.

#### Scenario: Tentativa de importar
- **WHEN** o tema contém `@import url(https://exemplo.com/a.css)`
- **THEN** o build emite um aviso e ignora a regra, sem nenhum acesso à rede

### Requirement: Tema padrão lume
O app SHALL incluir o tema "lume", que reproduz o mockup: fundo escuro azul-petróleo, destaque azul `#4DB3FF`, tiles com cantos arredondados e cores de marca, foco com contorno branco e leve aumento de escala, tipografia sans-serif geométrica e o gradiente roxo escuro no hero.

#### Scenario: Comparação visual com o mockup
- **WHEN** as telas Início, Apps e Editar são renderizadas a 1280x720 lógico com os mesmos dados do mockup
- **THEN** cores, espaçamentos, raios e tamanhos de fonte coincidem com os de `mockup/` dentro de uma tolerância de 2 px lógicos e de diferenças imperceptíveis de cor

### Requirement: Tema único na v0.1.0
A v0.1.0 SHALL usar exclusivamente o tema embutido no build, sem opção de trocar de tema na interface e sem carregar arquivos de tema em tempo de execução.

#### Scenario: Sem seletor de tema
- **WHEN** o usuário percorre as Configurações do lume e o modo edição
- **THEN** não existe opção de trocar de tema
