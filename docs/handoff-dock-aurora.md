# Continuidade: change `dock-aurora-redesign` (lume v0.2.0)

Este arquivo existe para quem retomar o trabalho sem o histórico da conversa. A fonte da verdade são os
artefatos da change em `openspec/changes/dock-aurora-redesign/` (`proposal.md`, `design.md`, `specs/`,
`tasks.md`); aqui ficam o estado, as decisões que não estão óbvias no código e como rodar e testar.

Branch: `feat/dock-aurora-redesign`, **tudo commitado em 8 commits por tema** (build, openspec, remoção do carrossel,
wallpaper, tema, Android, app, versão). Arquivos compartilhados (`app/mod.rs`, `app/tests.rs`, `shell/mod.rs`...) foram
para o commit do tema dominante, então só o último commit compila e passa nos testes por inteiro. Antes de arquivar,
rodar `openspec validate dock-aurora-redesign --strict` e `cargo test --workspace`.

## O que a change faz (resumo do resultado final)

- **Carrossel e seções removidos.** `HomeConfig` só tem `favorites` (`config.v1` antigo continua legível, o
  resto é ignorado). O editor abre direto na edição dos favoritos.
- **Dock** na tela Início (`crates/lume-core/src/app/dock.rs`, `app/home.rs`): barra translúcida flutuante, até 12
  ícones quadrados, **Magnification** (pico 1,6, tabela 1,0/0,45/0,15/0, vizinhos empurrados, base ancorada),
  balão com o nome que desliza junto com o foco, cor + inicial enquanto o ícone não chega. Constantes de
  tamanho/espaço ficam no código; o tema só tem `.dock` e `.dock-label` (fundo, raio, anel, sombra, balão).
  Animação do foco do dock: 210 ms ease-out (`MOVE`).
- **Ícones do dock**: faixa de 8x2 células de 128x128 no atlas (1024x2304); `LumeBridge.dockIcon` (Java) devolve o ícone
  inteiro (adaptativo desenhado por camadas) num quadrado arredondado; cache em disco por tipo. Carregados só
  para os favoritos, um por volta do laço e **só quando nenhuma animação está rodando**.
- **Papel de parede = imagem "06" (aurora gerada)**: base 640x360 RGB + 420 estrelas em vetor (1 ou 2 pixels do
  buffer). Gerada por `tools/wallpaper/aurora06.py` (Python puro; reproduz os arquivos byte a byte) em
  `crates/lume-core/assets/wallpaper/`. `Painter::wallpaper()` desenha tudo no início de cada quadro; telas de
  texto (Apps, Configurações, Sobre, edição) ganham a camada `TEXT_SCREEN_DIM` = `rgba(15,23,30,200/255)`
  (0,78: a aurora exige ≥ ~0,72 para 4.5:1; o 0,65 do plano não bastava). Início não tem a camada. Uma foto de
  terceiros foi experimentada e **descartada** (sem estrela em vetor, licença incerta); o código dela foi removido.
  Ver `docs/wallpaper.md`.
- **Grade de Apps**: 6 colunas, tiles ~172x97. O tile de Configurações do Android mostra a **engrenagem** (glifo
  `icons::SETTINGS`), não o nome, como o launcher do Google.
- **Barra superior**: abas sem contorno. **Mover o foco para uma aba (Esquerda/Direita) troca a tela na hora, sem
  OK.** O sublinhado fica sob a aba ativa: **azul com o foco na barra (abas/engrenagem), branco com o foco no
  conteúdo (dock/grade)**, com a transição do foco. Engrenagem em foco: contorno azul `--accent`.
- **Home**: com o lume **visível**, Home leva ao ponto de partida (aba Início, foco no 1º item do dock). Se o lume
  tinha aberto um app (dock, grade, ou itens das Configurações do Android) e o Home o **traz de volta**, ele
  restaura tela + foco + rolagem de antes (ex.: Apps com o foco no YouTube). Regra: `return_point` guardado ao
  abrir, vale uma vez, descartado por qualquer outra tecla ou se o app não abre; só conta se o lume saiu da tela
  (`went_to_background`/`came_to_foreground`, `RETURN_WINDOW_MS` = 2 s). Só em memória (processo morto = Início).
- **Cores por papel**: `--primary` (azul `#4DB3FF`), `--on-primary`, `--secondary` (`#F2F5F7`, neutro/texto); `--accent`/`--on-accent` são
  apelidos. **Todo contorno de foco** (tiles, itens de Configurações, linhas e chips do editor, engrenagem, anel dos cartões)
  é azul primário (antes branco). Documentado em `docs/theme-format.md` (seção "Color roles"). No código use `vars::PRIMARY`
  e `vars::SECONDARY`.
