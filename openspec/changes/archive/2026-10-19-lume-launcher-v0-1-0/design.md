# Design

## Context

O projeto começa do zero: só existem o mockup (`mockup/*.png`, desenhado em @2x sobre um canvas de 1280 px de largura) e o OpenSpec. A motivação está em proposal.md; os requisitos estão nas specs desta change.

Restrições que moldam o desenho:
- **Dispositivo de referência**: TCL 55C715, com 2 GB de RAM, Android TV 9, build `release-keys` e sem root. Não há acesso a permissões privilegiadas (`ACCESS_ALL_EPG_DATA`), então não é possível ler os canais de recomendação de outros apps.
- **GPU**: TVs dessa faixa usam GPUs Mali, várias delas limitadas a GLES 2.0.
- **Processo Android**: todo app nasce do zygote. Mesmo com código nativo, existe um piso de memória (ART + framework) que o lume não controla. O orçamento de memória só pode cortar o que vem depois desse piso.
- **Distribuição**: GitHub primeiro, Play Store depois, sob GPL-3.0-or-later com CLA (prevendo uma futura versão Pro fechada num repositório privado).

## Goals / Non-Goals

**Goals:**
- Um único renderer próprio, pequeno, em Rust, que desenhe toda a interface do mockup a partir de primitivas procedurais.
- Separar a lógica (layout, foco, estado, tema) da plataforma, para que possa ser testada e visualizada no desktop.
- Um ciclo de vida explícito ATIVO ⇄ DORMINDO que torne mensurável o orçamento de memória em segundo plano.
- Um formato de tema (subconjunto de CSS) que o renderer já consuma na v0.1.0, ainda que só com o tema embutido.

**Non-Goals:**
- Sistema de plugins ou pontos de extensão para a versão Pro (serão criados quando o primeiro recurso Pro existir).
- Página HTML de preview de temas e carregamento de temas em runtime (fica para a release de temas).
- Layout flexível definido pelo tema (flexbox/`taffy`): na v0.1.0, o layout é código, e o tema só controla estilo e dimensões.
- Shaping de texto complexo e layout da direita para a esquerda (árabe, hebraico, índico): na v0.1.0 os glifos são exibidos na ordem lógica, sem shaping. O módulo de layout de texto fica isolado para receber `rustybuzz` e o espelhamento RTL numa release futura.
- Escolha de idioma dentro do lume: na v0.1.0 o idioma segue sempre o Android.
- Acessibilidade via TalkBack (não há árvore de Views). Fica registrada como dívida para uma release futura.

## Decisions

### D1. NativeActivity + ponte Java mínima, sem AndroidX/Kotlin
A atividade é uma subclasse fina de `NativeActivity` (`LumeActivity.java`), e a lógica fica toda em `liblume.so`, via o crate `android-activity` (backend `native-activity`). Uma classe Java (`LumeBridge`) expõe para o Rust, via JNI, só o que exige o framework: listar apps (`queryIntentActivities`), `getChangedPackages`, abrir intents, rótulo e cor dominante do ícone, preferência 12/24 h, lista de idiomas do sistema e encaminhamento de `onTrimMemory`.
- *Por quê*: sem Views não há HWUI, o renderer de Views do Android, que criaria o próprio contexto GL. Sem AndroidX/Kotlin, há menos DEX carregado e menos heap do ART.
- *Alternativas*: GameActivity (depende de AndroidX); Kotlin + Compose (dezenas de MB a mais); JNI 100% sem Java (possível, mas cada chamada ao PackageManager vira muitas linhas de JNI frágil, e a classe Java é pequena e testável).

### D2. Renderer próprio em OpenGL ES 2.0
Usamos EGL (crate `khronos-egl`) + `glow`, com três programas de shader:
1. **Retângulo SDF**: cantos arredondados, preenchimento sólido ou gradiente linear de 2–3 paradas, borda/contorno e sombra suave aproximada pela própria SDF. Cobre fundos, tiles, cards, chips, botões, foco e o logo.
2. **Texto/ícones**: amostragem de um atlas `ALPHA`/`LUMINANCE` de 8 bits, com cor por vértice.
3. **Imagem RGBA**: reservado para o banner e para usos futuros. A v0.1.0 praticamente não o usa.

