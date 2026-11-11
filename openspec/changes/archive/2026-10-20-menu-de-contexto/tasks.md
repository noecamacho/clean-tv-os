# Tasks

## 1. Tecla Menu

- [x] 1.1 Conferir na TCL e na AIWA qual código o botão Menu do controle e do teclado envia (`adb shell getevent -l` ou log temporário) e registrar em `docs/device-notes.md`
- [x] 1.2 Adicionar `Key::Menu` e mapear `KEYCODE_MENU` (e o código extra achado em 1.1, se houver) em `shell/keys.rs`; atualizar os testes `the_remote_keys_are_mapped`/`other_keys_are_ignored`; aceitar `menu` em `--keys` do `lume-desktop`; verificar com `cargo test -p lume-core -q keys`

## 2. Catálogo: marca de app do sistema

- [x] 2.1 `LumeBridge` devolve a marca de sistema por entrada na consulta de apps; o catálogo a guarda e o cache sobe de versão; verificar com os testes do catálogo (`cargo test -p lume-core -q catalog`) incluindo um cache da versão anterior relido

## 3. Componente e estado do menu

- [x] 3.1 Classes do menu em `themes/lume/theme.css` (painel, item, item desabilitado com motivo) e textos em `strings.xml` en e pt-BR ("Adicionar ao Início", "Remover do Início", "Mover para a esquerda", "Mover para a direita", "Informações do app", "Desinstalar", "Limite de 12 favoritos"); atualizar `openspec/specs/design-system` só pelo delta desta change; verificar `cargo test --workspace -q` (tema e traduções compilam)
- [x] 3.2 Criar `app/context_menu.rs` com o estado (`Option<ContextMenu>` em `LumeApp`), foco preso, Menu/Voltar fecham, Home fecha antes de agir, item desabilitado pulado, fade de 150 ms só no painel; testes em `app/tests.rs` para abrir, fechar, foco preso e Menu sem efeito na barra superior e no tile "Lume"

## 4. Ações

- [x] 4.1 Grade de Apps: itens conforme a spec (adicionar/remover, limite, informações, desinstalar só fora do sistema), posição à direita/esquerda do tile; testes para cada cenário de `apps-screen`
- [x] 4.2 Dock: mover (sem as pontas), remover (foco no vizinho; "Adicionar favoritos" quando vazio), informações; posição acima do favorito; testes para cada cenário de `home-screen`
- [x] 4.4 Ajustes pedidos pelo autor no teste na TV: só um item do menu em foco por vez (a animação de foco dos itens é zerada ao abrir); "Mover" único no dock com modo de mover (setas, Esquerda/Direita, OK grava, Voltar cancela); ícones do dock deslizam ao mover, remover e desinstalar, e o removido encolhe no lugar; menu claro (`.menu`, `--menu-surface`) com subida de 8 ao aparecer; item em foco do menu sem anel de contorno (só o preenchimento); botão Menu da AIWA chega como 174 (`KEYCODE_BOOKMARK`) e foi mapeado
- [x] 4.3 `Effect::OpenAppInfo` e `Effect::Uninstall` no `lume-app` e comandos "app-info" e "uninstall" no `LumeBridge` (com `-keep` se chamados por JNI); permissão `REQUEST_DELETE_PACKAGES` no manifesto; `FORBIDDEN_PERMISSIONS` do `verify-apk.py` continua passando; verificar `cargo clippy --workspace --all-targets -q` limpo

## 5. Verificação integrada

- [x] 5.1 Capturas no desktop (`--keys down,menu --screenshot apps`, `--keys menu --screenshot home`) conferidas contra o design system
- [ ] 5.2 Na TCL e na AIWA: abrir o menu na grade e no dock, adicionar, mover, remover, abrir informações e desinstalar um app de teste; conferir que o foco volta certo e que o editor reflete as mudanças; medir quadros perdidos com o menu abrindo e fechando (meta: no máximo 1%). Parcial, arquivada pendente: menu e botão Menu conferidos nas duas TVs pelo autor; mover e remover medidos na TCL sem quadros de 33 ms ou mais; falta desinstalar um app de teste e conferir "Informações do app" na TV
- [x] 5.3 Revisar a tela Sobre: se entrou ícone Lucide, crate ou outro componente de terceiros novo, conferir que aparece com a licença; senão registrar "Sobre revisado, sem mudança"; conferir com `cargo run -p lume-desktop -- --screenshot about a.png`. Sobre revisado, sem mudança (nenhum crate, fonte ou ícone novo)
- [x] 5.4 `openspec validate menu-de-contexto --strict`
