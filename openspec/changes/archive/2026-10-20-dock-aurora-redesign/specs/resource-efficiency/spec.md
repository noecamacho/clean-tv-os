# Spec Delta

## MODIFIED Requirements

### Requirement: Renderização sob demanda
O lume SHALL desenhar quadros apenas quando algo visível muda (entrada do usuário, animação em curso, virada de minuto do relógio, mudança no catálogo). Ocioso, SHALL NOT desenhar quadros nem manter threads acordadas em laço. O papel de parede e o dock SHALL NOT manter animação contínua.

#### Scenario: Tela ociosa
- **WHEN** a tela Início está visível, sem interação
- **THEN** o lume desenha no máximo 1 quadro por minuto (relógio), e o uso de CPU do processo fica abaixo de 1% em média

#### Scenario: Outras telas ociosas
- **WHEN** a aba Apps, as Configurações, a tela Sobre ou o editor estão visíveis, sem interação
- **THEN** o lume desenha no máximo 1 quadro por minuto (relógio)

#### Scenario: Magnification terminada
- **WHEN** a animação de Magnification do dock termina
- **THEN** o lume deixa de desenhar quadros até a próxima mudança visível

### Requirement: Fluidez da navegação
Animações de foco, rolagem e Magnification do dock SHALL rodar a 60 quadros por segundo no dispositivo de referência, com no máximo 1% de quadros perdidos durante a navegação contínua, mesmo com o papel de parede desenhado atrás do conteúdo.

#### Scenario: Rolagem da linha de favoritos
- **WHEN** o usuário mantém a seta Direita pressionada sobre os 12 favoritos do dock
- **THEN** o `dumpsys gfxinfo`/medição de quadros mostra ≤ 1% de quadros perdidos

#### Scenario: Rolagem da grade de Apps
- **WHEN** o usuário mantém a seta Baixo pressionada na grade de Apps com 30 apps
- **THEN** a medição de quadros mostra ≤ 1% de quadros perdidos
