# Notas de dispositivo e ambiente

Registro de medições e descobertas feitas durante o desenvolvimento do Lume v0.1.0.

## Build em container: validação do `wslc` (tarefa 1.6)

Ambiente: `wslc` 3.0.3.0 (`C:\Program Files\WSL\wslc.exe`), chamado do WSL2 (Ubuntu).

| Recurso | Resultado |
|---|---|
| Dockerfile em estágios (`FROM ... AS`, `--target`) | Funciona |
| `RUN --mount=type=cache` | Funciona, e o cache persiste entre builds (contador foi de 1 para 2 quando a camada foi invalidada) |
| `--secret id=...,src=<arquivo>` | Funciona (o segredo ficou visível em `/run/secrets/<id>` só durante o `RUN`) |
| Cache de camadas | Funciona (segunda execução idêntica: 0,8 s, todas as camadas `CACHED`) |
| Contexto convertido com `wslpath -w` | Funciona (caminhos `\\wsl.localhost\...`) |
| `-o type=local,dest=<pasta>` | **Não suportado**: `directory exporters are not supported` |
| `-o type=tar,dest=<arquivo>` | Funciona; o arquivo aparece na pasta do WSL |

**Decisão:** como o `wslc` só exporta um arquivo, o `scripts/container-build.sh` usa `-o type=tar,dest=dist/artifacts.tar` e extrai com `tar -xf`. O mesmo formato funciona com `docker buildx`, então o script usa um único caminho para os dois runtimes. O estágio `artifacts` (`FROM scratch`) não muda.

## TCL 55C715: dispositivo de referência (tarefas 2.1 a 2.3)

Medido em 2026-10-19 com o APK de debug (`dist/lume-0.1.0-debug.apk`), via `adb` por rede.

| Item | Valor |
|---|---|
| Modelo / fabricante | `BeyondTV2` / TCL (SoC Realtek `rtd285o`) |
| Android | 11 (API 30), build `release-keys`, `ro.config.low_ram=false`; firmware global instalado pelo autor (o firmware original do Brasil só chega ao Android 9) |
| ABIs | `armeabi-v7a`, `armeabi` (somente 32 bits) |
| GPU | ARM Mali-470 MP, **OpenGL ES 2.0** (`ro.opengles.version=131072`), driver r9p0 |
| Tela | `wm size` 1920×1080, `wm density` 320 |
| RAM | `MemTotal` 2 GB (`MemAvailable` ~498 MB com a TV ociosa) |
| Fontes do sistema | 236 arquivos em `/system/fonts` (Noto por escrita, `NotoSansCJK-Regular.ttc`, `DroidSans*`), mais `/system/etc/fonts.xml` |

### 2.1 Hello GL / renderer

- O primeiro build falhou na TV com `program failed to link: L0010 Uniform 'u_view' differ on precision`: o driver Mali exige a mesma precisão para um `uniform` declarado nos dois estágios, e o fragment shader do retângulo usava `highp` condicional enquanto o vertex shader usava o padrão. **Correção:** o `u_view` saiu do fragment shader, e a unidade de pixel (`1 / escala`) passa do vertex shader para o fragment shader pelo varying `v_local.z`.
- Depois disso, o Lume desenha a interface inteira na TV (barra superior, carrossel, favoritos, textos em pt-BR). Screenshot por `adb exec-out screencap -p`.
- O build de debug do container usa uma chave de debug diferente a cada imagem, então `adb install -r` falha com `INSTALL_FAILED_UPDATE_INCOMPATIBLE`. Em desenvolvimento, desinstale antes (`adb uninstall app.lume.tv`).

### 2.2 Botão Home

| Tentativa | Resultado |
|---|---|
| `cmd package set-home-activity app.lume.tv/.LumeActivity` | Aceito, **mas o Home continua abrindo o `com.google.android.tvlauncher`** (a resolução de HOME ignora a escolha) |
| `pm disable-user --user 0 com.google.android.tvlauncher` | **Funciona**: HOME resolve para `app.lume.tv/.LumeActivity`, e `input keyevent KEYCODE_HOME` volta ao Lume a partir de Configurações e do YouTube |

Reverter: `adb shell pm enable com.google.android.tvlauncher`.
Limitação: o teste usou `input keyevent KEYCODE_HOME`. A conferência com o botão físico do controle da TCL continua pendente (tarefa 14.1).

### 2.3 Memória e comportamento em segundo plano (build de debug)

| Medida | Resultado |
|---|---|
| Lume em primeiro plano, ocioso (TOTAL PSS) | ~20,4 MB (Native 5,2 MB, Java 3,3 MB, Code 2,3 MB, System ~8,5 MB, Graphics 0) |
| Lume dormindo atrás de outro app, ~12 s (TOTAL PSS) | **18,6 MB**, Graphics 0 (`window gone: renderer, glyph atlas and fonts released (dormant)`) |
| Launcher de fábrica (`com.google.android.tvlauncher`), ocioso | **48,7 MB** (Native 21 MB, Code 10 MB) |

Descobertas:
- O Android da TCL **encerra o processo do Lume pouco depois de ele ir para segundo plano** (`ActivityManager: Killing ... app.lume.tv (adj 600): stop app.lume.tv`; há também uma lista `persist.tcl.standby.processName`). Nos testes isso aconteceu quando outro app pesado abriu (Play Store). Nesses casos o custo em segundo plano é zero, e todo retorno é uma abertura a frio.
- Configurações da TV (`com.android.tv.settings/.MainSettings`) é translúcida: o Lume só pausa, não para, e o ciclo DORMINDO não dispara. Para testar o DORMINDO, use um app de tela cheia (Plex, Play Store).
- O YouTube, nesta TV, fecha sozinho ao ser aberto por `am start` e devolve a tela ao Lume (provavelmente não está logado), então não serve para as medições.

### Build de release na TCL (APK de 1,9 MB, assinado com chave de debug)

| Medida | Resultado | Orçamento |
|---|---|---|
| Primeiro plano, ocioso 30 s (TOTAL PSS) | **13,8 MB** (Native 4,5 MB, Java 1,2 MB, Code 3,1 MB, System 4,4 MB, Graphics 0) | ≤ 45 MB |
| Dormindo atrás do Plex, ~12 s (TOTAL PSS) | **13,6 MB**, Graphics 0 | ≤ 20 MB |
| Retorno a quente (Home → `window 1280x720`, 3 execuções) | 0,48 a 0,57 s | ≤ 300 ms até o primeiro quadro |
| Abertura a frio (Home → `window 1280x720`, 3 execuções) | 0,87 a 0,88 s (`Displayed` +470 a 490 ms) | ≤ 800 ms até o primeiro quadro |

Observações:
- Os tempos vão do `START u0 {HOME}` do `ActivityManager` até o log `window` do Lume, que **não inclui a criação do renderer nem o primeiro quadro**. Boa parte do intervalo é a transição de janelas do sistema, que o Lume não controla. Falta um log de "primeiro quadro desenhado" para fechar a medição (tarefa 14.1).
- Mesmo só até a janela, o retorno a quente (~0,5 s) já passa de 300 ms. Parte disso é a transição do sistema, mas é o principal ponto de atenção da tarefa 14.1 e pode exigir recalibrar esse orçamento na spec.
- A release usa metade da memória do build de debug (o debug carrega símbolos e DEX sem otimização).

### Bug encontrado só na release: o R8 removia a ponte JNI

Com `minifyEnabled true` e sem regras, o R8 removia `LumeActivity.bridge(String, String)`, chamado apenas pelo Rust. A TV mostrou o Lume em inglês, sem catálogo (só o tile "Settings"), com os logs `bridge command '...' failed: Method not found`. **Correção:** `android/app/proguard-rules.pro` mantém `LumeActivity.bridge`, `nativeHomePressed` e `LumeBridge`. Depois disso, a TV lista os apps reais, em pt-BR (idioma do sistema vindo da ponte).

