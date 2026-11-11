# Spec Delta

## ADDED Requirements

### Requirement: Mini-cards de papel de parede no editor
Os mini-cards de papel de parede no modo edição SHALL ser desenhados no canvas lógico 1280x720 com proporção aproximada de 16:9, medindo 178 px de largura por 100 px de altura, com raio de cantos de 12 px (`--radius`) e vão horizontal de 16 px entre eles, dispostos em linha horizontal alinhada à margem esquerda de 64 px (`LEFT`).

#### Scenario: Mini-card em repouso
- **WHEN** o mini-card está sem foco
- **THEN** ele exibe a miniatura e o nome em `--text-muted` nas escalas `--font-caption` (16 px) ou `--font-label` (18 px)

#### Scenario: Mini-card em foco
- **WHEN** um mini-card de papel de parede recebe o foco do D-Pad
- **THEN** ele é contornado pela borda de foco azul `--primary` com escala de 1.05x e o nome em `--text` com peso 700

#### Scenario: Alinhamento horizontal na régua
- **WHEN** os 6 primeiros mini-cards são medidos na captura em 1280x720
- **THEN** a linha ocupa a largura útil de 1152 px com vãos regulares de 16 px

## MODIFIED Requirements

### Requirement: Splash
O splash SHALL ser só o logo do Lume (o quadrado arredondado com o L e o ponto, nas proporções de `logo.rs`) centralizado sobre `--bg`, com tamanho definido no tema, sem texto. O tema SHALL reservar abaixo do logo uma faixa para um indicador de carregamento futuro (posição e altura em tokens), que nesta versão fica vazia e não desloca o logo. O logo SHALL entrar com fade de 300 ms (ease-out) e crescimento de 0,96 a 1 no primeiro quadro do splash, sobre o mesmo `--bg` da janela inicial do sistema, e SHALL ficar à vista por no mínimo 600 ms (abertura a frio) ou 300 ms (retorno a quente) desde o primeiro quadro antes da revelação. A revelação SHALL ser um progresso linear dividido em etapas que se sobrepõem, cada uma com curva ease-in-out, de modo que logo e interface fiquem parcialmente visíveis ao mesmo tempo, sem um momento vazio entre eles: o papel de parede passa da intensidade 0 (só `--bg`) à intensidade normal, pela própria cor com que é desenhado; o logo some nos primeiros 60% da revelação, com fade e encolhendo até 0,92 do tamanho, aplicado só à área do logo; e a interface entra de 30% a 100% da revelação, com fade pela opacidade de cada elemento (como o fade das abas), com o conteúdo (dock ou grade) subindo 16 unidades e a barra superior parada; o papel de parede sobe de 0 a 80%. A revelação SHALL durar 600 ms na abertura a frio e 360 ms no retorno a quente. Um ícone que chegar com a interface na tela SHALL entrar com fade de 150 ms, ease-out (`--icon-in`), enquanto a cor e a inicial do app saem sob ele. Nenhuma etapa SHALL usar camada translúcida de tela cheia.

#### Scenario: Entrada do logo
- **WHEN** a janela do sistema é substituída pelo primeiro quadro do Lume
- **THEN** o fundo continua da mesma cor e o logo surge com fade, sem saltar

#### Scenario: Revelação sem camada de tela cheia
- **WHEN** a interface é revelada
- **THEN** o papel de parede ganha intensidade pelo próprio desenho, e só a área do logo é desenhada com transparência
