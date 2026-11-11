# Proposal

## Why

O botão Menu do controle e do teclado não faz nada no Lume, e gerenciar os favoritos exige abrir o tile "Lume", depois "Tela Início", e achar o app no editor. O uso padrão desse botão no Android é o menu de contexto do item em foco: com ele, adicionar, mover e remover favoritos, ver as informações de um app ou desinstalá-lo passa a ser feito onde o app está. Segunda das três changes do release v0.3.0.

## What Changes

- O botão Menu passa a abrir um menu de contexto para o item em foco, num cartão claro junto do item (como o do launcher do Android TV), sem escurecer a tela.
- **Grade de Apps**, sobre um app: "Adicionar ao Início" (ou "Remover do Início", se já for favorito), "Informações do app" (tela do app nas configurações do Android) e "Desinstalar" (só para apps que não são do sistema). Com 12 favoritos, "Adicionar ao Início" aparece desabilitado com "Limite de 12 favoritos".
- **Dock**, sobre um favorito: "Mover" (modo de mover como no launcher do Android TV: setas ao lado do favorito, Esquerda/Direita o levam, OK grava, Voltar cancela), "Remover do Início" e "Informações do app". Mover e remover são animados (os ícones deslizam; o removido encolhe no lugar).
- Fora desses itens (tile "Lume", "Adicionar favoritos", barra superior, Configurações, Sobre, editor), Menu não faz nada.
- Voltar ou Menu fecham o menu sem fazer nada; OK executa a ação e fecha. O foco fica preso no menu enquanto ele está aberto.
- As mudanças nos favoritos são gravadas como as do editor e aparecem no editor.
- Um componente novo no design system: o menu de contexto.
- A desinstalação pede a permissão normal `REQUEST_DELETE_PACKAGES` (concedida na instalação, sem diálogo), exigida pelo Android 9+ para abrir o desinstalador do sistema.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `launcher-shell`: o botão Menu entra nas teclas reconhecidas e abre o menu de contexto onde houver um.
- `apps-screen`: menu de contexto dos apps da grade.
- `home-screen`: menu de contexto dos favoritos do dock.
- `design-system`: componente "menu de contexto" (superfície, itens, foco, item desabilitado, posição).
- `platform-compatibility`: as permissões mínimas passam a admitir `REQUEST_DELETE_PACKAGES`.

## Impact

- Rust: `shell/keys.rs` (KEYCODE_MENU 82 → `Key::Menu`), `app/mod.rs` (estado do menu, `Effect::OpenAppInfo`, `Effect::Uninstall`), novo `app/context_menu.rs`, `app/apps.rs` e `app/dock.rs` (ações e posição), `catalog` (marca de app do sistema), testes em `app/tests.rs` e `shell/keys.rs`.
- Tema: `themes/lume/theme.css` (classes do menu). Texto: `strings.xml` (en e pt-BR).
- Java: `LumeBridge` (comandos "app-info" e "uninstall", e a marca de sistema na consulta de apps), `AndroidManifest.xml` (permissão), `proguard-rules.pro` se algo novo for chamado por JNI.
- `lume-desktop`: uma tecla para Menu (para capturas e testes).
- Sem custo em segundo plano; o menu só existe enquanto aberto.
