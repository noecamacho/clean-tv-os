# lume: guia para quem retoma o projeto

Lume é uma tela inicial (launcher) rápida e leve para Android TV, escrita em Rust (interface, tema, renderização OpenGL ES 2.0)
com uma camada fina de Java (`LumeActivity` e `LumeBridge`, só onde o Android exige). Orçamentos e decisões ficam nas specs.

## Onde está a verdade

- `openspec/specs/`: o comportamento atual, por capacidade (home-screen, apps-screen, launcher-shell, wallpaper, theming, resource-efficiency...).
- `openspec/changes/archive/`: o histórico das changes (`2026-10-19-lume-launcher-v0-1-0`, `2026-10-20-dock-aurora-redesign`, `2026-10-20-v0-2-1`, `2026-10-20-escala-tipografica`, `2026-10-20-menu-de-contexto`, `2026-10-20-splash`, `2026-10-22-wallpapers-full-hd`.
- `docs/handoff-dock-aurora.md`: decisões do v0.2.0, armadilhas já resolvidas, como rodar e testar.
- `docs/device-notes.md`: medidas na TV de referência (memória, quadros, CPU, tempos).
- `docs/theme-format.md`, `docs/wallpaper.md`, `docs/translations.md`: formato do tema, papel de parede e traduções.

## Design system

A spec `openspec/specs/design-system/` é a regra visual: tokens, tipografia, medidas, componentes, foco e movimento.
Leia antes de criar ou alterar qualquer tela; valores em `themes/lume/theme.css`; divergiu, atualize a spec no mesmo change.

## Estado

A `main` está na versão da última tag (`git tag`); o histórico das changes mescladas fica em `openspec/changes/archive/`. Remoto: `origin` (GitHub `douglaspands/lume-tv-launcher`).
Pendências conhecidas, que dependem do autor: masters 4K em Git LFS (change `masters-no-lfs`; o histórico antigo guarda os 60 MB como blobs, decisão de não reescrevê-lo); assinatura do CLA (o job `cla` está em bypass com `continue-on-error`; falta a branch
`cla-signatures` e a assinatura do autor; ao ligar, remover o bypass de `.github/workflows/cla.yml`), keystore de release, CI e release no GitHub, tag `v0.1.0`/`v0.2.0` e decisão sobre o orçamento do retorno a quente (a spec diz 300 ms; o sistema da TCL
gasta cerca de 400 ms só na troca de janelas, o Lume 75 a 110 ms; a proposta é medir só a parte do Lume, até 150 ms). O emulador Android TV 9 (API 28 x86) está configurado (`tv_api28`, automação em `scripts/emulator.sh`).

## Versão do build

Sem arquivo para editar: a tag `vX.Y.Z` vira `X.Y.Z` (`LUME_VERSION_NAME`, tirando os "v" do início); build local usa o hash curto do
commit (`-dirty` se há mudança rastreada não commitada; `dev` sem git) e `versionCode` = contagem de commits. A regra vive em
`crates/lume-version/build.rs`, `android/app/build.gradle` e `scripts/container-build.sh`. O `version` do `Cargo.toml` (1.0.0) não é a versão do build.

## Como trabalhar aqui

- **Uma branch por change OpenSpec**: logo após `openspec new change "<nome>"`, `git checkout -b feat/<nome>`.
- **Git LFS nos masters**: `tools/wallpaper/masters/*.png` são objetos LFS (`.gitattributes`); rode `git lfs install` uma vez. Clone sem `git-lfs` traz ponteiros de texto nesses PNGs: o build, os testes e a CI não os leem (a CI não usa `lfs: true`, para poupar banda), mas `generate_wallpapers.py` precisa dos PNGs reais (`git lfs pull`).
- **Não commitar sem o autor pedir.** Quando pedir, commits por tema; o fechamento é `/opsx:archive` e merge **squash** na `main`.
- **Tela Sobre ao fim de cada change**: a última tarefa de toda change revisa a tela Sobre (Configurações do Lume › Sobre o Lume)
  e a atualiza se a change trouxe algo novo: componente de terceiros (crate, fonte, ícones), licença, endereço do repositório
  (`https://github.com/douglaspands/lume-tv-launcher`, uma fonte só) ou versão. Sem novidade, registre "Sobre revisado, sem mudança".
- Fluxo OpenSpec: `/opsx:propose`, `/opsx:apply`, `/opsx:archive`. Validar com `openspec validate <change> --strict`.
- Idioma: respostas, specs e commits do projeto em português do Brasil; código, comentários e mensagens de commit em inglês.
- Sempre que precisar usar o Python, use o `uv` (`uv run ...`), sendo permitido instalar dependências quando necessário.
- **Shell de execução**: Usar sempre o `zsh` como prioridade absoluta caso esteja instalado (via `zsh -c "..."`) ao invés do `bash` para qualquer execução de comandos no workspace.
- **Economize contexto e tokens**: prefira comandos que tragam só o necessário. Use `grep -n`/`rg` com padrão
  específico, `sed -n 'A,Bp'` ou leitura por faixa de linhas em vez de arquivos inteiros, `head`/`tail`/`wc -l`
  para saídas longas, `--json` com filtro (`python3 -c`, `jq`) nos comandos do `openspec`, `--quiet`/`-q` em
  `cargo`, `gradle` e `git` quando o detalhe não importa, e `2>&1 | tail -n 30` em builds. Não releia arquivos já
  lidos nem repita buscas cujo resultado já está na conversa.

## Comandos

```
cargo test --workspace && cargo clippy --workspace --all-targets   # deve ficar limpo; cargo fmt --all
cargo run -p lume-desktop -- --size 1920x1080 --locale en --keys right --screenshot home h.png   # telas: home apps settings tvinfo about editor
uv run scripts/measure-ink.py h.png columns --y 28 60   # vãos e alturas na tinta (captura em 1280x720); régua no design system
scripts/container-build.sh release       # APK em dist/ (runtime: wslc, docker, podman, nesta ordem; LUME_CONTAINER força outro)
uv run scripts/verify-apk.py dist/lume-<versão>-release-debugsigned.apk   # limite 40 MB
scripts/emulator.sh start                # Inicia o emulador Android TV 9 (API 28, 1080p, KVM)
scripts/emulator.sh install debug        # Instala APK debug (com x86) e abre o Lume
```

## Emulador Android TV (testes locais)

Emulador local Android TV 9 (API 28, x86) acelerado por KVM, com AVD `tv_api28` (1080p).

- Iniciar e aguardar boot: `scripts/emulator.sh start` (ou `scripts/emulator.sh start --no-window` para modo headless).
- Instalar e testar: `scripts/emulator.sh install debug` (o build debug embute `x86`, `x86_64`, `armeabi-v7a` e `arm64-v8a`).
- Controle remoto por comando: `scripts/emulator.sh key up|down|left|right|ok|back|home|menu`.
- Alternar resolução: `scripts/emulator.sh resolution 720p` (para simular a AIWA) ou `scripts/emulator.sh resolution reset`.
- Captura de tela: `scripts/emulator.sh screenshot x.png`.
- Encerrar: `scripts/emulator.sh stop`.

## TV de referência (testes por adb)

TCL 55C715 (2 GB de RAM, 4 núcleos); o autor autorizou testar nela. A TCL roda o firmware global (Android 11); o firmware original do Brasil é Android 9.
Segunda TV de teste: AIWA SMART TV (KTC, 1280x720, ~913 MB de RAM, `low_ram`). Os IPs mudam e não ficam no repositório: pergunte ao autor (`adb -s <ip>:5555`). Comparação em `docs/device-notes.md`.

- Instalar: `adb uninstall app.lume.tv` antes (a chave de debug muda a cada build no container), depois `adb install dist/<apk>` e
  `adb shell cmd package set-home-activity app.lume.tv/.LumeActivity`.
- Capturas: `adb exec-out screencap -p > x.png`. Memória: `dumpsys meminfo app.lume.tv`.
- Quadros perdidos: `dumpsys SurfaceFlinger --latency "<camada>"` (o nome da camada, `app.lume.tv/app.lume.tv.LumeActivity#N`, muda
  depois de reinstalar: procure com `--list`); conte intervalos de 33 ms ou mais.
- Abrir outro app para testar segundo plano: `adb shell monkey -p <pacote> -c android.intent.category.LEANBACK_LAUNCHER 1`.

## Decisões fixas do autor

- **Contraste da barra superior**: os papéis de parede não levam sombra no topo nem na base (o autor achou feia). O texto da barra pode ficar abaixo de 4,5:1 sobre os papéis mais claros e isso é decisão do autor, não pendência: dois testes de contraste estão com `#[ignore]` por isso. Só mude se o autor pedir.
- **Construção 100% Rust.** Java só quando a funcionalidade não tem como ser feita sem consumir ou usar um recurso
  do Java (API do Android sem equivalente no NDK). Antes de escrever ou mudar qualquer Java, pergunte ao autor no chat,
  explicando por que o Rust não resolve; sem o sim dele, não entra Java.
- Papel de parede: 6 papéis em 3840x2160 (masters em `tools/wallpaper/masters/`), baixados, não gerados pelos scripts; licença Unsplash License segundo o autor, com autores e URLs desconhecidos (ver `docs/wallpaper.md`); a Aurora é o padrão.
- Tela Início: só o dock (até 12 favoritos, Magnification), com favoritos redondos por padrão. Sem carrossel nem seções.
- Abas: mover o foco para uma aba troca a tela sem OK, com fade de 180 ms. Sublinhado azul com o foco na barra, branco com o foco no conteúdo.
- Cores por papel (`--primary` azul `#4DB3FF`, `--secondary`); todo contorno de foco é azul primário; o nome do app em foco fica branco.
- Grade de Apps: 6 colunas, ordem alfabética pelo nome mostrado (muda com o idioma do sistema). O tile "Lume" é um app como os outros
  (retangular, logo) e abre as Configurações do Lume (só por ele). Na barra superior, o botão de entradas abre o menu de entradas da TV (escondido se a ação não resolve) e a engrenagem abre as configurações do Android. A marca se escreve "Lume" na interface.
- Home com o Lume visível leva à aba Início; voltando de um app aberto pelo Lume, restaura tela, foco e rolagem (uma vez).

## Armadilhas já resolvidas

- **Nunca desenhar camada translúcida de tela cheia**: na GPU da TV ela custou 54% dos quadros da grade. O escurecimento das telas de
  texto é feito no próprio desenho do papel de parede (`Painter::wallpaper(true)`, sobre o fundo `--bg` em que o quadro é limpo).
- Animação com valor de repouso 0 é descartada pela limpeza (`Animations::prune`); passe o valor inicial em `animate_to`.
- Métodos Java chamados só pelo Rust (JNI) precisam de `-keep` em `android/app/proguard-rules.pro`.
- Carregar ícones durante uma animação trava o dock: o laço só carrega quando `Shell::is_busy` é falso.
- `entry_down`/`entry_up` de `Row` são do destino do foco, não da origem.
- O Android da TCL encerra o processo do Lume em segundo plano quando um app pesado abre; o retorno é então uma abertura a frio.
- Botão Menu: a TCL envia `KEYCODE_MENU` (82), a AIWA `KEYCODE_BOOKMARK` (174); os dois abrem o menu de contexto. Para achar o código
  de um botão novo, o Lume registra os que ignora: `adb logcat | grep "not used"` (na AIWA, `getevent` não vê o controle IR).
- Na AIWA, `screencap -p` escreve texto antes do PNG (corte até `\x89PNG`). Confira o `Success` do `adb install` em cada TV: um
  `tail -1` esconde a falha de assinatura quando a desinstalação foi esquecida.

## Orçamentos (spec `resource-efficiency`)

PSS até 64 MB em primeiro plano e 20 MB em segundo; APK até 40 MB (subiram com os papéis de parede em Full HD; hoje 21,6 MB, com uma cópia de cada papel em `assets/`); no máximo 1% de quadros de 33 ms ou mais na navegação; retorno a quente
até 300 ms; o laço não pede quadros parado além do relógio. Última medida (2026-10-21, `docs/device-notes.md`): TCL com 14,7 MB de PSS em primeiro plano e 16,9 MB em segundo, APK de 21,6 MB, 0 de 454 quadros perdidos no dock e 1,5% na grade (7 de 459), primeiro quadro a frio em 89 a 98 ms.