### Aba Apps: serviços de sistema e ícones (2026-10-19)

- **O OK nos tiles funciona**: testei os 22 tiles da grade antigos por `adb input`, e cada OK abriu a activity certa. O "volta para o início" e o "vai para configurações" vinham de apps de sistema da TCL que estavam na grade: `copyDataToTV` e `TclHomePassive` fecham na hora (a TV volta ao HOME, que é o Lume), e o Android TV Remote Service, `Config.` e `Configuração Inicial` abrem telas de configuração.
- **Origem**: eram atividades com a categoria `LAUNCHER` comum (não `LEANBACK_LAUNCHER`) de apps de sistema. O launcher original só lista `LEANBACK_LAUNCHER`. O Lume agora descarta as só-`LAUNCHER` de apps de sistema, e a grade passa de 25 para 18 tiles (apps de TV e o tile "Configurações").
- **Ícones**: aparecem na grade (sem o nome dentro do tile) e nos favoritos (ícone mais o nome). Medido na release com ícones carregados: **18,3 MB** em primeiro plano e **16,9 MB** dormindo (Graphics 0), contra 13,8 MB e 13,6 MB sem ícones. Ainda dentro dos orçamentos de 45 e 20 MB.
- **Banners de TV** (como o Android TV Launcher): os apps de TV mostram o banner 16:9 cobrindo o tile, com os cantos arredondados do tile, e os demais mostram o ícone sobre a cor do tile. Memória na release: **16,4 MB** em primeiro plano e **15,5 MB** dormindo (Graphics 0), dentro dos orçamentos de 45 e 20 MB.

## Avaliação de desempenho na TCL (release com banners, 2026-10-19)

APK de release assinado com chave de debug, TCL 55C715 (4 núcleos, 1,4 GB de RAM, Mali-470, Android 9). O PSS vem de `dumpsys meminfo`, a CPU de `/proc/<pid>/stat` (ticks de 10 ms, relativos aos 4 núcleos) e os quadros do `dumpsys SurfaceFlinger --latency`.

### Resumo

| Item | Lume | Launcher original | Orçamento |
|---|---|---|---|
| PSS em primeiro plano, estável (30 s a 90 s depois de abrir) | **28 a 32 MB** | 40,9 MB (48,7 MB em outra medição) | ≤ 45 MB |
| PSS em primeiro plano, pico (0 a 10 s depois de abrir) | **51 a 53 MB** | n/a | ≤ 45 MB (**acima**) |
| PSS dormindo atrás de outro app (5 a 12 s) | **19,6 a 22,3 MB** | n/a | ≤ 20 MB em 10 s (**no limite**) |
| Graphics (`dumpsys meminfo`) | 0 | 0 | 0 dormindo |
| CPU parada na Home, foco no botão do carrossel (60 s) | **0,00%** (1 quadro/min) | 1,26% | < 1% |
| CPU parada na Home, carrossel girando (60 s) | **0,05%** (13 quadros/min) | n/a | < 1% |
| CPU parada na aba Apps (60 s) | **0,00%** (1 quadro/min) | n/a | < 1% |
| CPU de 16 teclas de rolagem da grade | 0,14 s no total | n/a | n/a |
| CPU de 80 teclas de troca de abas | 0,76 s no total | n/a | n/a |
| Threads | 15 | 38 | n/a |
| APK | **1,9 MB** (`liblume.so` 750 KB em ARMv7 e 1092 KB em ARM64, `classes.dex` 9 KB) | n/a | ≤ 6 MB |

### Vazamento
Sem vazamento: em 90 ciclos de troca de abas, o heap Java ficou em 1,7 MB e o heap nativo em 5,9 MB. O PSS só oscilou por páginas de arquivo (ver abaixo).

### Abertura
- Cronologia de uma abertura a frio (release, logs): `START` → processo +0,1 s → `lume starting` +0,3 s → `Displayed` **+0,54 a +0,61 s** → `window` (início do renderer) **+1,0 s**. O primeiro quadro desenhado vem depois do `window` e ainda não é medido (falta o log de "primeiro quadro", tarefa 14.1). O orçamento é 800 ms até o primeiro quadro, então a abertura a frio provavelmente passa dele.
- Pelo botão Home, antes dos banners: 0,87 s até o `window`.
- **O `TotalTime` do `am start -W` não serve para este app**: dá 0,65 s a 17 s para a mesma abertura (o sistema espera o primeiro quadro de uma activity que desenha pelo próprio EGL). Usar o `Displayed` do log.
- Retorno a quente: 0,2 s pelo `am start -W` (mesma ressalva) e ~0,5 s da tecla Home até o `window`, parte disso é a transição do sistema.

### O que explica o PSS
- **Pico de 51 a 53 MB na abertura**: ~23 MB são páginas **limpas** de arquivo (`.apk mmap`, privadas e reclamáveis pelo kernel), que caem para ~7 MB sozinhas em 20 a 30 s. A explicação mais provável é que o `PackageManager` abre o APK de cada app instalado para ler o rótulo e o banner (Netflix, Disney+ e Prime têm dezenas de MB). O pico vem do custo de abrir esses recursos, não do Lume. Para reduzir: guardar os banners num cache em disco (`cacheDir`, ~150 KB por app) e carregar só uma vez por versão do app.
- **Heap nativo de ~11 MB em primeiro plano** (contra 4,5 MB sem imagens): memória livre que o alocador ainda não devolveu depois de montar os banners. Reutilizar os `Bitmap` (já feito) não mudou o valor estável. Cai para ~6 MB dormindo.
- **Dormindo, ~6 MB de heap nativo** é o piso do processo; o Lume em si usa pouco.
- A TCL encerra o processo do Lume ~40 a 60 s depois de ir para segundo plano.

### Bug encontrado e corrigido
Na aba Apps (e em Configurações, Sobre e Editor), com o foco fora do botão do carrossel, o prazo do carrossel ficava vencido para sempre (só o desenho da Home o consumia) e o laço desenhava a **60 quadros por segundo sem parar, usando ~30% de um núcleo**. Corrigido: o carrossel só tem prazo na Home. Teste: `no_screen_but_home_keeps_the_carousel_timer_running`. Depois da correção, a aba Apps parada usa 0,00% de CPU.

### Não medido
Quadros perdidos na rolagem contínua (o `--latency` guarda só os últimos 128 quadros e as teclas pelo `adb` chegam a ~0,4 s uma da outra), YouTube em primeiro plano (fecha sozinho nesta TV) e o botão Home físico do controle.

### Depois da otimização de memória (cache de banners e sequência salva), 2026-10-19

A explicação do pico dada acima estava incompleta. O pico de 51 a 53 MB vinha, na verdade, de um **escaneamento completo do catálogo a cada início de processo**: o número de sequência do `getChangedPackages` ficava só em memória, então todo processo novo lia o rótulo de todos os apps abrindo o APK de cada um (`.apk mmap` de ~22 MB, páginas limpas). O design previa só aplicar as diferenças. Duas correções no `LumeBridge`:

1. **Sequência salva em disco** (`filesDir/change-sequence.v1`, junto com o `BOOT_COUNT` do aparelho, porque os números recomeçam a cada boot). Com ela, um processo novo só consulta as diferenças desde a última vez. Verificado na TV: desativar o Plex (`pm disable-user`) o tira da grade no processo seguinte, e reativá-lo o traz de volta.
2. **Cache em disco dos banners e ícones** (`cacheDir/tiles/`, ~147 KB por app, chave com a data de atualização do app). Um app atualizado ganha outro arquivo, e os de apps removidos são apagados.

