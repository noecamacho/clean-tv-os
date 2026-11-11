## MODIFIED Requirements

### Requirement: Retorno rápido
Ao pressionar Home, o primeiro quadro do Lume (o splash) SHALL aparecer, e o Lume SHALL estar respondendo ao controle, em até 300 ms quando o processo do lume ainda existe (retorno a quente) e em até 800 ms quando o processo foi encerrado (abertura a frio), no dispositivo de referência. A interface completa (fim da revelação do splash, ver `launcher-shell`) SHALL estar na tela no máximo 600 ms mais a duração da revelação depois do primeiro quadro. O Lume SHALL registrar no log o tempo do primeiro quadro e o da interface pronta.

#### Scenario: Retorno a quente
- **WHEN** o lume está em segundo plano, com recursos gráficos liberados, e o usuário pressiona Home
- **THEN** o primeiro quadro (o splash) aparece em até 300 ms, e a tela Início completa em até 820 ms depois dele

#### Scenario: Abertura a frio
- **WHEN** o processo do lume foi encerrado e o usuário pressiona Home
- **THEN** o primeiro quadro aparece em até 800 ms, usando o catálogo em cache sem esperar pela consulta completa de apps, e a tela Início completa em até 900 ms depois dele

#### Scenario: Log da interface pronta
- **WHEN** a revelação do splash termina
- **THEN** o log do Lume registra o tempo desde a janela e desde o início do processo