Sem instancing (indisponível no GLES2), as primitivas viram quads num único VBO dinâmico, agrupados por programa. Os recortes (linhas roláveis) usam `glScissor`.
- *Por quê*: o mockup é quase todo procedural, então um tile custa ~6 vértices e nenhuma textura. GLES2 cobre todas as GPUs-alvo.
- *Alternativas*: `wgpu` (exige GLES3, binário e heap maiores); Skia (`skia-safe`, ~10 MB+); frameworks como Slint, egui e iced (mais memória e controle limitado do ciclo de vida da GPU).

### D3. Texto: `fontdue` + atlas sob demanda + fallback para fontes do sistema
Os glifos são rasterizados na CPU com `fontdue`, no tamanho final em pixels (escala × tamanho lógico), **sob demanda**: só entram no atlas (até 1024×512, 512 KB, 8 bits) os glifos de fato exibidos (textos do idioma ativo + nomes de apps). Quando o atlas enche, ele é reconstruído com os glifos visíveis. Mudança de escala ou de idioma invalida o atlas.
- **Fonte principal**: Plus Jakarta Sans (OFL 1.1, parecida com a do mockup), em subset latino estendido (Latin-1 + Latin Extended-A, que cobrem pt, en, es, fr, de, it e a maioria das línguas europeias de escrita latina), embutida no `.so`.
- **Fallback**: para um caractere ausente, o lume procura nas fontes do sistema (`/system/fonts`, ex.: NotoSans*, NotoSansCJK, DroidSansFallback), abertas via `mmap` só quando necessárias. São páginas mapeadas de arquivo, que o kernel pode descartar, e o fallback é liberado no estado DORMINDO. A lista de candidatas vem de `/system/etc/fonts.xml`.
- Usamos o kerning da fonte, e o shaping fica para depois (ver Non-Goals).
- *Alternativas*: `ab_glyph` (equivalente); `swash`/`cosmic-text` (shaping completo, mais pesado); embutir Noto para todas as escritas (dezenas de MB no APK).

### D3b. Idiomas no formato de recursos do Android, compilados no build
- **Fonte da verdade**: `android/app/src/main/res/values/strings.xml` (inglês, idioma de reserva), `values-pt-rBR/strings.xml` (idioma de referência do design) e futuros `values-<qualificador>/`. É o formato padrão do Android, com `<string>`, `<plurals>` e marcadores `%1$s`/`%1$d`, suportado nativamente por Weblate, Crowdin e Android Studio.
- **Compilação**: o `build.rs` do `lume-core` lê esses XMLs e gera tabelas estáticas (`&[&str]` por idioma, indexadas por um `enum StringId` gerado). Em runtime não há JNI, parsing nem XML: o texto vem direto da tabela. Os avisos de build cobrem chave faltando, chave desconhecida e marcadores incompatíveis entre idiomas.
- **Resolução do idioma**: o `LumeBridge` entrega a lista de idiomas do Android (`LocaleList`, em ordem de preferência) como tags BCP-47. O `lume-core` escolhe o primeiro idioma com tradução: primeiro a correspondência exata (`pt-BR`), depois só o idioma (`pt` → qualquer `pt-*` disponível) e, por fim, `en`. A troca de idioma chega pelo evento de mudança de configuração da NativeActivity, que recarrega a tabela e invalida o atlas.
- **Plurais**: um módulo pequeno com as famílias de regras CLDR (ex.: "um = n é 0 ou 1" para pt/fr; "um = n é 1" para en/es/de/it; "sem plural" para ja/zh/ko; famílias eslava e árabe) e uma tabela idioma → família. Um idioma novo com família já existente só precisa de uma linha na tabela.
- **Formatos**: hora 12/24 h segue a preferência do sistema. Números e tempos relativos ("há 2 horas") vêm de strings traduzidas com plurais, sem biblioteca de i18n completa.
- **Pseudo-idioma**: o harness desktop tem um modo `--pseudo-locale` que alonga os textos em 40% e acentua os caracteres, para testar truncamento e cobertura de fonte.
- *Alternativas*: ler as strings em runtime via `Resources.getString` (JNI a cada texto e strings no heap do ART); Fluent (`fluent-rs`, mais expressivo, mas formato menos suportado pelas plataformas de tradução e crates maiores); `gettext`/`.po` (bom suporte de ferramentas, mas fora do padrão Android).