| Item (release, TCL) | Antes | Depois | Orçamento |
|---|---|---|---|
| Pico de PSS na abertura (0 a 10 s) | 51 a 53 MB | **18 MB** | ≤ 45 MB |
| PSS estável em primeiro plano | 28 a 32 MB | **15,6 a 18 MB** | ≤ 45 MB |
| PSS dormindo (11 s) | 19,6 a 22 MB | **14,3 a 14,6 MB** (Graphics 0) | ≤ 20 MB |
| Heap nativo em primeiro plano | 11 MB | 4,9 MB | n/a |
| `.apk mmap` | até 24 MB | 1,2 MB | n/a |
| `Displayed` na abertura a frio | +540 a +610 ms | **+310 a +350 ms** | n/a |
| Início do renderer (`window`) a partir do `START` | ~1,0 s | ~0,5 s | n/a |
| CPU parada na aba Apps (45 s) | 0,00% | 0,01% | < 1% |

O primeiro uso, com o cache vazio, continua custando o escaneamento e a renderização dos banners (pico de 48 MB e ~13 MB de heap nativo), só uma vez.

## Primeiro quadro, animações e quadros perdidos (2026-10-19)

### Primeiro quadro (log `first frame` do Lume, release, TCL)
- **A frio** (processo morto, `START` → primeiro quadro): **725 a 904 ms** (mediana ~745 ms), contra o orçamento de 800 ms. O quadro sai 80 a 90 ms depois da janela e ~430 a 510 ms depois do início do processo.
- **A quente** (processo vivo, outro app na frente): **503 a 714 ms** (mediana ~520 ms), contra 300 ms. A parte do Lume (janela → primeiro quadro) é de **75 a 110 ms**. Cerca de 400 ms vão da camada do Lume aparecer no SurfaceFlinger até a janela chegar ao Lume, e esse tempo é da transição de janelas da TCL, fora do controle do Lume.
- Um tema próprio (`LumeTheme`, sem animação de janela e com o fundo do Lume) melhorou só a abertura a frio (mediana de ~745 para ~690 ms) e não mudou a quente.
- O primeiro quadro é desenhado sem as imagens dos tiles, que entram em seguida.

### Bug: as animações não rodavam na TV
`Animations::next_deadline` devolvia "agora + 16 ms", e o laço recalculava isso a cada espera, então o prazo nunca vencia: cada tecla gerava **um único quadro**, sem os quadros intermediários da animação de foco, da rolagem e da troca de slide. Corrigido em `LumeApp::next_deadline` (o próximo quadro é o do último quadro desenhado mais 16 ms, e o quadro que encontra tudo parado é o último). Testes: `frames_keep_coming_while_an_animation_runs` e os ajustes de `frames_are_built_only_when_something_changed` e `the_clock_only_asks_for_a_frame_when_the_minute_changes`. Na TV, cada tecla passou a gerar ~10 quadros em ~200 ms.

### Quadros perdidos (ainda acima do orçamento)
Com as animações rodando, cada animação de ~200 ms tem **1 a 3 quadros de 33 ms** (um vsync perdido) em ~10, ou seja, 10 a 30%, contra o orçamento de 1%. O custo por quadro medido é de ~8,5 ms de CPU. Já reordenei o desenho dos tiles em passes (caixas, imagens, texto) para reduzir as chamadas de GPU, e há um teste que fixa o número de lotes. Falta descobrir se o limite é a CPU (driver Mali) ou a GPU.

### Carrossel
Os 78 quadros em 30 s com o foco no botão do carrossel ficaram sem investigação: o usuário decidiu não seguir com o carrossel, e as melhorias sobre ele foram descartadas.

## Análise de desempenho da v0.2.0 (dock, papel de parede, abas por foco)

Medida em 19/10/2026 na TCL 55C715 (SoC rtd285o, 4 núcleos a 0,9 a 1,1 GHz, 2 GB de RAM), com o APK de release
(`lume-0.2.0-release-debugsigned.apk`, 3,14 MB), via `adb` (`dumpsys meminfo`, `/proc/meminfo`, `/proc/<pid>/stat`,
`am start -W`, `logcat`). Valores de um aparelho em uso normal, não de um laboratório.

### Memória do aparelho (sistema inteiro)

| | Valor | Observação |
|---|---|---|
| RAM total | 2.048 MB | |
| Livre (`MemFree`) | cerca de 115 MB (5,6%) | o Android usa o resto como cache |
| Disponível (`MemAvailable`) | cerca de 530 MB (25%) | o que dá para usar sem matar apps |
| Cache + buffers | cerca de 475 MB | recuperável |
| Em uso por apps (`Active`) | cerca de 680 MB | |
| Swap (zram) total | 600 MB | |
| Swap livre | cerca de 373 MB; **usado cerca de 232 MB (38%)** | variou de 232 a 235 MB nas leituras |
| Dados no zram | 205 MB originais → 73 MB comprimidos (2,8x) | `mm_stat` |
| Páginas para o swap desde o boot | 1,06 milhão de saídas e 77 mil de entradas | cerca de 4 GB escritos em zram |
| Carga (`loadavg`) | cerca de 21 em 4 núcleos | o sistema da TV (serviços TCL) está bem carregado, com ou sem o Lume |

Leitura: a TV trabalha com pouca RAM livre e com swap em uso o tempo todo. Isso torna importante o Lume ficar
pequeno e **não manter memória gráfica em segundo plano**, e explica por que o sistema às vezes encerra o processo
do Lume quando outro app pesado abre.

### Memória do Lume

| | Primeiro plano | Segundo plano | Orçamento |
|---|---|---|---|
| PSS total | 16,7 a 17,2 MB | 16,4 MB | 45 MB / 20 MB |
| Native heap (Rust) | 7,0 a 7,9 MB | | |
| Java heap | 2,2 a 3,4 MB | | |
| Código mapeado (.so, .jar, .apk) | 1,7 a 2,2 MB de PSS (73 MB de RSS, compartilhado) | | |
| Swap do processo | 15 a 67 KB | | |
| RSS | 106 a 110 MB, quase tudo arquivo compartilhado | | |

O Lume representa cerca de 0,8% da RAM. **A memória gráfica não aparece no PSS** nesta TV (`Graphics: 0`). Pela conta
do projeto ela é: atlas de ícones 1024x2304 RGBA = 9,4 MB, papel de parede 640x360 RGB = 0,7 MB e o atlas de glifos;
somando o PSS mais cerca de 10 a 11 MB, o total fica perto de 28 MB, dentro dos 45 MB. Em segundo plano o PSS quase não
cai (16,4 contra 16,7 MB) justamente porque a parte gráfica liberada nunca estava contada; o que sobra é o runtime.
O Java e o Rust do processo ficam residentes de propósito (retorno a quente rápido).

### Tempo

| | Medido | Observação |
|---|---|---|
| Abertura a frio (`am start -W`) | **371 ms** (TotalTime) | processo novo |
| Primeiro quadro | 650 ms depois do início do processo, 77 ms depois de a janela existir | log do próprio Lume |
| Retorno a quente (Home com Lume em 2º plano) | **127 ms** | spec: até 300 ms |
| Troca de aba, dock, grade | um quadro de animação a cada 16 ms; custo por quadro abaixo de 8 ms | em 6 movimentos do dock só um quadro chegou a 8 ms |

Antes da correção do dock havia saltos de 15 a 46 ms por volta no 1º uso (carga de ícones durante a animação) e a
posição 0 da animação sendo descartada: ver `docs/handoff-dock-aurora.md`.

### CPU e ciclos

`/proc/<pid>/stat` em ticks de 10 ms; ciclos estimados com 0,9 a 1,1 GHz (frequência dos núcleos do aparelho).

