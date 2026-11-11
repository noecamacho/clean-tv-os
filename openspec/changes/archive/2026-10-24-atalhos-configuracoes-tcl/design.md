# Design: Compatibilidade de Atalhos de Configurações para TV TCL e AIWA

## Context

O Lume implementa os atalhos de configurações do sistema em `LumeBridge.java` usando uma resolução em cascata (`openSetting`). Os intents foram inicialmente homologados na TV AIWA (Android 11, KTC/MediaTek). Na TV de referência TCL 55C715 (Android 11, Realtek BeyondTV2), a maioria dos atalhos caía nas Configurações Gerais ou abria telas indesejadas (como a de Reset de fábrica para Armazenamento).

Ver `proposal.md` para a motivação completa e `specs/remote-mapping/spec.md` para os requisitos.

## Goals / Non-Goals

**Goals:**
- Garantir que a TV TCL abra as telas de sistema correspondentes aos atalhos (Armazenamento, App de início padrão, Wi-Fi, Bluetooth, Apps, Idioma, Data e Hora, etc.).
- Proteger contra stubs vazios (`EmptyStubActivity`) que mascaram a ausência de implementação e impedem a cascata de tentar alternativas.
- Evitar que o atalho de Armazenamento abra a tela de Reset de fábrica na TCL.
- Manter 100% do comportamento e homologação existentes na TV AIWA sem regressão.

**Non-Goals:**
- Criar telas proprietárias novas para Imagem ou Som onde o firmware não expõe uma Activity pública para terceiros (o fallback continuará sendo as Configurações Gerais nesses casos).
- Modificar a camada Rust (`lume-core`); a correção reside estritamente na ponte Java (`LumeBridge.java`) e nas specs/documentação.

## Decisions

### 1. Filtragem de Activities Stub em `leadsToScreen`

**Decisão:**
Atualizar o método `leadsToScreen(Context, Intent)` em `LumeBridge.java` para descartar qualquer resolução cujo nome de classe termine em `EmptyStubActivity`:
```java
if (found.activityInfo.name != null && found.activityInfo.name.endsWith("EmptyStubActivity")) {
    return false;
}
```
**Por quê:** A TCL registra `com.android.tv.settings.EmptyStubActivity` para responder a intents não suportados pelo firmware (como `HOME_SETTINGS`). Como o pacote é `com.android.tv.settings`, a verificação anterior aceitava o intent, que ao ser lançado fechava imediatamente. Rejeitar stubs permite que a cascata prossiga.

### 2. Cascata Segura de Armazenamento (`storage`)

**Decisão:**
Adicionar a action `"com.android.tv.settings.STORAGE"` e o componente `device.StorageSummaryActivity` no início da cascata de armazenamento:
```java
case "storage":
    intents = new Intent[] {
        new Intent("com.android.tv.settings.STORAGE"),
        tvSettings("device.StorageSummaryActivity"),
        new Intent(Settings.ACTION_INTERNAL_STORAGE_SETTINGS),
        new Intent(Settings.ACTION_MEMORY_CARD_SETTINGS),
        new Intent(Settings.ACTION_SETTINGS)
    };
    break;
```
**Por quê:** Na TCL, `ACTION_INTERNAL_STORAGE_SETTINGS` aponta para `StorageResetActivity` (Redefinição de fábrica). A tela real de gerenciamento de disco é `StorageSummaryActivity`. Se a TV não possuir essa classe, o fallback mantém a lista original.

### 3. Direcionamento Explícito de App de Início Padrão (`home_app`)

**Decisão:**
Priorizar o pacote `com.google.android.permissioncontroller`:
```java
case "home_app":
    intents = new Intent[] {
        new Intent(Settings.ACTION_HOME_SETTINGS).setPackage("com.google.android.permissioncontroller"),
        new Intent(Settings.ACTION_HOME_SETTINGS),
        new Intent(Settings.ACTION_SETTINGS)
    };
    break;
```
**Por quê:** Tanto na AIWA quanto na TCL, o seletor de launcher padrão (`DefaultAppActivity`) reside em `com.google.android.permissioncontroller`. Na TCL, a query genérica é mascarada pelo `EmptyStubActivity`.

### 4. Tratamento Resiliente de Exceções ao Iniciar Activities

**Decisão:**
No laço de tentativas de `openSetting`, capturar `Exception` geral em vez de apenas `ActivityNotFoundException | SecurityException`:
```java
try {
    context.startActivity(intent);
    return "ok";
} catch (Exception ignored) {
}
```
**Por quê:** Em ROMs com defeito de compilação (como a TCL, que declara `SoundActivity` no manifesto mas não tem a classe no DEX, gerando `ClassNotFoundException`), o erro não encerra a aplicação nem aborta o fluxo, permitindo que a cascata tente o próximo intent ou o fallback geral.

### 5. Preservação Absoluta da AIWA

**Decisão:**
Todas as activities e actions prioritárias da AIWA (`partnercustomizer.timer.TimerActivity`, `android.settings.PQAQ.SETTINGS`, etc.) permanecem no topo de seus respectivos arrays.
**Por quê:** Na AIWA, essas activities existem e respondem com sucesso imediato no primeiro passo do laço. Na TCL, como essas classes não existem, falham rapidamente no `leadsToScreen` e continuam para os intents do Android TV padrão.

### 6. Resolução Escalada em 3 Níveis para Sleep/Timer (`sleep`)

**Decisão:**
Adicionar `device.display.daydream.DaydreamActivity` na cascata de `sleep`:
```java
case "sleep":
    intents = new Intent[] {
        tvSettings("partnercustomizer.timer.TimerActivity"),
        tvSettings("device.display.daydream.DaydreamActivity"),
        new Intent("android.settings.DREAM_SETTINGS"),
        new Intent(Settings.ACTION_SETTINGS)
    };
    break;
```
**Por quê:** Na TV TCL, o temporizador de desligamento automático (10 a 240 minutos) e as opções de tela apagada residem dentro de `DaydreamActivity` ("Protetor de tela"). Como a ação `DREAM_SETTINGS` não é resolvida pelo Android TV nesta TV, incluir a activity direta garante acesso imediato ao timer de sleep.

## Risks / Trade-offs

- **[Risco]** Firmware novo que use outro nome para stubs vazios.  
  → **Mitigação:** `leadsToScreen` já checa pacotes de stub do Google e agora cobre stubs com sufixo `EmptyStubActivity` em pacotes de configurações do sistema.
- **[Risco]** TVs onde Imagem/Timer não possuem telas públicas expostas para terceiros (como a TCL).  
  → **Mitigação:** Comportamento especificado: o Lume faz fallback suave para as Configurações Gerais (`Settings.ACTION_SETTINGS`).
