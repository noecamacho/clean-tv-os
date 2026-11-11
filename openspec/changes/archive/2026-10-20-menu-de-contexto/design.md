# Design

## Context

- `shell/keys.rs` mapeia os códigos do Android para `Key` (Up, Down, Left, Right, Ok, Back, Home); o 82 (`KEYCODE_MENU`) está na lista de teclas ignoradas do teste.
- O editor já tem um diálogo que prende o foco (`editor.rs`, `confirm_dialog`) e a lógica de favoritos (mover, adicionar, remover, limite de 12) em `HomeConfig`; as mudanças viram `Effect::SaveConfig`.
- Os efeitos de plataforma passam por `Effect` e chegam ao Java por `LumeBridge.handle(context, command, argument)`.
- O Java já sabe se um app é do sistema (`isSystemApp`, flags `FLAG_SYSTEM | FLAG_UPDATED_SYSTEM_APP`), mas só o usa para filtrar entradas LAUNCHER.
- A remoção de um pacote já chega como `PACKAGE_REMOVED` e o catálogo e os favoritos são atualizados no `on_resume`/`sync_catalog`.

## Goals / Non-Goals

**Goals:** um componente de menu reutilizável, operado só pelo D-pad, que altera os favoritos pelo mesmo caminho do editor.

**Non-Goals:** menu na barra superior, no tile "Lume" ou nas telas de texto; "Desinstalar atualizações" de apps do sistema; pressão longa de OK como atalho (fica para depois; o mesmo componente serviria).

## Decisions

### 1. O menu é um estado do app, não uma tela

`LumeApp` ganha `context_menu: Option<ContextMenu>` com a origem (app ou favorito), os itens e o item em foco. Enquanto ele existe, a entrada vai só para ele (foco preso) e a tela de baixo continua a mesma, desenhada normalmente. Alternativa descartada: um `Screen::Menu`, que obrigaria a guardar e restaurar a tela de origem e o foco, e duplicaria o desenho de fundo.

### 2. Ações reaproveitam `HomeConfig`

Adicionar, remover e mover chamam as mesmas operações do editor e emitem `Effect::SaveConfig`. Assim o limite de 12, a persistência atômica e a consistência com o editor saem de graça. Testes de `app/tests.rs` cobrem cada ação pelo menu.

### 3. Dois efeitos novos para o Android

- `Effect::OpenAppInfo { package }` → `Settings.ACTION_APPLICATION_DETAILS_SETTINGS` com `package:<nome>`.
- `Effect::Uninstall { package }` → `Intent.ACTION_DELETE` com `package:<nome>`; exige `REQUEST_DELETE_PACKAGES` desde a API 28. Sem essa permissão a intent falha em silêncio no Android 9+.

O resultado da desinstalação não é esperado: o Lume reage ao `PACKAGE_REMOVED` como já faz.

### 4. "É do sistema" vem do catálogo

A consulta de apps do Java passa a devolver a marca de sistema por entrada, guardada no catálogo (e no cache em disco, com a versão do cache incrementada, para que o cache antigo seja relido em vez de interpretado errado). O menu esconde "Desinstalar" com a marca ligada.

### 5. Posição do painel

- Dock: centrado no favorito, acima do dock (acima da Magnification do item), limitado à área segura.
- Grade: à direita do tile; na 6ª coluna, ou quando não couber, à esquerda; verticalmente alinhado ao topo do tile e limitado à área segura (a grade não rola por causa do menu).

Sem escurecimento: só o painel (área pequena) é translúcido durante o fade de 150 ms, respeitando a regra de nenhuma camada translúcida de tela cheia.

### 6. Mover como no launcher do Android TV

O menu do dock tem um "Mover" só, que abre um modo de mover (`app/dock_move.rs`): os vizinhos abrem 24 de cada lado para as setas, Esquerda/Direita trocam o favorito de lugar em memória, OK grava e Voltar restaura a ordem guardada no início. Alternativa descartada: "Mover para a esquerda/direita" no menu, que exigia reabrir o menu a cada posição.

As mudanças de ordem do dock deslizam (`DockShift`/`DockGrow` por posição, 240 ms): o quadro seguinte à mudança parte de onde cada ícone foi desenhado e anima o que falta até o novo lugar; a Magnification pula direto para a nova posição do foco, para nenhum ícone trocar de tamanho no caminho. O removido é desenhado à parte, encolhendo (`DockGhost`).

### 7. Menu claro

O cartão do menu é claro (`--menu-surface`), como o do launcher do Android TV: sobre as telas escuras, um painel `--surface` ficava camuflado.

### 8. Tecla Menu no desktop

`lume-desktop` aceita `menu` em `--keys`, para capturas como `--keys down,menu --screenshot apps m.png`.

## Risks / Trade-offs

- [O controle da TV pode não enviar `KEYCODE_MENU` (82); alguns mandam `KEYCODE_SETTINGS` 176 ou o sistema intercepta] → primeiro passo das tarefas é conferir na TCL e na AIWA com `adb shell getevent -l` ou log; se um deles mandar outro código, ele é mapeado também.
- [Cache de catálogo antigo sem a marca de sistema] → versão do cache incrementada, releitura completa uma vez.
- [Ação feita no menu enquanto o catálogo muda] → o menu fecha se o app de origem sumir do catálogo.