| Situação (30 s) | CPU do Lume | Núcleo ocupado | Ciclos |
|---|---|---|---|
| Início ocioso (foco no dock) | 0,07 s (6 ticks de usuário e 1 de sistema) | **0,2% de 1 núcleo** | cerca de 65 milhões (2 milhões por segundo, só o relógio por minuto) |
| Navegação contínua, 60 teclas (esquerda/direita a cada 0,5 s) | 3,37 s (226 de usuário e 111 de sistema) | **11% de 1 núcleo** | cerca de 3,0 a 3,7 bilhões (50 a 60 milhões por tecla, uma animação de 210 ms) |
| Início desde a abertura até esta medida | 1,73 s | | |

O Lume tem 20 threads, nenhuma delas trabalhando enquanto parado. O laço de eventos dorme até o próximo prazo
(relógio ou animação), por isso o início ocioso custa 0,2%.

### Sobre gerar log durante o desenvolvimento

O Lume **já gera log**, também na release: `android_logger` em nível `Info`, só eventos (versão ao iniciar, tamanho da
janela, tempo do primeiro quadro, janela liberada, avisos). São poucas linhas por sessão, sem log por quadro, e custam
praticamente nada. Não vale acrescentar log por quadro ou por tecla: ele custa CPU no laço de desenho e polui o `logcat`.
Para medir tempo basta instrumentação temporária (como foi feito para achar o engasgo do dock) ou, se virar rotina, um
`log::debug!` de tempo de quadro compilado só em `debug_assertions`.

## Fechamento da v0.2.0: quadros perdidos e memória (20/10/2026)

Mesma TCL 55C715, APK de release `lume-0.2.0-release-debugsigned.apk` (3,15 MB, `verify-apk.py` passou). Quadros medidos com
`dumpsys SurfaceFlinger --latency` (camada do `LumeActivity`) durante 40 a 60 teclas; "perdido" é um intervalo de 33 ms ou mais.

| Situação | Quadros | Perdidos |
|---|---|---|
| Foco na grade de Apps, antes da correção | 102 | **55 (54%)** |
| Grade de Apps sem a camada de escurecimento (teste) | 114 | 5 (4%) |
| Grade de Apps com o escurecimento no desenho do papel de parede | 115 | 0 e 1 (0 a 1%) |
| Dock (Início, esquerda e direita) | 118 | 0 |
| Troca de abas com o fade de 180 ms | 117 | 0 |

**Causa:** a camada `rgba(15,23,30,0,78)` de tela cheia das telas de texto, desenhada sobre o papel de parede, dobrava o
preenchimento de pixels por quadro na GPU da TV. Hoje o papel de parede e as estrelas são desenhados já escurecidos
(`Painter::wallpaper(true)`: fator 0,22 sobre o fundo `--bg` em que o quadro é limpo), com o mesmo resultado visual
(os testes de contraste em GL passam) e sem a camada.

| Memória do Lume (PSS) | Valor | Orçamento |
|---|---|---|
| Primeiro plano | 17 a 20 MB | 45 MB |
| Segundo plano (YouTube aberto, 12 s) | 12,2 MB | 20 MB |
| Abertura a frio (`am start -W`) | 373 ms | n/a |
| Retorno a quente | 12 ms de espera no `am start -W` (127 ms de ponta a ponta, antes) | 300 ms |

## Comparação entre as duas TVs de teste (2026-10-20)

Coletado por `adb` (dados brutos em `docs/device-specs/`). A TCL é o final `.6` e a AIWA o final `.3`.

| | TCL 55C715 (`.6`) | AIWA SMART TV (`.3`) |
|---|---|---|
| Fabricante / modelo | TCL `BeyondTV2` | KTC (marca AIWA), device `stanford`, build `KTC_AIWA_2K_SA_21V2` |
| Android | 11 (API 30), patch 2021-12-05, firmware global | 11 (API 30), patch 2024-01-01 |
| SoC (plataforma) | Realtek `rtd285o` | MediaTek `mt5867` |
| CPU | 4 × Cortex-A53, ARMv7 32 bits, até 1,1 GHz | 4 × Cortex-A53, ARMv7 32 bits, até 1,5 GHz |
| ABI | `armeabi-v7a` | `armeabi-v7a` |
| RAM | 2 GB (`MemAvailable` ~498 MB na coleta) | ~913 MB, `low_ram=true` (`MemAvailable` ~348 MB) |
| GPU | Mali-470 MP, OpenGL ES 2.0 | Mali-G31, OpenGL ES 3.2, Vulkan 1.0 |
| Tela | 1920×1080, 320 dpi, 60 Hz, HDR 1/2/3 | 1280×720, 213 dpi |
| `/data` | 10,1 GB, 18% usado | 4,0 GB, 82% usado (~755 MB livres após limpar caches) |
| Launcher do fabricante | `com.tcl.tv.tclhome_master` | `com.google.android.tvlauncher` |
| Configurações | `com.android.tv.settings.MainSettings` | `com.android.tv.settings.MainSettings` |
| `com.android.tv.action.VIEW_INPUTS` | `com.tcl.tv/.sourcemenu.sourcemanager.EnableViewInputLauncher` | `com.mediatek.wwtv.tvcenter/.nav.inputpanel.InputsPanelActivity` |
| Idioma do sistema | pt-BR | pt-BR |

Observações:

- A AIWA é o caso mais exigente: menos da metade da RAM, `low_ram`, 720p e pouco espaço livre. Os orçamentos de `resource-efficiency` só foram medidos na TCL.
- O menu de entradas abre nas duas TVs pela ação `com.android.tv.action.VIEW_INPUTS`, sem permissão especial. Na AIWA a abertura foi confirmada visualmente pelo autor (o `screencap` não captura o painel); o log do `ActivityTaskManager` registra a activity exibida em 131 ms.
- Em 2026-10-20 foram limpos os caches dos apps da AIWA (`pm trim-caches`): o `/data` passou de 3,38 GB para 3,33 GB usados.

## v0.2.1: entradas, engrenagem e painel de estatísticas (2026-10-20)

APK `lume-0.2.1-release-debugsigned.apk` (3,34 MB), instalado nas duas TVs.

- **O que um app lê (tarefa 4.1)**: nas duas TVs `/proc/stat` é bloqueado para o app (Android 11), e o Lume cai na frequência dos núcleos
  (`/sys/devices/system/cpu/cpu*/cpufreq`, log `stats: cpu source freq`). Memória (`ActivityManager.MemoryInfo`) e disco (`StatFs`) funcionam
  sem permissão. **A CPU do painel é, portanto, a frequência atual contra a máxima, não a carga**: com a TV ociosa o governador costuma
  subir o clock logo após uma tecla (viram-se 73% e 82%).
- **Botão de entradas**: OK abre o menu nas duas TVs (`START ... act=com.android.tv.action.VIEW_INPUTS`, TCL
  `EnableViewInputLauncher`, AIWA `InputsPanelActivity`). `<queries>` no manifesto faz o `resolveActivity` enxergar a ação no Android 11.
- **Engrenagem**: abre `com.android.tv.settings/.MainSettings` (TCL confirmada pelo `dumpsys activity`; a AIWA usa a mesma atividade).
- **Tile "Lume"**: abre as configurações do Lume; Voltar devolve o foco ao tile (AIWA, captura).
- **Custo do painel**: com a tela de Configurações aberta por 20 s, 9 a 10 quadros (um a cada 2 s, o painel); fora dela, nenhum quadro
  além do da saída. PSS na TCL com o painel: 28,6 MB (orçamento 45 MB).
- **Android 9**: sem emulador Android TV 9 neste ambiente, então a validação em API 28 (instalação, engrenagem e botão escondido
  quando a ação não resolve) **continua em aberto**; o único risco é a ação de entradas, que no Android 9 não exige `<queries>`.

## Escala tipográfica e régua (2026-10-20)

APK de release da change `escala-tipografica` (3,40 MB; a fonte de ícones, agora desenhada pelo script a partir dos SVGs do Lucide com traço 1,75, tem 30 KB).

