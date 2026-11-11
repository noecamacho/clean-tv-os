# Spec Delta

## MODIFIED Requirements

### Requirement: Tema como subconjunto de CSS
O tema SHALL ser descrito em um arquivo CSS que aceita apenas: variáveis em `:root` (`--nome: valor`) e `var()`; seletores de componente fixos (`.app`, `.topbar`, `.tab`, `.clock`, `.icon-button`, `.button`, `.section-title`, `.tile`, `.tile-label`, `.dock`, `.dock-label`, `.editor-row`, `.chip`, `.panel`, `.breadcrumb`, `.settings-item`, `.toast`) com as pseudo-classes `:focus`, `:active` (selecionado) e `:disabled`; e as propriedades `background` (cor ou `linear-gradient`), `color`, `opacity`, `border-radius`, `outline` / `outline-offset`, `box-shadow` (uma sombra), `font-size`, `font-weight`, `letter-spacing`, `padding`, `gap`, `width`, `height`, `transform: scale()` e `transition` (duração e curva). Valores de comprimento SHALL estar em `px` do canvas lógico (ver `display-adaptation`).

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

### Requirement: Tema padrão lume
O app SHALL incluir o tema "lume", derivado do mockup: destaque azul `#4DB3FF`, tiles com cantos arredondados e cores de marca, tipografia sans-serif geométrica, o dock translúcido de cantos arredondados e, atrás de tudo, o papel de parede de aurora boreal (ver `wallpaper`), que substitui o fundo liso azul-petróleo, mantido como cor de reserva. Nas abas, o tema não define contorno de foco: o foco é indicado pelo sublinhado, azul com o foco na barra e branco com o foco no conteúdo. O tema SHALL nomear as cores por papel: `--primary` (azul `#4DB3FF`, a cor de foco e de controle), `--on-primary` e `--secondary` (branco `#F2F5F7`, a cor neutra e do texto), mantendo `--accent` e `--on-accent` como apelidos para temas antigos. Todo contorno de foco (tiles da grade de Apps, itens de Configurações, linhas do editor, chips, engrenagem) SHALL usar a cor primária, com o leve aumento de escala dos tiles.

#### Scenario: Comparação visual com o mockup
- **WHEN** as telas Apps e Editar são renderizadas a 1280x720 lógico com os mesmos dados do mockup
- **THEN** cores, espaçamentos, raios e tamanhos de fonte coincidem com os de `mockup/` dentro de uma tolerância de 2 px lógicos e de diferenças imperceptíveis de cor, exceto onde esta mudança redefine o visual (papel de parede, tamanho dos tiles de Apps e abas sem contorno)

#### Scenario: Início com dock
- **WHEN** a tela Início é renderizada com cinco favoritos
- **THEN** ela mostra o papel de parede, a barra superior e o dock translúcido embaixo, sem carrossel