### D4. Ícones como fonte e logo procedural
Os ícones da UI (engrenagem de Configurações, setas, lixeira, x, check, +, play) vêm de um subset de fonte de ícones de licença permissiva (Lucide, ISC), rasterizado no mesmo atlas do texto. O logo é composto por primitivas SDF (quadrado arredondado + "L" + ponto).
- *Por quê*: um único pipeline (atlas) para texto e ícones, nítido em qualquer escala, sem rasterizador de vetores.

### D5. Escala e buffer de renderização
O layout é calculado em unidades do canvas de 1280×720 (a mesma medida do mockup), com `escala = min(w/1280, h/720)` e o conteúdo centralizado. O buffer é limitado com `ANativeWindow_setBuffersGeometry` a, no máximo, 1920×1080 (escala 1,5), deixando a ampliação para 4K ao compositor. Usamos formato RGBA8888.
- *Alternativas consideradas e adiadas*: RGB565 + dithering (economiza ~12 MB em primeiro plano, mas cria banding nos gradientes) e renderização em 720p em TVs 4K (texto menos nítido). Como o primeiro plano não é o momento crítico de memória (ver D6), ficam como possível "modo econômico" futuro.

### D6. Ciclo de vida ATIVO ⇄ DORMINDO
```
 InitWindow ──> cria EGLContext/Surface, compila shaders, gera atlas ──> ATIVO
 ATIVO ── onStop / TerminateWindow / TRIM_MEMORY_UI_HIDDEN ──> DORMINDO:
          destroi Surface, EGLContext, VBO, atlas;
          libera caches; malloc purge (M_PURGE, API 29+)
 DORMINDO mantem: config, catalogo, historico de uso (KBs)
 DORMINDO ── InitWindow ──> ATIVO (recria tudo; meta < 300 ms)
```
O momento crítico de memória é quando outro app (o player) está em primeiro plano, e é aí que o lume libera tudo o que é de GPU, inclusive a memória do driver ligada ao contexto. A recriação é barata: 3 shaders pequenos e um atlas pequeno.
- *Alternativa*: manter o EGLContext vivo para voltar mais rápido. Isso deixaria vários MB de memória do driver presos enquanto o usuário assiste a vídeo, justamente o oposto da meta.

### D7. Laço de eventos sob demanda
O laço do `android-activity` bloqueia em `poll_events` com timeout = próximo prazo conhecido (fim de animação, próximo slide do carrossel, virada do minuto) ou infinito. Um quadro só é desenhado quando o estado está "sujo". Há uma única thread de UI. A consulta completa de apps e a extração de cores rodam numa thread auxiliar de curta duração, que termina ao concluir.

### D8. Arquitetura de crates
```
crates/
  lume-css/     parser do subconjunto CSS (sem deps) -> AST + avisos
  lume-core/    lib independente de plataforma:
                  theme (struct Theme gerada pelo build.rs a partir de themes/lume/theme.css)
                  layout (canvas 1280x720, telas Inicio/Apps/Configuracoes/Editor/Sobre)
                  i18n (tabelas de strings geradas do strings.xml, resolucao de idioma, plurais)
                  ui (foco, navegacao D-pad, rolagem, animacoes/tweens)
                  model (catalogo, config, historico, regras do carrossel)
                  render (lista de primitivas -> backend GL via trait)
  lume-app/     cdylib Android: NativeActivity, EGL, JNI (LumeBridge), persistencia
tools/
  lume-desktop/ binario de desenvolvimento (winit + glutin, contexto GLES2/GL compat):
                roda lume-core com catalogo falso, teclado = D-pad, e gera screenshots
android/        projeto Gradle minimo (manifesto, LumeActivity/LumeBridge em Java, banner, assinatura)
themes/lume/    theme.css (MIT)
```
O `lume-core` não conhece o Android: recebe eventos abstratos (tecla, tempo, catálogo) e emite primitivas. Isso permite testes unitários e de snapshot no desktop/CI e a comparação visual com o mockup (spec `theming`).
- A direção de dependência (`lume-app` → `lume-core` → `lume-css`) já prepara a futura versão Pro, que consumiria o `lume-core` como biblioteca, sem criar nenhum ponto de extensão agora.

