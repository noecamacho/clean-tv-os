# Proposta: Compatibilidade de Atalhos de Configurações para TV TCL e AIWA

## Por quê

Os atalhos de configurações do sistema mapeados no controle remoto funcionam perfeitamente na TV AIWA (Android 11, KTC/MediaTek), mas na TV TCL (Android 11, Realtek BeyondTV2) a maioria das ações cai incorretamente nas Configurações Gerais do Android TV ou falha:
1. O atalho de **Armazenamento** abria a tela de Redefinição/Reset de Fábrica (`StorageResetActivity`) em vez da tela de detalhes de Armazenamento (`StorageSummaryActivity`).
2. O atalho de **App de início padrão** era interceptado por um stub vazio da TCL (`EmptyStubActivity`), que fechava instantaneamente sem abrir o seletor.
3. A tela de **Som** sofria crash por classe inexistente na ROM da TCL (`ClassNotFoundException: SoundActivity`).
4. Telas específicas da AIWA na cascata (como Timer e Imagem) não tinham correspondência na TCL e caíam no fallback geral.

É necessário unificar a cascata de resolução de intents no `LumeBridge` para garantir compatibilidade completa e segura com ambos os modelos de TV de referência, sem quebrar o comportamento já homologado na AIWA.

## O que muda

- **Detecção de Stubs**: Atualizar `leadsToScreen` para descartar activities stub vazias (como `EmptyStubActivity`), impedindo que intents não implementados por um firmware finjam ter resolvido a ação.
- **Armazenamento seguro**: Priorizar a action `"com.android.tv.settings.STORAGE"` e a classe `com.android.tv.settings.device.StorageSummaryActivity` antes de qualquer intent genérico de armazenamento, exibindo a partição de disco e uso em vez do diálogo de reset.
- **Seletor de App de Início Padrão**: Apontar prioritariamente para o pacote `com.google.android.permissioncontroller` (`DefaultAppActivity`), funcional tanto na TCL quanto na AIWA.
- **Tratamento de exceções resiliente**: Capturar `Exception` geral na inicialização de activities para evitar que falhas de ROM (como `ClassNotFoundException`) fechem ou travem a execução do Lume.
- **Cascata mista de TV**: Preservar os intents dedicados da AIWA no topo da cascata e adicionar as alternativas universais/TCL para manter 100% de retrocompatibilidade com a AIWA e pleno suporte à TCL.

## Capacidades

### Modified Capabilities

- `remote-mapping`: Atualizar os cenários do requisito "Cada atalho abre a sua tela do sistema" para cobrir a resolução em múltiplos firmwares (conferido na AIWA e na TCL), garantindo que Armazenamento abra a tela de estatísticas de memória/disco e App de Início abra o seletor de launcher.

## Impacto

- **Java Bridge**: `android/app/src/main/java/app/lume/tv/LumeBridge.java` (métodos `leadsToScreen` e `openSetting`).
- **Documentação de dispositivos**: `docs/device-notes.md` com as anotações e medições da TCL adicionadas à seção de atalhos.
- **Especificações**: `specs/remote-mapping/spec.md` atualizada com o suporte a múltiplos firmwares.
