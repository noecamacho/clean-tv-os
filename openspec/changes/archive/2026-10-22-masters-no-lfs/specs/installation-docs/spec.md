## ADDED Requirements

### Requirement: Requisitos de instalação nos READMEs
Cada README (`README.md`, `README.pt-BR.md`, `README.es.md`, `README.id.md`, `README.ru.md`) SHALL abrir a seção de instalação com a lista de requisitos: uma TV com Android TV 9 ou superior na mesma rede do computador, o adb instalado no computador, a depuração USB ou de rede ativada na TV e o APK do Lume.

#### Scenario: Requisitos visíveis
- **WHEN** alguém abre a seção de instalação de qualquer README
- **THEN** vê a lista de requisitos antes das duas opções de instalação

### Requirement: Contexto sobre o adb
Cada README SHALL explicar em poucas linhas o que é o adb (Android Debug Bridge, a ferramenta de linha de comando do Google que conversa com a TV pela rede e instala apps nela) e por que é necessário (a TV não instala o Lume pela loja).

#### Scenario: Leitor sem experiência com adb
- **WHEN** alguém abre a seção de instalação de qualquer README
- **THEN** encontra a explicação do que é o adb antes dos passos

### Requirement: APK indicado pelo repositório
Cada README SHALL indicar o APK apenas pelo endereço do repositório, `https://github.com/douglaspands/lume-tv-launcher`, sem número de versão.

#### Scenario: Link do APK
- **WHEN** o README manda baixar o APK
- **THEN** aponta para `https://github.com/douglaspands/lume-tv-launcher` e não cita versão nem nome de arquivo com versão

### Requirement: Duas opções de instalação
Cada README SHALL oferecer duas opções, nesta ordem: a Opção 1, por um assistente de IA (um agente de linha de comando capaz de rodar comandos no computador), com um prompt pronto para colar; e a Opção 2, manual, com os passos atuais. A Opção 2 SHALL manter o conteúdo dos 6 passos existentes, ajustado apenas para não citar versão do APK.

#### Scenario: Escolha da opção
- **WHEN** o leitor chega à seção de instalação
- **THEN** encontra a Opção 1 (assistente de IA) antes da Opção 2 (manual), e as duas levam ao mesmo resultado

### Requirement: Prompt para assistente de IA
O prompt da Opção 1 SHALL estar escrito no idioma de cada README e SHALL instruir o assistente a verificar o adb, baixar o APK do repositório, pedir o IP da TV, conectar, instalar com `adb install`, confirmar `Success` e definir o Lume como Home com `adb shell cmd package set-home-activity app.lume.tv/.LumeActivity`.

#### Scenario: Passos do prompt
- **WHEN** o assistente segue o prompt
- **THEN** verifica o adb (e explica como instalá-lo, se faltar), baixa o APK, pede o IP e a confirmação da depuração, roda `adb connect` e `adb devices` lembrando o usuário de aceitar o aviso na TV, instala e confirma `Success`

#### Scenario: Endereço da TV
- **WHEN** o assistente precisa conectar
- **THEN** pede o IP ao usuário em vez de adivinhar

### Requirement: Paridade entre idiomas
Os 5 READMEs SHALL ter as mesmas seções de instalação, na mesma ordem e com os mesmos comandos; só o texto muda de idioma.

#### Scenario: Conferência
- **WHEN** a seção de instalação de um README muda
- **THEN** os outros 4 recebem a mesma mudança na mesma change