### D9. Tema compilado pelo `build.rs`
O `build.rs` do `lume-core` lê `themes/lume/theme.css`, usa o `lume-css` para validar (avisos via `cargo:warning`, erro de sintaxe falha o build) e gera um `theme_generated.rs` com uma `const THEME: Theme`. Em runtime não há nenhum parsing. A mesma `struct Theme` e o mesmo parser servirão, no futuro, para carregar temas em tempo de execução.

### D10. Catálogo, cores e cache
- **Primeira abertura**: `queryIntentActivities` para LEANBACK_LAUNCHER e LAUNCHER, sem duplicatas e preferindo a de TV. As atividades só-LAUNCHER de apps de sistema (`FLAG_SYSTEM`/`FLAG_UPDATED_SYSTEM_APP`) são descartadas: na TCL elas são serviços e telas de fábrica (serviço do controle remoto, OTA, menus), que o launcher original não mostra. **Aberturas seguintes**: o catálogo vem do cache em disco, e `getChangedPackages(seq)` no `onResume` aplica só as diferenças. O número de sequência é gravado em `filesDir/change-sequence.v1` junto com o `BOOT_COUNT` (os números recomeçam a cada boot), para que um processo novo não escaneie tudo outra vez: o escaneamento lê o rótulo de cada app e abre o APK de todos, o que custava ~20 MB de páginas de arquivo e ~0,3 s na abertura.
- **Cor**: primeiro a tabela fixa pacote → cor (apps conhecidos, conforme o mockup). Na falta dela, o `LumeBridge` desenha o ícone num `Bitmap` de 24×24, calcula a cor dominante (histograma simples, descartando quase-branco/preto e escurecendo até o contraste de 4,5:1 com branco), recicla o bitmap e devolve um `int`. Só esse `u32` vai para o cache.
- **Ordenação**: rótulos normalizados (sem acentos e em minúsculas) para a grade alfabética.

### D10b. Banner e ícone nos tiles
Os tiles imitam o launcher de TV do Android: mostram o banner de TV do app (16:9, `PackageManager.getActivityBanner`/`getApplicationBanner`) e, na falta dele, o ícone (`getActivityIcon`/`getApplicationIcon`). O `LumeBridge.iconPixels` desenha a imagem num `Bitmap` de 256×144 (16:9, o formato do tile): o banner ocupa a célula inteira, com os cantos arredondados do tile (`BitmapShader` + `drawRoundRect`, antialiasing sem alterar shaders), e o ícone vai centralizado, com metade da altura, sobre transparência (a cor do tile aparece). Os bytes RGBA pré-multiplicados voltam pelo JNI num `byte[]` com um byte final (1 = banner, 0 = ícone).
O `lume-core` reserva uma célula de 256×144 numa textura RGBA única de 1024×2048 (até 56 apps) e guarda só o tipo da imagem. O `lume-app` envia os pixels com `glTexSubImage2D`, sem cópia na CPU. Um banner cobre o tile e já traz o nome do app. Um ícone, na grade, ocupa o tile e o nome fica sob ele; nos favoritos, o ícone sobe e o nome fica no tile.
As imagens vêm 2 por volta do laço, depois de cada quadro, com os favoritos primeiro, para não atrasar o primeiro quadro nem as teclas. A textura (8 MB) nasce na primeira imagem e vai embora com o contexto no estado DORMINDO. O programa de imagem já existia no backend (D2), então não há shader novo. Um app cuja imagem falha não é pedido de novo até a próxima janela.
Cada imagem pronta é gravada em `cacheDir/tiles/` (~147 KB por app; o nome tem o pacote, a data de atualização do app, a atividade, o tamanho e a versão do desenho, então um app atualizado ou uma mudança no desenho gera outro arquivo). Nas aberturas seguintes ela é lida do arquivo, sem abrir os recursos do outro app: o pico de memória da abertura caiu de ~53 MB para ~18 MB. Os arquivos de apps desinstalados são apagados na primeira gravação de cada execução.
- *Alternativas*: carregar só as visíveis com remoção LRU (menos memória, mais complexidade), ambos adiados.