- **Versão 0.2.0** (`Cargo.toml` e `android/app/build.gradle`).
- **Container build**: `scripts/container-build.sh` escolhe o runtime por prioridade **wslc, docker, podman** (nesta
  ordem; `LUME_CONTAINER` força outro) e, sem nenhum, imprime alternativas. Caminho com espaço corrigido.

## Estado das tarefas

Todas as tarefas da change estão concluídas (incluindo a seção 10, ajustes finais). Falta só `/opsx:archive`, depois de revisar o diff e commitar
(há mudanças ainda não commitadas: tile do lume, fade das abas, escurecimento no papel de parede, specs principais, capturas do README).
A change antiga `lume-launcher-v0-1-0` foi encerrada com as tarefas abertas marcadas como canceladas.

## Armadilhas já resolvidas (não cair de novo)

- **Animação em 0 era descartada**: `Animations::prune(.., |_| 0.0)` apaga valores em repouso em 0; a posição do dock
  no ícone 0 sumia e o 1º movimento recomeçava no destino (pulo). Hoje `track_dock` passa a posição anterior como
  valor inicial. Qualquer animação nova cujo valor de repouso seja 0 tem o mesmo risco.
- **R8**: métodos Java chamados só pelo Rust (JNI) precisam de `-keep` em `android/app/proguard-rules.pro`
  (`dockIcon` foi removido na release até ganhar a regra).
- **Chave de debug muda a cada build no container**: `adb install -r` falha com
  `INSTALL_FAILED_UPDATE_INCOMPATIBLE`; é preciso `adb uninstall app.lume.tv` antes (perde a config do lume e o HOME
  padrão; refazer com `adb shell cmd package set-home-activity app.lume.tv/.LumeActivity`).
- **Python**: neste ambiente `python3` (asdf) pode falhar por falta de versão; usar `/usr/bin/python3`.
- `entry_down`/`entry_up` do `Row` são do **destino** (a linha em que o foco entra), não da origem.
- Carregar ícones (15–46 ms por volta no 1º uso) trava a animação se rodar durante ela: o laço só carrega
  quando `Shell::is_busy` é falso.

## Como rodar e testar

```
cargo test --workspace && cargo clippy --workspace --all-targets   # deve ficar limpo
cargo run -p lume-desktop -- --size 1920x1080 --favorites 12 --keys right,right --screenshot home h.png
scripts/container-build.sh release            # dist/lume-<versão>-release-debugsigned.apk (assinado com chave de debug)
python3 scripts/verify-apk.py dist/lume-<versão>-release-debugsigned.apk   # limite 40 MB
```

TV de referência (TCL 55C715, 2 GB RAM, 4 núcleos): `adb -s <ip-da-tv>:5555` (o IP muda e não fica no repositório; o usuário autorizou o acesso por adb).
Instalar: `adb uninstall app.lume.tv; adb install dist/lume-<versão>-release-debugsigned.apk; adb shell cmd package
set-home-activity app.lume.tv/.LumeActivity`. Capturas: `adb exec-out screencap -p > x.png`.

Testes que guardam os requisitos: `app/tests.rs` (navegação, dock, abas, retorno, sublinhado, contraste),
`app/dock.rs` (Magnification e balão), `tools/lume-desktop/tests/{render,contrast,wallpaper}.rs` (GL real: fidelidade
≤ 2/255 contra `mockup/wallpaper-aurora-lume@1080p.png`, estrelas nítidas, contraste 4.5:1 no quadro renderizado).

## Medidas na TV (v0.2.0, release)

Ver a seção "Análise de desempenho da v0.2.0" em `docs/device-notes.md` (resumo: 17 MB de PSS em primeiro plano e 16 MB em
segundo, abertura a frio 371 ms, retorno a quente 127 ms, 0,2% de um núcleo ocioso e 11% navegando).

## Versão do build

A versão vem do build (tag normalizada no release; hash curto, `-dirty` ou `dev` no build local), definida igual em `crates/lume-version/build.rs`, `android/app/build.gradle` e `scripts/container-build.sh`. Exemplo: `LUME_VERSION_NAME=v0.3.0 scripts/container-build.sh release` gera `dist/lume-0.3.0-release-debugsigned.apk`.
