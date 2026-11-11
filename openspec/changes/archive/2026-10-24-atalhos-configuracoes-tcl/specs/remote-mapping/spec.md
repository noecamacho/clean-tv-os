# Spec Delta: remote-mapping

## MODIFIED Requirements

### Requirement: Cada atalho abre a sua tela do sistema
Cada atalho de configuração SHALL abrir a sua tela do sistema, ou as Configurações gerais onde o firmware não a tem, e SHALL valer com o Lume em primeiro plano. O Lume SHALL descartar telas stub vazias e abrir a tela real de Armazenamento e o seletor do App de início padrão nos firmwares suportados.

#### Scenario: Cada atalho abre a sua tela
- **WHEN** uma tecla está associada a "Idioma" (ou a "Data e hora", "Armazenamento", "Teclado", "Legendas", "Localização", "Segurança e restrições", "Opções do desenvolvedor", "App de início padrão", "Sobre a TV") e o usuário a pressiona
- **THEN** o Lume abre a tela correspondente do sistema e não as Configurações gerais, onde o firmware a tem (conferido na AIWA e na TCL)

#### Scenario: Armazenamento não abre redefinição de fábrica
- **WHEN** uma tecla está associada a "Armazenamento" e o usuário a pressiona na TV TCL
- **THEN** o Lume abre o resumo de armazenamento (`StorageSummaryActivity`) e não a tela de redefinição de dados de fábrica (`StorageResetActivity`)

#### Scenario: App de início padrão ignora stub vazio
- **WHEN** uma tecla está associada a "App de início padrão" e o usuário a pressiona na TV TCL
- **THEN** o Lume ignora o `EmptyStubActivity` e abre o seletor de aplicativo padrão do sistema (`permissioncontroller`)

#### Scenario: Sleep abre tela com temporizador de desligamento
- **WHEN** uma tecla está associada a "Timer / Sleep da TV" e o usuário a pressiona na TV TCL
- **THEN** o Lume abre a tela de descanso do sistema (`DaydreamActivity`) com opções de Temporizador de desligamento e Desligar a tela
