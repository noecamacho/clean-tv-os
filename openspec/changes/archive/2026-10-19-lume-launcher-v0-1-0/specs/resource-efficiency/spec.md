# Spec Delta

## Purpose

Torna mensurável o requisito central do lume, que é consumir o mínimo de memória e de processamento em TVs com 2 GB de RAM, sobretudo quando outro app (ex.: um player de vídeo) está em primeiro plano.

## ADDED Requirements

### Requirement: Orçamento de memória em primeiro plano
Com o lume visível e ocioso na tela Início, o TOTAL PSS do processo (medido por `dumpsys meminfo`, incluindo Graphics/GL) SHALL ser de no máximo 45 MB no dispositivo de referência (TCL 55C715) com renderização em 1080p e 30 apps instalados.

#### Scenario: Medição em primeiro plano
- **WHEN** o lume está na tela Início há 30 segundos sem interação
- **THEN** `adb shell dumpsys meminfo <pacote>` reporta TOTAL PSS ≤ 45 MB

### Requirement: Orçamento de memória em segundo plano
Quando outro app passa ao primeiro plano, o lume SHALL liberar superfície, contexto gráfico, texturas e atlas, e o TOTAL PSS do processo SHALL cair para no máximo 20 MB em até 10 segundos.

#### Scenario: Assistindo a outro app
- **WHEN** o usuário abre o YouTube pelo lume e espera 10 segundos
- **THEN** `dumpsys meminfo` do lume reporta TOTAL PSS ≤ 20 MB, sem memória de GL/Graphics atribuída

#### Scenario: Pressão de memória
- **WHEN** o sistema envia um aviso de pouca memória (trim memory) com o lume em segundo plano
- **THEN** o lume libera todos os caches recriáveis, mantendo só a configuração e o catálogo em memória

### Requirement: Retorno rápido
Ao pressionar Home, a tela Início SHALL estar desenhada e respondendo ao controle em até 300 ms quando o processo do lume ainda existe (retorno a quente) e em até 800 ms quando o processo foi encerrado (abertura a frio), no dispositivo de referência.

#### Scenario: Retorno a quente
- **WHEN** o lume está em segundo plano, com recursos gráficos liberados, e o usuário pressiona Home
- **THEN** o primeiro quadro completo da tela Início aparece em até 300 ms

#### Scenario: Abertura a frio
- **WHEN** o processo do lume foi encerrado e o usuário pressiona Home
- **THEN** o primeiro quadro completo aparece em até 800 ms, usando o catálogo em cache sem esperar pela consulta completa de apps

### Requirement: Renderização sob demanda
O lume SHALL desenhar quadros apenas quando algo visível muda (entrada do usuário, animação em curso, troca de slide do carrossel, virada de minuto do relógio, mudança no catálogo). Ocioso, SHALL NOT desenhar quadros nem manter threads acordadas em laço.

#### Scenario: Tela ociosa
- **WHEN** a tela Início está visível, sem interação, com o carrossel desligado
- **THEN** o lume desenha no máximo 1 quadro por minuto (relógio), e o uso de CPU do processo fica abaixo de 1% em média

#### Scenario: Outras telas ociosas
- **WHEN** a aba Apps, as Configurações, a tela Sobre ou o editor estão visíveis, sem interação, mesmo com o foco fora do carrossel
- **THEN** o lume desenha no máximo 1 quadro por minuto (relógio), e o carrossel não mantém nenhum prazo em segundo plano

### Requirement: Fluidez da navegação
Animações de foco, rolagem e transição de slide SHALL rodar a 60 quadros por segundo no dispositivo de referência, com no máximo 1% de quadros perdidos durante a navegação contínua.

#### Scenario: Rolagem da linha de favoritos
- **WHEN** o usuário mantém a seta Direita pressionada sobre 12 favoritos
- **THEN** o `dumpsys gfxinfo`/medição de quadros mostra ≤ 1% de quadros perdidos

### Requirement: Tamanho do pacote
O APK de release SHALL ter no máximo 6 MB, contendo as duas arquiteturas suportadas.

#### Scenario: Verificação no CI
- **WHEN** o CI gera o APK de release
- **THEN** o build falha se o arquivo passar de 6 MB
