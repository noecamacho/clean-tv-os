# Proposal

## Why

Na TV a engrenagem leva às configurações do próprio Lume, mas quem usa a TV espera nela as configurações do Android, e falta um atalho para as entradas (HDMI e outras), que o launcher do Google oferece ao lado da engrenagem. Além disso, a marca aparece em minúsculo ("lume") no rótulo do app e na tela Sobre, enquanto a interface já escreve "Lume".

## What Changes

- Um botão de entradas na barra superior, à esquerda da engrenagem, abre o menu de entradas da TV (`com.android.tv.action.VIEW_INPUTS`). Ele fica escondido quando o sistema não resolve a ação.
- A engrenagem passa a abrir direto as configurações do Android. **BREAKING**: ela não leva mais às configurações do Lume.
- As configurações do Lume ficam só no tile "Lume" da grade de Apps. Voltar delas leva o foco de volta a esse tile, e a tela deixa de lembrar de qual aba veio.
- O item "Configurações do Android" sai das configurações do Lume, que ficam com "Tela Início" e "Sobre o Lume".
- "Lume" com L maiúsculo em tudo que o usuário lê: rótulo do app (`app_name`), tela Sobre, textos das configurações, docs e specs. Identificadores técnicos (pacote `app.lume.tv`, crate, biblioteca nativa, tag de log) não mudam.
- A tela de Configurações do Lume ganha, à direita dos itens, um painel fixo com estatísticas da TV (memória, disco e CPU), sem interruptor. O Lume só lê o sistema enquanto essa tela está visível. A lista de apps em execução com memória e CPU fica de fora, porque um app comum não consegue lê-la sem permissão de sistema.
- A compatibilidade Android TV 9 a 11 continua obrigatória: o botão e a engrenagem funcionam nas duas pontas. No Android 11 a ação precisa ser declarada em `<queries>` no manifesto.
- Versão 0.2.1.

## Capabilities

### New Capabilities

- `device-stats`: painel de memória, disco e CPU na tela de Configurações do Lume, lido só com a tela visível.

- `design-system`: o design system do frontend (tokens, tipografia, medidas, componentes, foco e regras de desenho) como regra que toda mudança visual segue.

### Modified Capabilities

- `launcher-shell`: barra superior (botão de entradas, engrenagem para o Android) e tela de Configurações do Lume (sem o item do Android, acessada só pelo tile, com o painel de estatísticas à direita).
- `platform-compatibility`: o teste do Android 9 deixa de depender da TCL (hoje em Android 11) e o botão de entradas passa a ter requisito nas duas versões.

## Impact

- Rust: `crates/lume-core/src/app/{topbar,settings,mod}.rs` (inclui o painel e o ciclo de leitura de 2 s) (nova entrada de foco, `Effect::OpenInputs`, remoção de `settings_return` e de `ITEM_ANDROID`), `icons.rs` e `scripts/subset-fonts.py` (novo ícone Lucide), `crates/lume-app/src/android/` (efeito e ponte), testes em `app/tests.rs`.
- Java: `LumeBridge` (comandos para abrir as entradas, consultar se existem e ler `stats`: memória e disco), `AndroidManifest.xml` (`<queries>`), `proguard-rules.pro` se algo novo for chamado por JNI.
- Recursos: `strings.xml` (en e pt-BR), com "Lume" em maiúsculo e sem `settings_android`.
- Versão em `Cargo.toml`, `CLAUDE.md` e nomes de APK; docs (`device-notes`, `README`, `translations`) e specs com a grafia nova.
- Testes em TV: TCL (Android 11) e AIWA (Android 11) por adb; Android 9 só em emulador, pois nenhuma TV de teste roda Android 9.