### D11. Persistência
Três arquivos pequenos em `filesDir`: `config.v1` (seções, carrossel, intervalo, favoritos), `catalog.v1` (cache) e `usage.v1` (histórico). O formato é texto `chave=valor` versionado: legível em `adb shell run-as`, sem `serde`. A escrita é atômica (arquivo temporário + `fsync` + `rename`). Um arquivo ilegível ou de outra versão é ignorado (para `catalog.v1`, basta reconstruir) e registrado no log.
- *Alternativa*: SharedPreferences via JNI (carrega XML no heap do ART e adiciona dependência da ponte).

### D12. Build, assinatura e CI
- `cargo-ndk` gera `liblume.so` para `armeabi-v7a` e `arm64-v8a` (release e debug) e também para `x86_64`, **só no debug**, para rodar nos emuladores Android TV x86_64. Tudo com `-C link-arg=-Wl,-z,max-page-size=16384`. O perfil de release usa `opt-level="z"`, `lto="fat"`, `codegen-units=1`, `panic="abort"` e `strip=true`.
- O Gradle (AGP, sem dependências de bibliotecas) empacota APK e AAB. `minSdk 28`, `targetSdk` = o mínimo exigido pela Play no momento do release.
- No GitHub Actions, o PR roda `cargo test` + `cargo clippy` + `cargo-deny` (licenças compatíveis com GPL) + `assembleDebug`. A tag `v*` roda o build assinado (keystore em secrets), verifica o tamanho do APK (≤ 6 MB) e o alinhamento de 16 KB, e anexa o APK ao release.
- A mesma chave de app é usada no APK do GitHub e na Play (enviada no cadastro do Play App Signing), e o `applicationId` é definido uma única vez (ver Open Questions).

### D12b. Build local em container (wslc ou Docker)
O ambiente de build completo fica num `Dockerfile` versionado no repositório (`docker/Dockerfile`), para que qualquer máquina com um runtime de containers gere o APK sem instalar Rust, NDK ou Android SDK. O runtime principal no ambiente de desenvolvimento é o **`wslc`** (CLI de containers do WSL, já instalado: `C:\Program Files\WSL\wslc.exe` 3.0.3). O Docker continua aceito, para colaboradores e para o CI.
- **Dockerfile em estágios**:
  - `toolchain`: Debian slim x86_64 com JDK 17, Android cmdline-tools, `platforms;android-<targetSdk>`, `build-tools`, **NDK em versão fixa**, Rust na versão do `rust-toolchain.toml` com os alvos Android e `cargo-ndk` com versão fixa;
  - `build`: copia o repositório e roda o mesmo comando de build do CI, com caches de BuildKit (`RUN --mount=type=cache`) para o registro do Cargo, `target/` e `~/.gradle`;
  - `artifacts`: `FROM scratch`, contendo só `lume-<versão>-<variante>.apk` e `.aab`.
- **Sem bind mount**: os artefatos saem com `build --target artifacts -o type=tar,dest=dist/artifacts.tar`, e o script extrai o tar em `dist/`. Isso evita compilar sobre um sistema de arquivos compartilhado entre Windows e WSL (lento) e evita problemas de dono e permissão dos arquivos gerados. O `type=tar` é usado porque o `wslc` não suporta `type=local` (validado na tarefa 1.6), e o comando é o mesmo para `wslc` e `docker buildx`.
- **Script** `scripts/container-build.sh [debug|release]`: detecta o runtime (`LUME_CONTAINER` explícito, senão `wslc.exe`, senão `docker`). Para o `wslc.exe`, converte o contexto com `wslpath -w`. Um `.dockerignore` exclui `target/`, `build/`, `dist/` e `mockup/`.
- **Assinatura local**: com `LUME_KEYSTORE` e as senhas definidos, o keystore e as senhas entram como **segredos de build** (`--secret`), sem ficarem gravados em camadas da imagem, e o release sai assinado. Sem eles, o release sai assinado com a chave de debug e marcado como não publicável.
- **Paridade com o CI**: o workflow de release constrói o mesmo Dockerfile com `docker buildx`, então o build local e o do CI usam as mesmas versões de ferramentas.
- **Recursos**: o WSL tem ~7 GB de RAM, e o Dockerfile limita `CARGO_BUILD_JOBS` e a heap do Gradle (via `--build-arg`) para o LTO "fat" caber.
- *Risco*: o `wslc` é recente. A tarefa 1.6 valida logo no início (estágios, `-o type=local`, cache mounts e `--secret`). Se algo não funcionar, o fallback é Docker Engine no WSL com o mesmo Dockerfile.
- *Alternativas*: build direto no host (mais rápido no dia a dia e continua possível, mas não é reproduzível entre máquinas); Nix (reproduzível, porém com curva de aprendizado maior para colaboradores).

