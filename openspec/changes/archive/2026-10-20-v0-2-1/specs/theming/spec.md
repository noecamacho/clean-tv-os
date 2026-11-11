# Spec Delta

## MODIFIED Requirements

### Requirement: Tema como subconjunto de CSS
O tema SHALL ser descrito em um arquivo CSS que aceita apenas: variáveis em `:root` (`--nome: valor`) e `var()`; seletores de componente fixos (`.app`, `.topbar`, `.tab`, `.clock`, `.icon-button`, `.button`, `.button-secondary`, `.button-danger`, `.section-title`, `.tile`, `.tile-label`, `.dock`, `.dock-label`, `.editor-row`, `.chip`, `.panel`, `.breadcrumb`, `.settings-item`, `.toast`) com as pseudo-classes `:focus`, `:active` (selecionado) e `:disabled`; e as propriedades `background` (cor ou `linear-gradient`), `color`, `opacity`, `border-radius`, `outline` / `outline-offset`, `box-shadow` (uma sombra), `font-size`, `font-weight`, `letter-spacing`, `padding`, `gap`, `width`, `height`, `transform: scale()` e `transition` (duração e curva). Valores de comprimento SHALL estar em `px` do canvas lógico (ver `display-adaptation`).

#### Scenario: Tema válido
- **WHEN** o tema usa apenas a sintaxe e as propriedades suportadas
- **THEN** o build termina sem avisos de tema

#### Scenario: Propriedade não suportada
- **WHEN** o tema contém `filter: blur(4px)` em `.tile`
- **THEN** o build emite um aviso identificando arquivo, linha e propriedade, e a propriedade é ignorada

#### Scenario: Erro de sintaxe
- **WHEN** o tema tem um bloco sem `}` de fechamento
- **THEN** o build falha com uma mensagem indicando arquivo e linha

#### Scenario: Seletor do carrossel
- **WHEN** o tema contém `.hero` ou `.pager-dot`
- **THEN** o build emite um aviso de seletor não suportado e ignora a regra
