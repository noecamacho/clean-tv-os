# Design

## Context

Hoje a barra superior tem três alvos de foco (`TAB_HOME`, `TAB_APPS`, `GEAR`) numa `Row` em `topbar.rs`. A engrenagem chama `open_settings()`, que abre a tela de configurações do Lume e guarda a aba de origem em `settings_return`. O efeito `Effect::OpenAndroidSettings` já existe e hoje só é disparado pelo item "Configurações do Android" e pelo tile "Lume" (`GRID_SETTINGS`) leva à tela do Lume. A ponte Java (`LumeBridge.handle`) expõe `settings` (abre `Settings.ACTION_SETTINGS`) e `start`. O Android TV precisa continuar suportado do 9 ao 11; a TCL de teste roda o firmware global (Android 11), a AIWA também, e nenhuma tem Android 9.

Medido nas duas TVs: `com.android.tv.action.VIEW_INPUTS` resolve para atividades diferentes (`com.tcl.tv/...EnableViewInputLauncher` na TCL, `com.mediatek.wwtv.tvcenter/...InputsPanelActivity` na AIWA) e abre sem permissão especial. O `screencap` não captura o painel.

## Goals / Non-Goals

**Goals:**
- Botão de entradas e engrenagem como descritos nas specs, sem depender de pacote de fabricante.
- Nada novo no custo de quadros: o botão é só mais um item estático da barra.

**Non-Goals:**
- Listar ou escolher entradas dentro do Lume; só abrimos o menu do sistema.
- Atalho para as configurações do Lume na barra superior.
- Renomear pacote, crate ou biblioteca nativa.

## Decisions

1. **Ação padrão em vez de componentes por fabricante.** A ponte monta `Intent(action)` com `FLAG_ACTIVITY_NEW_TASK`. Alternativa descartada: nomes de componente por marca, que quebram em TVs não testadas.
2. **Disponibilidade consultada uma vez, na inicialização.** Novo comando `inputs` na ponte responde "1" ou "0" com `resolveActivity`; o Rust guarda o booleano no estado do app e monta a `Row` com ou sem o botão. Alternativa: mostrar sempre e ignorar a falha. Descartada: botão morto.
3. **`<queries>` no manifesto** com a ação `com.android.tv.action.VIEW_INPUTS`. Sem isso, no Android 11 (API 30) a consulta de visibilidade de pacotes devolve "não existe" mesmo com o painel instalado e o botão nunca apareceria. No Android 9 o elemento é ignorado, então uma só declaração cobre 9 a 11. O `startActivity` não depende da visibilidade, mas a consulta de `resolveActivity` sim.
4. **Novo `Effect::OpenInputs`**, tratado em `lume-app/src/android/mod.rs` como `OpenAndroidSettings`. Falha de `startActivity` (`ActivityNotFoundException`, `SecurityException`) só registra um aviso no log.
5. **Engrenagem dispara `Effect::OpenAndroidSettings`**; `open_settings()` passa a ser chamada só pelo tile `GRID_SETTINGS`. Como a tela só vem da aba Apps, `settings_return` é removido e voltar leva o foco ao `GRID_SETTINGS`. O item `ITEM_ANDROID` e a string `settings_android` saem (en e pt-BR).
6. **Ícone**: glifo do Lucide mais próximo do ícone de entradas do Google TV (retângulo com seta entrando, candidato `log-in`/`square-arrow-right-enter`), adicionado em `ICONS` de `subset-fonts.py` e regerado `icons.rs`. O glifo final é escolhido olhando uma captura do desktop (`lume-desktop --screenshot`) ao lado do ícone do Google TV.
7. **Posição**: o botão de entradas fica a 56 px à esquerda da engrenagem (mesmo tamanho 48 px, espaço de 8 px); sem o botão, a engrenagem continua em x=1096. O foco horizontal percorre abas, entradas e engrenagem.
8. **Grafia "Lume"**: mudança só em texto de usuário (strings, docs, specs prosa tocados por esta change). Nomes técnicos ficam. A verificação é um `grep` de `\blume\b` em strings e docs.
9. **Painel de estatísticas só com a tela visível.** O comando `stats` da ponte devolve memória (`/proc/meminfo`) e disco (`StatFs` em `/data`) numa só chamada, sem permissão. A CPU vem de `/proc/stat` (bloqueado para apps desde o Android 8); se falhar, tenta-se a frequência dos núcleos em `/sys/devices/system/cpu` e, não havendo, a linha mostra "—". O que cada TV permite fica por confirmar em teste (a tarefa 4.1). Um temporizador de 2 s existe só enquanto `Screen::Settings` está ativa, e respeita `Shell::is_busy` (sem leitura durante animação). Ao sair da tela ou ir a segundo plano, o temporizador é cancelado e o laço volta a não pedir quadros. O painel é um cartão à direita, com barras finas na cor primária, sem animação e sem foco. Alternativas descartadas: interruptor liga e desliga (o usuário preferiu o painel sempre visível) e lista de apps em execução (um app comum não lê memória nem CPU de outros apps; `UsageStatsManager` daria só tempo de uso e exigiria conceder acesso nas configurações).
10. **Versão 0.2.1** no `Cargo.toml` do workspace e no nome do APK; `verify-apk.py` e os docs seguem o novo número.

## Risks / Trade-offs

- **Android 9 sem dispositivo.** Nenhuma TV de teste roda Android 9. Mitigação: validar num emulador Android TV 9 (API 28) a instalação, a engrenagem e a ausência do botão quando a ação não resolve; registrar o resultado em `docs/device-notes.md`. Se não houver emulador disponível, a task fica aberta e o risco é declarado no archive.
- **`resolveActivity` e visibilidade.** Se o `<queries>` falhar em alguma TV com política própria, o botão some em vez de quebrar. É a falha mais segura.
- **Painel não capturável.** A validação nas TVs é visual, pelo autor, mais o log do `ActivityTaskManager` (`Displayed ...InputsPanelActivity`).
- **CPU pode ser só um traço.** Se a TV bloquear `/proc/stat` e a frequência, o painel mostra memória e disco. É aceito pela spec.
- **Custo de quadros com o painel.** O painel pede um quadro a cada 2 s; a AIWA (`low_ram`, 720p) é a TV para medir PSS e quadros perdidos.
- **Quebra de hábito.** Quem usava a engrenagem para as configurações do Lume passa a usar o tile. É intencional e fica nas notas de versão.
