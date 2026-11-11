## MODIFIED Requirements

### Requirement: Tamanho mínimo de texto
Nenhum texto da interface SHALL ter tamanho menor que 16 unidades lógicas. A escala uniforme (canvas lógico × `min(largura/1280, altura/720)`) SHALL manter a mesma composição em qualquer resolução 16:9, e em outras proporções o conteúdo SHALL ficar centralizado dentro da área segura, sem texto cortado nem novas reticências.

#### Scenario: Legenda pequena
- **WHEN** o tema define o rótulo de tile com 13px
- **THEN** o build emite um aviso, e o texto é renderizado com 16 unidades

#### Scenario: Mesma composição em 720p, 1080p e 4K
- **WHEN** a mesma tela é exibida em 1280x720, 1920x1080 e 3840x2160 (limitado a 1920x1080)
- **THEN** as posições e a quebra de linhas em unidades lógicas são idênticas, e só o fator de escala muda

#### Scenario: Proporção diferente de 16:9
- **WHEN** a superfície é 1024x768 (4:3) ou 2560x1080 (21:9)
- **THEN** o conteúdo fica centralizado, com o fundo do tema nas sobras, e todo texto continua dentro da área segura