### D13. Licenciamento do repositório
O código fica sob `LICENSE` GPL-3.0-or-later. `themes/` fica sob MIT. O `TRADEMARKS.md` reserva o nome "lume" e o logo. O CLA Assistant fica ativo no repositório antes do primeiro PR externo. Os avisos de terceiros (OFL da fonte, ISC dos ícones, MIT/Apache dos crates) são gerados para a tela "Sobre".

## Risks / Trade-offs

- [Botão Home da TCL abre sempre o launcher do Google] → O spike (tarefa 2) testa isso logo no início. Mitigações documentadas no README: `cmd package set-home-activity` e, como último recurso, `pm disable-user --user 0 com.google.android.tvlauncher` (reversível). Se nenhuma funcionar, o escopo é revisto antes de seguir.
- [Piso de memória do processo maior que o previsto torna 20 MB em segundo plano inviável] → O spike mede o PSS de um NativeActivity vazio na TCL. Se o piso estiver acima, os orçamentos da spec `resource-efficiency` são recalibrados com base em dados (alteração explícita da spec), antes de construir as telas.
- [Driver Mali com custo fixo alto por contexto EGL] → Medido no spike. O ciclo DORMINDO já o elimina em segundo plano.
- [Recriar o contexto a cada Home deixa o retorno perceptível] → Shaders e atlas são pequenos, e a meta de 300 ms é medida. Se falhar, primeiro manter só o atlas na CPU (KBs), depois considerar `OES_get_program_binary`.
- [Sem Views, não há TalkBack/acessibilidade] → Aceito na v0.1.0 e registrado como dívida. Uma árvore de acessibilidade virtual pela ponte Java é possível no futuro.
- [Fonte do mockup não confirmada] → Plus Jakarta Sans é a escolha provisória. Trocar a fonte não muda specs nem arquitetura, só o arquivo do subset.
- [Cores de marca e nomes de terceiros] → Usados apenas para identificar apps instalados (uso normal de launcher). Screenshots da loja usam um catálogo de demonstração neutro.
- [Desktop harness divergir do Android (GL desktop vs GLES2)] → Os shaders são escritos em GLSL ES 1.00 e, no desktop, rodam em contexto GLES2 (ANGLE/Mesa) ou GL com o perfil compatível. A validação final é sempre na TV.
- [Kerning sem shaping pode diferir levemente do Figma] → A tolerância de 2 px lógicos está na spec de theming.
- [Fontes do sistema variam entre fabricantes, e algum caractere pode não existir em nenhuma] → O glifo ausente vira um retângulo neutro (tofu) sem travar. O harness testa o fallback com nomes em cirílico e CJK, e a TCL é verificada na integração.
- [Traduções da comunidade com marcadores errados (`%1$s` faltando)] → O build compara os marcadores de cada tradução com o idioma de referência e avisa. Em runtime, uma tradução inválida cai para o inglês em vez de exibir lixo.

## Migration Plan

Não há migração: é o primeiro release. Para instalar, use `adb install` (ou o APK do GitHub) e defina o lume como HOME. Para reverter, desinstale o lume ou reabilite o launcher de fábrica (`pm enable com.google.android.tvlauncher`), conforme o README.

## Open Questions

- **`applicationId` definitivo** (ex.: `app.lume.tv` ou um domínio próprio). Precisa ser fixado antes do primeiro release público (tarefa 13.1), mas não muda nenhuma outra decisão.
- **Conta da Play Store e Play App Signing**: só no momento do primeiro envio. O CI já gera o AAB.
- **Fonte final**: confirmar com o arquivo de design se é Plus Jakarta Sans ou outra fonte OFL.