| | TCL 55C715 (1080p) | AIWA (720p, `low_ram`) | Orçamento |
|---|---|---|---|
| Quadros de 33 ms ou mais na navegação (12 teclas) | 0 em 116 intervalos | 0 em 116 intervalos | ≤ 1% |
| PSS em primeiro plano, 30 s parado | 25,0 MB | não medido | ≤ 45 MB |
| PSS em segundo plano, 6 a 20 s (Graphics 0, `dormant`) | 19,9 MB | não medido | ≤ 20 MB (**no limite**) |

- No Android 11 o `dumpsys SurfaceFlinger --list` mostra três nomes para o Lume (com prefixo hexadecimal, `ActivityRecord{...}` e o simples); o `--latency` só tem os quadros no nome simples, `app.lume.tv/app.lume.tv.LumeActivity#N`.
- O PSS fica na linha `TOTAL PSS:` do `dumpsys meminfo app.lume.tv`; a coluna "TOTAL" da tabela de cima não é o PSS.
- A AIWA passou a responder "Service dumps disabled due to hung system process" ao `dumpsys meminfo`: o sistema dela desliga os relatórios quando um processo do sistema trava. Reiniciar a TV deve resolver; refazer a medida depois.
- `am start -a android.settings.SETTINGS` pode abrir um painel por cima sem tirar o Lume da tela; para medir o segundo plano, abrir um app de verdade (`monkey -p com.android.vending -c android.intent.category.LEANBACK_LAUNCHER 1`).

## Botão Menu dos controles (change `menu-de-contexto`, 2026-10-20)

- **AIWA (KTC, firmware MediaTek):** o botão Menu do controle chega aos apps como `KEYCODE_BOOKMARK` (174), visto no log
  do Lume (`key 174 not used`) com o autor apertando só esse botão. O Lume mapeia 174 como Menu.
- Teclas injetadas por adb na AIWA (`input keyevent`, com o Lume na tela): `MENU` (82) abre o menu; `INFO` (165),
  `TV_CONTENTS_MENU` (256), `BOOKMARK` (174), `PROG_RED`/`PROG_BLUE` (183/186) e `HELP` (259) chegam ao Lume;
  `SETTINGS` (176), `TV_MEDIA_CONTEXT_MENU` (257) e `TV_INPUT` (178) ficam com o sistema; `GUIDE` (172) abre outra tela.
- Na AIWA, `getevent` não mostra o controle IR (o receptor MStar injeta as teclas por um serviço do sistema) e o
  `HotKeyDispatcher` da MediaTek consome os botões globais (um deles abre as configurações do Android) antes dos apps.
  Para achar o código de um botão, use o log do Lume: `adb logcat | grep "not used"`.
- `screencap -p` na AIWA escreve duas linhas de texto antes do PNG: corte até o cabeçalho `\x89PNG`.
- **TCL:** o botão Menu do controle chega como `KEYCODE_MENU` (82) e abre o menu (conferido pelo autor).

## Splash (change `splash`, 2026-10-20)

APK de release da change `splash` (3,28 MB), as duas TVs reiniciadas antes das medidas. Os tempos vêm dos logs do Lume,
contados a partir da janela: `first frame`, `icons: N in X ms` (todos os ícones das duas abas carregados) e
`interface ready` (fim da revelação, com o número de quadros da revelação e quantos levaram 33 ms ou mais).

| | Rodadas | Primeiro quadro | Ícones prontos | Interface pronta | Revelação |
|---|---|---|---|---|---|
| TCL, retorno a quente para o Início | 10 | 71–87 ms | 189–229 ms | 420–460 ms | 15 quadros, 0 de 33 ms ou mais |
| TCL, retorno para a grade | 10 | 76–89 ms | 201–215 ms | 437–448 ms | 15 quadros, 0 |
| TCL, abertura a frio | 5 | 75–77 ms | 211–225 ms | 521–535 ms | 20 quadros, 0 |
| AIWA, retorno a quente para o Início | 10 | 88–107 ms | 167–297 ms | 400–533 ms | 15 quadros, 0 |
| AIWA, retorno para a grade | 10 | 85–105 ms | 158–253 ms | 395–487 ms | 15 quadros, 0 |
| AIWA, abertura a frio | 5 | 99–118 ms | 204–268 ms | 530–580 ms | 20 quadros, 0 |

Em nenhuma das 50 rodadas um ícone chegou depois da revelação. A revelação dura 220 ms no retorno a quente e 300 ms a frio;
o resto da espera é o splash: o logo entra em 120 ms enquanto os ícones carregam.

| Memória (`dumpsys meminfo`, `TOTAL`: PSS e Private Dirty) | TCL 55C715 | AIWA (`low_ram`) | Orçamento |
|---|---|---|---|
| Primeiro plano, 30 s parado | PSS 20,5–26,5 MB (privada 12–19) | PSS 39,0–43,4 MB (privada 21–23) | PSS ≤ 45 MB |
| Segundo plano, 15 s depois de abrir o YouTube | PSS 15,5 MB (privada 7,4) | PSS 23,2–25,8 MB (privada 5–6) | PSS ≤ 20 MB |

- **AIWA acima do orçamento em segundo plano pelo PSS.** A memória própria do Lume lá é de 5 a 6 MB; o resto do PSS são
  páginas compartilhadas com o sistema (`System` ~23 MB, `.apk mmap` e `.so mmap`), que no aparelho `low_ram`, com
  menos processos, ficam divididas entre menos donos. O build sem o splash também passou do orçamento na AIWA (30 a
  36 MB em segundo plano numa medida avulsa), então não é efeito do splash. O PSS da AIWA varia muito entre medidas
  (de 30 a 43 MB em segundo plano para o mesmo build): use a memória privada para comparar builds. Decisão pendente do
  autor: definir o orçamento de segundo plano pela memória privada, ou por aparelho.

### Como foi medido e o que se aprendeu

- **O `--latency` não vê a revelação.** Depois que a janela é recriada, o `dumpsys SurfaceFlinger --latency` da camada
  do Lume não registra os primeiros quadros (mesmo nome de camada, histórico vazio). Por isso o Lume mede a revelação
  ele mesmo (intervalo entre as trocas de buffer) e escreve no log `interface ready`.
