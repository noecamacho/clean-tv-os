# Spec Delta

## MODIFIED Requirements

### Requirement: Orçamento de memória em primeiro plano
Com o lume visível e ocioso na tela Início, o TOTAL PSS do processo (medido por `dumpsys meminfo`, incluindo Graphics/GL) SHALL ser de no máximo 64 MB no dispositivo de referência (TCL 55C715) com renderização em 1080p e 30 apps instalados. O limite subiu de 45 MB porque o papel de parede passou a ter 1920x1080 (textura de 6,2 MB, contra 0,7 MB) e o catálogo de 10 papéis.

#### Scenario: Medição em primeiro plano
- **WHEN** o lume está na tela Início há 30 segundos sem interação
- **THEN** `adb shell dumpsys meminfo <pacote>` reporta TOTAL PSS ≤ 64 MB

### Requirement: Tamanho do pacote
O APK de release SHALL ter no máximo 40 MB, contendo as duas arquiteturas suportadas. O limite subiu de 6 MB para acomodar os 10 papéis de parede de alta definição embutidos.

#### Scenario: Verificação no CI
- **WHEN** o CI gera o APK de release
- **THEN** o build falha se o arquivo passar de 40 MB

### Requirement: Retorno rápido
Ao pressionar Home, o primeiro quadro do Lume (o splash) SHALL aparecer, e o Lume SHALL estar respondendo ao controle, em até 300 ms quando o processo do lume ainda existe (retorno a quente) e em até 800 ms quando o processo foi encerrado (abertura a frio), no dispositivo de referência. A interface completa (fim da revelação do splash, ver `launcher-shell`) SHALL estar na tela no máximo 600 ms mais a duração da revelação depois do primeiro quadro. O Lume SHALL registrar no log o tempo do primeiro quadro e o da interface pronta.

#### Scenario: Retorno a quente
- **WHEN** o lume está em segundo plano, com recursos gráficos liberados, e o usuário pressiona Home
- **THEN** o primeiro quadro (o splash) aparece em até 300 ms, e a tela Início completa em até 960 ms depois dele

#### Scenario: Abertura a frio
- **WHEN** o processo do lume foi encerrado e o usuário pressiona Home
- **THEN** o primeiro quadro aparece em até 800 ms, usando o catálogo em cache sem esperar pela consulta completa de apps, e a tela Início completa em até 1200 ms depois dele

#### Scenario: Log da interface pronta
- **WHEN** a revelação do splash termina
- **THEN** o log do Lume registra o tempo desde a janela e desde o início do processo