- **TV com horas ligada fica lenta.** Antes de reiniciar, a revelação na TCL saía com quadros de 40 a 100 ms (o
  `render` esperava um buffer livre do compositor) e a AIWA tinha um processo de sistema travado ("Service dumps
  disabled due to hung system process"). Depois do restart, os mesmos builds ficaram com 0 quadros atrasados. Reinicie
  as TVs antes de medir quadros.
- **Ícones:** buscar e enviar os 14 a 16 ícones custa 30 a 140 ms no total (o cache em disco é de pixels crus, ~2 ms por
  ícone). O que fazia os ícones surgirem um a um era carregar um por volta do laço, com um quadro inteiro entre eles;
  agora cada volta carrega em lote (40 ms com o splash parado, 16 ms com a interface parada, 6 ms durante um fade).
- **Primeiro quadro da revelação:** a textura do papel de parede era criada nele (~70 ms de atraso); agora é criada
  durante o splash, enquanto o Lume espera os ícones.
- **Logo na janela do sistema (descartado).** Um `windowBackground` com o logo em VectorDrawable mostrava o logo desde o
  primeiro instante, mas o `NativeActivity` avisa o sistema que a janela foi desenhada antes do primeiro quadro do Lume,
  e aparecia um quadro preto. A correção exigia Java (`surfaceRedrawNeededAsync`); o autor preferiu o splash todo em
  Rust. A janela do sistema mostra só `--bg`, e o logo do Lume entra com fade.
- **`screenrecord --output-format=frames --size 640x360`** grava quadros crus que um script em Python lê sem
  decodificador: na TCL serve para conferir a passagem da janela do sistema para o splash (sem salto nem quadro preto).
  Na AIWA ele só capta a tela anterior e a interface pronta, não a transição. Os arquivos ocupam 20 a 40 MB em
  `/data/local/tmp`: apague depois (a AIWA ficou sem espaço para instalar).
- **AIWA cheia:** com 22 MB livres na partição de 4 GB, o `adb install` falha ("not enough space"); `pm trim-caches`
  liberou pouco, e o restart da TV liberou ~860 MB.
- **Um retorno a frio de 2,1 s na AIWA** (1 em 8, logo depois de uma gravação de tela): o teto de 600 ms não segura
  quando o laço fica preso numa chamada à ponte (ícones e sincronização do catálogo rodam na mesma thread do splash). Não
  se repetiu nas 7 aberturas seguintes.
- A grade de nenhuma das TVs rola (12 e 10 apps): o retorno para a grade rolada só está coberto pelos testes.
- **Build no contêiner:** o `cargo` às vezes reaproveitava código Rust antigo dentro do contêiner (dois builds com Rust
  diferente saíram com a mesma `liblume.so`); o `docker/Dockerfile` passou a tocar os `.rs` do projeto antes de compilar.

## Papéis de parede em Full HD (change `seletor-wallpapers`, 2026-10-21)

10 papéis em 1920x1080, embutidos na biblioteca (APK de 35 MB, com as duas arquiteturas; cada uma carrega os papéis). Textura do papel ativo: 6,2 MB.

| Medida (estável, 35 a 40 s depois de abrir) | TCL 55C715 | AIWA |
|---|---|---|
| TOTAL PSS | 21 MB | 50 MB (GL 18 MB); uma medida logo após instalar deu 62,7 MB |
| Primeiro quadro (após a janela) | 96 ms | 105 ms |
| Interface pronta (após a janela, abertura a frio) | 1300 ms | 1308 ms |
| Quadros de 33 ms ou mais na revelação | 0 de 38 | 0 de 39 |

Splash: logo à vista por no mínimo 600 ms a frio e 300 ms a quente, revelação de 600 ms a frio e 360 ms a quente. Os limites de `resource-efficiency` subiram (APK 40 MB, PSS 64 MB em primeiro plano).

## Antialiasing do tema (change `antialiasing-do-tema`, 2026-10-21)

### Base, antes do change (tarefa 1.1)

Build da `main` (`1d64123`, o mesmo conteúdo do APK `58b4986-dirty` instalado), TCL 55C715. Quadros pelo `--latency` da camada
simples, em 4 rodadas de 12 teclas com 0,4 s entre elas; intervalos acima de 150 ms são a espera entre teclas e não contam.
Grade: foco andando 5 tiles para a direita, uma linha para baixo, 5 para a esquerda e uma para cima. Dock: 6 para cada lado.

| Medida | TCL 55C715 |
|---|---|
| Grade de Apps, quadros de 33 ms ou mais | 16 de 458 (3,5%) e 18 de 460 (3,9%) |
| Dock, quadros de 33 ms ou mais | 0 de 422 |
| PSS em primeiro plano, 30 s parado | 18,1 MB |

- **Medida feita com a TV ligada havia 21 h** (carga média 20). Refeita com a TV recém-reiniciada, ainda com este build:
  grade **7 de 458 (1,5%)**, dock 0 de 417, PSS 28,1 MB. A carga média da TCL fica perto de 19 mesmo logo depois do
  boot (é o normal dela), e a grade já passa de 1% sem este change: é a base da tarefa 7.3.
- Na captura da TCL, a curva interna do contorno de foco do tile mostra pixels de azul mais escuro, misturados com o fundo
  do banner e o papel de parede: é o fio que este change remove.

### Depois do change (tarefas 6.3, 7.1 a 7.3), só na TCL

Mesmo método da base, TV reiniciada no mesmo dia, `--edge-softness` 1,5 (escolhido pelo autor olhando o painel 4K).

| Medida | Base (`main`) | Com o change |
|---|---|---|
| Grade de Apps, quadros de 33 ms ou mais | 7 de 458 (1,5%) | 8 de 459 (1,7%) e 5 de 459 (1,1%) |
| Dock, quadros de 33 ms ou mais | 0 de 417 | 1 de 427 (0,2%) |
| PSS em primeiro plano, 30 s parado | 28,1 MB | 33,1 MB (atlas de 1088x2610 em 1080p, contra 1024x2304) |

- **Dither por pixel derruba a Mali-470.** A primeira versão calculava o Bayer 4x4 com `floor`/`mod` no fragmento do
  papel de parede: **58 a 62% de quadros perdidos na grade e 44% no dock**. Sem o dither do papel de parede, 3 a 4%
  (sobrava o fragmento de imagem maior, com a composição sobre o fundo, rodando em tela cheia). Com o papel de parede
  num programa próprio e o dither lido de uma textura 4x4 em `REPEAT` (coordenada calculada no vértice), voltou à base.
  Regra: nada por pixel no quad de tela cheia além de leituras diretas de textura.
- **Coordenada de textura sem conta no fragmento.** Crescer o quad da imagem e prender a coordenada na borda dentro do
  fragmento vira leitura dependente em fp16 na Mali-470 (erro de cerca de 1 texel num atlas de 2600 linhas); a forma
  recortada passou a ficar meia rampa para dentro do quad.
- Capturas na TCL: dock e grade sem anel da cor do app e sem fio escuro entre o contorno e o banner; a curva interna do
  contorno vai do azul ao banner sem pixel mais escuro.

### TCL 55C715, remedida em 2026-10-21 (tarefa 7.3)

Mesmo método da base (4 rodadas de 12 teclas, 0,4 s entre elas, `--latency` limpo a cada rodada), APK `1d64123-dirty` com o
change, TV ligada havia ~1 h 7 min, carga média 19.

| Medida | Base (`main`) | Esta rodada |
|---|---|---|
| Grade, quadros de 33 ms ou mais | 7 de 458 (1,5%) | 7 de 459 (1,5%): 1, 1, 4 e 1 por rodada |
| Dock, quadros de 33 ms ou mais | 0 de 417 | 0 de 454 |
| PSS parado (30 s) | 28,1 MB | 21,1 MB (21,6 MB depois da navegação) |
| CPU do Lume nas 8 rodadas | n/d | 5,36 s |

- A grade empata com a base (1,5%, acima de 1% nos dois casos); o dock não perdeu quadro. A rodada com 4 perdidos é uma
  só; as outras três tiveram 1.

### AIWA SMART TV (tarefas 6.3 e 7.1)

APK `1d64123-dirty` com o change, 1280x720, Android 11, `low_ram`; instalado limpo (`adb uninstall` antes).

- PSS em primeiro plano: 86,5 MB logo após a abertura a frio (splash e carga dos ícones; fora do regime parado), **44,7 a 46,1 MB**
  depois de navegar dock e grade. Dentro do limite de 64 MB.
- Capturas: dock com favoritos redondos e grade com foco no Play Store sem anel da cor do app nem fio escuro entre o contorno e o banner.
- Quadros e CPU, 2 rodadas (4 voltas de 12 teclas no dock e 4 de 12 na grade, 0,4 s entre teclas; a carga média da AIWA é ~55, o laço
  de teclas levou ~68 s por fase). O `--latency` guarda só os últimos ~127 quadros, então os quadros contados (111 a 112 por fase)
  são o fim de cada fase, não ela toda.

| Medida | Rodada 1 | Rodada 2 |
|---|---|---|
| Dock, quadros de 33 ms ou mais | 0 de 112 | 0 de 112 |
| Grade, quadros de 33 ms ou mais | 0 de 111 | 0 de 111 |
| CPU do Lume no dock (utime+stime) | 1,83 s em 48 teclas (~38 ms por tecla) | 1,80 s |
| CPU do Lume na grade | 1,98 s em 48 teclas (~41 ms por tecla) | 1,96 s |
| CPU parado, 30 s | 0,01 s | 0 s |
| PSS parado | 44,8 MB (Graphics 14,5, Native 3,3, Java 1,7) | 44,5 MB |

- CPU médio de navegação: ~2,7 a 2,9% de um núcleo na janela de 68 s (cerca de 0,7% da capacidade dos 4 núcleos); o laço não
  pede quadros parado, como pede a spec. PSS a 70% do limite de 64 MB.


## Papéis de parede em Full HD: QOI, assets, redução na GPU e cache (2026-10-21)

Medidas com `change wallpapers-full-hd`, nas duas TVs de teste (TCL e AIWA, ambas com Android 11), pelos logs do próprio Lume
(`first frame`, `interface ready`, `wallpaper ... decoded / from the cache`) e por `dumpsys SurfaceFlinger --latency`. O `TotalTime`
do `am start -W` não serve (ver acima).

### Base (build da `main`, deflate dentro do `liblume.so`, APK de 36,8 MB)

| Medida | TCL | AIWA |
|---|---|---|
| Primeiro quadro depois da janela, a frio | 89 a 95 ms | 108 a 129 ms |
| Interface pronta (fim do splash) | ~1290 ms | ~1310 ms |
| PSS em primeiro plano / em segundo | 16,8 / 19,4 MB | 51,2 / 44,1 MB |
| Troca de papel de parede no editor | um quadro bloqueado pela decodificação | idem |

### Depois (QOI em `assets/`, decodificação em thread, envio em fatias, cache, APK de 21,6 MB)

| Medida | TCL | AIWA |
|---|---|---|
| Primeiro quadro depois da janela, a frio | 89 a 98 ms | 103 a 136 ms |
| Interface pronta | 1287 a 1302 ms | 1296 a 1332 ms |
| PSS em primeiro plano / em segundo | 14,7 / 16,9 MB | 48,7 / 46,3 MB |
| Decodificação QOI (thread) | 135 a 200 ms | 124 a 154 ms |
| Leitura do cache (imagem decodificada, 6,2 MB) | 28 a 30 ms | 21 a 62 ms (mediana 24) |
| Troca no editor, quadros | 28 quadros de 33 ms (o editor já corre a 33 ms nesta TV, igual à base) | 28 quadros, nenhum acima de 18 ms |
| Buffer de renderização | 1920x1080 | 1280x720 (a imagem é reduzida na GPU uma vez) |

- O formato foi escolhido na máquina de desenvolvimento (QOI decodifica 2,3 vezes mais rápido que deflate ou PNG e o catálogo cabe com
  folga: 18 MB); as medidas das TVs acima confirmam a decodificação em 124 a 200 ms por imagem, escondida pelo splash e pela thread.
- O cache vale a pena: a leitura é de 5 a 6 vezes mais rápida que decodificar.
- A TCL desenha o editor a 33 ms por quadro mesmo só movendo o foco, também no build da base: não vem da mudança.
- A AIWA (`low_ram`) já passava de 20 MB em segundo plano na base (41 a 44 MB); fica fora do orçamento de 20 MB, que vale para a TCL.
- Retorno a quente: sem mudança além da medida (um novo quadro depois da janela em 87 a 103 ms na AIWA e 87 a 92 ms na TCL).

## Tuning de performance na TV: perfil CPU/GPU, culling, batching e memória (2026-10-22)

Medições realizadas durante a change `tuning-performance-tv` (benchmark automatizado `perf_benchmark.rs` em `crates/lume-core/tests/perf_benchmark.rs`), comparando o baseline da `main` (`opt-level = "z"`, alocações por quadro e renderização sem culling vertical) contra a implementação otimizada (`opt-level = 3`, flags Cortex-A53 para ARM, culling de viewport, passo único em elipsização de texto, batching de draw calls, cache de licenças e short-circuit no carregamento de ícones).

### Resultados Comparativos (Antes vs. Depois)

| Operação | Antes (Baseline) | Depois (Tuning) | Ganho (*Speedup*) | Redução de Tempo / Recursos |
|---|---|---|---|---|
| `TextEngine::ellipsize` (100 chamadas com corte de palavra) | 412.40 µs | **119.99 µs** | **3.44x mais rápido** | -70.9% |
| Quadro da Grade de Apps (48 apps, viewport 1080p) | 312.72 µs | **115.36 µs** | **2.71x mais rápido** | -63.1% |
| Batches GPU na Grade de Apps | 8 batches | **7 batches** | **-1 batch** | -12.5% |
| Quadro do Editor (`Screen::Editor`, cartões e seções) | 300.82 µs | **130.52 µs** | **2.31x mais rápido** | -56.6% |
| Quadro da tela Sobre (`Screen::About`, 343 linhas de licença) | 578.87 µs | **88.95 µs** | **6.51x mais rápido** | -84.6% |
| `Shell::icons_wanted(1)` (1.000 chamadas sequenciais) | 1185.10 µs | **60.46 µs** | **19.60x mais rápido** | -94.9% |

### Principais Otimizações Implementadas

1. **Texto e Tipografia**:
   - `TextEngine::ellipsize`: passo único sem alocação dinâmica de `String` intermediária por iteração; busca do corte e elipsização em tempo linear.
   - `TextEngine::pen_positions_and_width`: cálculo conjunto de posições de glifos e largura de linha evitando re-medição redundante em `draw_line` e `draw`.
   - `SystemFonts`: cache de busca de intervalos de codepoints e registro de falhas de parse de fonte em tabela estática de faces.

2. **Culling de Viewport e Batching de Telas**:
   - **Grade de Apps**: `apps_viewport_culling` descarta linhas inteiras de aplicativos situadas acima ou abaixo da janela visível. Reordenação da emissão de primitivas agrupando caixas de fundo e logo no 1º loop e textos no 3º loop, garantindo exatamente 1 batch contíguo de texto para a grade de apps e reduzindo a fragmentação de comandos da GPU.
   - **Editor e Configurações**: Culling de cartões, seções e chips fora da janela visível no editor; emissão de animações restrita a frames de desenho ativo (`draw == true`).
   - **Licenças na tela Sobre**: Linhas de texto cacheadas via `OnceLock` estático, eliminando alocações repetidas de `split('\n')` a cada quadro de renderização.

3. **Backend OpenGL ES 2.0 e Upload de Papel de Parede**:
   - Reutilização de vertex buffer estático compartilhado em `reduce_wallpaper`, eliminando geração e destruição de VBOs a cada chamada.
   - Reutilização de buffer de vértices em `Renderer::render` usando `glBufferSubData` quando a capacidade atual suporta o tamanho do lote, mitigando realocações de memória na GPU.
   - Cache de `uniform locations` no `ShaderProgram` para a redução de wallpaper.
   - Upload do papel de parede particionado em fatias de 90 linhas (`WALLPAPER_UPLOAD_ROWS = 90`), completando o envio de imagens 1080p em 12 quadros sucessivos para eliminar engasgos na TV.
   - `read_pixels`: Inversão de linhas verticais in-place (`flip_rows_in_place`), eliminando buffer heap intermediário.

4. **Gerenciador de Ícones e Catálogo**:
   - `AppIcons`: Reciclagem e reuso de slots no atlas de texturas através de pilha `free_slots`, permitindo atualizar ícones sem recriar o atlas e evitando vazamento de capacidade.
   - `Shell::icons_wanted`: Filtro com dedup de pacotes e interrupção antecipada (`.take(limit)`) assim que a cota solicitada for atendida, reduzindo o tempo de resolução de 1.18 ms para 0.06 ms.
   - `Catalog`: Busca por pacote em $O(N)$ usando mapa de posições pré-computado em `choose_activities`, serialização/desserialização de pares chave-valor com reutilização de buffers sem alocações temporárias.


## Atalhos de configurações na AIWA (change `mapeamento-de-controle`, 2026-10-22)

AIWA SMART TV (KTC, MediaTek, Android 11, API 30). Medido por `adb` com o build debug: o `keys.v1` da TV recebeu um
código por atalho e cada um foi disparado com `input keyevent` com o Lume na tela.

- **Android 11 esconde as telas de configurações do `resolveActivity`.** Sem `<queries>` para os pacotes de
  configurações, todo atalho achava "nenhuma tela" e caía nas Configurações gerais (era o motivo de todos irem ao
  mesmo lugar). O manifest agora declara `com.android.tv.settings`, `com.android.settings`,
  `com.google.android.permissioncontroller` e o pacote de stubs.
- **A cascata do `LumeBridge.openSetting`** tenta o primeiro que resolve; um stub
  (`com.google.android.tv.frameworkpackagestubs`, "Nenhum aplicativo pode executar esta ação") e o seletor "Abrir
  com" (`ResolverActivity`) são pulados. Tela de cada atalho nesta TV:

  | Atalho | Tela |
  |---|---|
  | Timer / Sleep | `com.android.tv.settings/.partnercustomizer.timer.TimerActivity` ("Tempo de desligamento automático") |
  | Imagem | ação `android.settings.PQAQ.SETTINGS` (`MainSettingsSecond`: lista Imagem e Som) |
  | Som | `android.settings.SOUND_SETTINGS` (`DisplaySoundActivity`, "Tela e Som") |
  | Wi-Fi | `android.settings.WIFI_SETTINGS` (`NetworkActivity`) |
  | Bluetooth | ação `com.google.android.intent.action.CONNECT_INPUT` (`AddAccessoryActivity`, procura acessórios) |
  | Aplicativos | `.device.apps.AppsActivity` (a ação padrão cai num seletor) |
  | Armazenamento | `android.settings.INTERNAL_STORAGE_SETTINGS` (`StorageResetActivity`, "Armazenamento") |
  | Idioma | `android.settings.LOCALE_SETTINGS` (`LanguageActivity`) |
  | Data e hora | `.system.DateTimeActivity` (a ação padrão cai num seletor) |
  | Teclado | `android.settings.INPUT_METHOD_SETTINGS` (`KeyboardActivity`) |
  | Legendas | `android.settings.CAPTIONING_SETTINGS` (`CaptionSetupActivity`) |
  | Localização | `android.settings.LOCATION_SOURCE_SETTINGS` (`LocationActivity`) |
  | Segurança e restrições | `android.settings.SECURITY_SETTINGS` (`SecurityActivity`) |
  | Opções do desenvolvedor | `android.settings.APPLICATION_DEVELOPMENT_SETTINGS` (`DevelopmentActivity`) |
  | App de início padrão | `android.settings.HOME_SETTINGS` (`permissioncontroller`, `DefaultAppActivity`) |
  | Sobre a TV | `android.settings.DEVICE_INFO_SETTINGS` (`AboutActivity`) |
  | Android | `android.settings.SETTINGS` (`MainSettings`) |

  Sem tela própria nesta TV: `DREAM_SETTINGS`, `DISPLAY_SETTINGS`, `BLUETOOTH_SETTINGS`, `BATTERY_SAVER_SETTINGS`,
  `CAST_SETTINGS` e `PRIVACY_SETTINGS` (esta cai num stub).
- **Acessibilidade não tem tela nesta TV** (`ACCESSIBILITY_SETTINGS` é respondida pelo stub e as telas `oemlink.*`
  encerram na hora), o que levou a retirar o serviço de acessibilidade para atalhos em 2º plano: não havia como
  ligá-lo pela interface da TV. Os atalhos valem com o Lume em primeiro plano.
- **Teclas por `adb`:** `sendevent` é negado ao `shell` nesta TV; `input keyevent` chega ao Lume (serve para disparar
  os atalhos com o Lume na tela). O `getevent -lp` lista os dispositivos `MStar Smart TV IR Receiver` e `MTK TV KEYPAD`.

## Atalhos de configurações na TCL (change `atalhos-configuracoes-tcl`, 2026-10-24)

TCL 55C715 (BeyondTV2, Android 11, API 30). Medido por `adb` com o build debug: o `keys.v1` da TV recebeu um
código por atalho e cada um foi disparado com `input keyevent` com o Lume na tela.

- **Filtragem de stubs vazios (`EmptyStubActivity`):** O firmware da TCL registra `com.android.tv.settings.EmptyStubActivity`
  para responder a intents não suportados pelo fabricante (como `HOME_SETTINGS`). O método `leadsToScreen` descarta classes
  com o sufixo `EmptyStubActivity`, permitindo que a cascata prossiga e entregue o seletor do `permissioncontroller`.
- **Armazenamento seguro:** Na TCL, `ACTION_INTERNAL_STORAGE_SETTINGS` aponta para `StorageResetActivity` (Redefinição de fábrica).
  A cascata foi ajustada para priorizar `com.android.tv.settings.STORAGE` e `device.StorageSummaryActivity`, abrindo a tela de estatísticas
  e partições de armazenamento em vez do diálogo de reset.
- **Timer / Sleep e Protetor de tela:** A cascata de desligamento automático/sleep prioriza a tela do parceiro da AIWA (`partnercustomizer.timer.TimerActivity`), seguida pela tela oficial de descanso do Android TV na TCL (`device.display.daydream.DaydreamActivity`), que concentra o Temporizador de desligamento (10 a 240 minutos), Desligar a tela e Protetor de tela, com fallback no intent padrão `DREAM_SETTINGS` e nas Configurações gerais.
- **Tratamento de exceções na ROM da TCL:** A ação `ACTION_SOUND_SETTINGS` é mapeada no manifesto da TCL para
  `com.android.tv.settings.device.displaysound.SoundActivity`, mas a classe não existe no DEX do pacote `TvSettings.apk` da fabricante.
  O tratamento com captura abrangente de `Exception` no `LumeBridge` evita instabilidades no launcher.

Tela de cada atalho conferida nesta TV:

| Atalho | Tela |
|---|---|
| Timer / Sleep | `com.android.tv.settings/.device.display.daydream.DaydreamActivity` ("Protetor de tela": Temporizador de desligamento de 10 a 240 min e Desligar a tela) |
| Imagem | `android.settings.SETTINGS` (`MainSettings`, fallback suave) |
| Som | `com.android.tv.settings/.MainSettings` (evita `SoundActivity` ausente no DEX da TCL e faz fallback seguro em MainSettings) |
| Wi-Fi | `android.settings.WIFI_SETTINGS` (`NetworkActivity`) |
| Bluetooth | ação `com.google.android.intent.action.CONNECT_INPUT` (`AddAccessoryActivity`, procura acessórios) |
| Aplicativos | `.device.apps.AppsActivity` |
| Armazenamento | ação `com.android.tv.settings.STORAGE` (`StorageSummaryActivity`, partição e detalhes de disco) |
| Idioma | `android.settings.LOCALE_SETTINGS` (`LanguageActivity`) |
| Data e hora | `.system.DateTimeActivity` |
| Teclado | `android.settings.INPUT_METHOD_SETTINGS` (`KeyboardActivity`) |
| Legendas | `android.settings.CAPTIONING_SETTINGS` (`CaptionSetupActivity`) |
| Localização | `android.settings.LOCATION_SOURCE_SETTINGS` (`LocationActivity`) |
| Segurança e restrições | `android.settings.SECURITY_SETTINGS` (`SecurityActivity`) |
| Opções do desenvolvedor | `android.settings.APPLICATION_DEVELOPMENT_SETTINGS` (`DevelopmentActivity`) |
| App de início padrão | `android.settings.HOME_SETTINGS` (`com.google.android.permissioncontroller`, `DefaultAppActivity`) |
| Sobre a TV | `android.settings.DEVICE_INFO_SETTINGS` (`AboutActivity`) |
| Android | `android.settings.SETTINGS` (`MainSettings`) |

