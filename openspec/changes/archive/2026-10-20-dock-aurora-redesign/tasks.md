# Tasks

## 1. Remover o carrossel e simplificar a configuração

- [x] 1.1 Tornar `HomeConfig::from_kv` tolerante: ler só `favorites`, ignorar `sections`, `section.*` e `carousel.*`, e gravar só `favorites` em `to_kv`; remover `sections`, `carousel_apps` e `interval` de `HomeConfig`. Verificar com testes em `home/config.rs`: arquivo antigo com seções e carrossel mantém os favoritos na mesma ordem, arquivo novo faz ida e volta, arquivo corrompido volta ao padrão.
- [x] 1.2 Remover `home/slides.rs`, o estado `Carousel`, `Anim::SlideFade`, `HERO_BUTTON`, `carousel_deadline`, `draw_hero`/`draw_slide`/`draw_pager` e o tratamento de Esquerda/Direita do carrossel em `app/mod.rs` e `app/home.rs`, deixando a tela Início só com a linha de favoritos atual. Verificar com `cargo build -p lume-core` sem avisos de código morto e com os testes de `app/tests.rs` ajustados passando.
- [x] 1.3 Remover as strings do carrossel (slides, "Apps no carrossel", intervalo, dica "Pelo menos um app", plurais) de `i18n`, `lume-strings` e dos XML de `values`/`values-pt-rBR`. Verificar com `cargo test -p lume-strings` e o teste de paridade de idiomas passando.
- [x] 1.4 Remover os seletores `.hero`, `.hero-badge`, `.hero-title`, `.hero-subtitle` e `.pager-dot` de `lume-css`, `themes/lume/theme.css`, `theme.rs` (inclusive o teste de `tint` do hero) e `docs/theme-format.md`, mantendo `.button`. Verificar com `cargo test -p lume-css` (incluindo `docs_examples_parse_without_diagnostics`) e um teste novo de que `.hero` gera aviso de seletor não suportado.
- [x] 1.5 Manter `usage.rs` gravando o histórico e conferir que nada de produção depende mais dos slides. Verificar com os testes de `usage` passando e uma busca por referências a `slides` sem resultados fora de comentários.

## 2. Editor sem seções

- [x] 2.1 Tirar de `editor.rs` as linhas de seção (subir, descer, ocultar) e o painel do carrossel, fazendo a tela principal abrir direto na edição de favoritos, com "Concluído" e "Restaurar padrão" (que agora só restaura os favoritos). Verificar com testes do editor: mover, remover e adicionar favorito, limite de 12, restaurar com e sem confirmação.
- [x] 2.2 Fazer o item "Adicionar favoritos" (lista vazia) abrir o editor e atualizar o caminho "Configurações › Tela Início › Editar". Verificar com um teste de foco e navegação do editor e uma captura de tela do `lume-desktop`.

## 3. Barra superior: abas sem contorno e engrenagem azul

- [x] 3.1 Remover o anel de foco das abas em `topbar.rs` e o `outline`/`border-radius` de `.tab:focus` em `theme.css`, mantendo só `color`. Verificar com um teste de renderização de que a aba em foco não gera comando de anel.
- [x] 3.2 Fazer o sublinhado azul seguir o foco: sob a aba em foco enquanto o foco está nas abas, senão sob a aba ativa, com a transição de `focus_progress`. Verificar com testes que cobrem foco no conteúdo, na aba ativa, na aba não ativa e na engrenagem.
- [x] 3.3 Mudar `.icon-button:focus` para `outline: 3px solid var(--accent)`. Verificar com `cargo test -p lume-css` e uma captura com a engrenagem em foco.
- [x] 3.4 Atualizar `docs/theme-format.md` para o foco das abas e da engrenagem e verificar que o teste dos blocos CSS da documentação passa.

## 4. Grade de Apps com 6 colunas

- [x] 4.1 Trocar `COLUMNS` para 6, calcular o tile em `(1152 - 5 * 24) / 6`, ajustar `ROW_PITCH` e o `.tile` do CSS para 172x97. Verificar com testes de geometria (6 colunas ocupam 1152 unidades sem sobra, tile de cerca de 172x97) e uma captura da aba Apps.
- [x] 4.2 Ajustar a navegação para 6 colunas (limite da 6ª coluna, última linha incompleta, Cima na primeira linha). Verificar com os testes de navegação da grade em `app/tests.rs` atualizados.

## 5. Papel de parede

- [x] 5.1 Gerar a base 640x360 RGB (borda das cortinas com cerca de 3 texels) e as posições das estrelas com `tools/wallpaper/aurora06.py`, embuti-las pelo `build.rs` de `lume-core` e expor o módulo `wallpaper` com `current()` como ponto único de carregamento. Verificar com testes: tamanho 640x360x3, 420 estrelas dentro da imagem de 1080p, base escura no topo e embaixo, e diferença média de no máximo 2/255 entre o quadro renderizado a 1920x1080 e `mockup/wallpaper-aurora-lume@1080p.png` (teste com GL em `tools/lume-desktop`); regerar os arquivos com o script dá o mesmo resultado.
- [x] 5.2 Adicionar ao renderer a textura RGB do papel de parede (filtro linear, `CLAMP_TO_EDGE`, criada e liberada junto com o contexto) e o seletor de textura no comando de imagem, quebrando o lote só quando ele muda. Verificar com testes de `render/batch.rs` e um teste de criação e liberação da textura no harness desktop.
- [x] 5.3 Desenhar o papel de parede no modo "cover" sobre toda a área visível, antes do conteúdo, em todas as telas, removendo o preenchimento liso de fundo e usando `--bg` só como reserva antes do upload. Verificar com capturas a 1280x720, 1920x1080 e uma proporção diferente de 16:9, sem faixas vazias.
- [x] 5.4 Desenhar as estrelas como retângulos de 1 ou 2 pixels do buffer de renderização, por cima da base (`Painter::wallpaper`). Verificar com um teste (GL) de que as estrelas ficam nítidas a 1080p e 720p (ponto brilhante sem halo a 4 pixels) e uma captura ampliada.
- [x] 5.5 Adicionar a camada de escurecimento `rgba(15, 23, 30, 0.78)` (valor medido: a aurora precisa de pelo menos cerca de 0,72; o 0,65 do plano não bastava) nas telas Apps, Configurações, Sobre e edição, e medir o contraste de texto (nome de app, rótulos, caminho de navegação) sobre o ponto mais claro da aurora, ajustando o valor até 4.5:1. Verificar com um teste de contraste que lê o quadro renderizado.
- [x] 5.6 Medir o custo: `dumpsys meminfo` (primeiro plano e segundo plano), tamanho do APK com `scripts/verify-apk.py` e quadros perdidos na rolagem da grade. Verificar contra a spec (textura até 1 MB, APK até +1,5 MB, 45 MB e 20 MB de PSS, 1% de quadros perdidos) e, se o APK apertar, aplicar o recuo do `design.md` (asset do APK). *(Medido em 2026-10-20 na TCL 55C715, release 0.2.0: APK 3,15 MB (limite 6 MB); PSS 17 a 20 MB em primeiro plano e 12,2 MB em segundo; quadros de 33 ms na navegação: 0 de 118 no dock, 0 de 115 e 1 de 115 na grade de Apps, 0 de 118 na troca de abas. Antes, a grade perdia 54% dos quadros por causa da camada de escurecimento de tela cheia, que foi trocada por um papel de parede já escurecido no próprio desenho.)*
- [x] 5.7 Documentar o papel de parede (como a imagem é gerada, como regerá-la, onde fica o ponto de carregamento) em `docs/wallpaper.md` e no `README`, e verificar que o comando de build citado roda como escrito.

## 6. Ícones quadrados do dock (atlas e plataforma)

- [x] 6.1 Estender `app_icons.rs` com a faixa de ícones de dock (8 colunas x 2 linhas de 128x128, atlas 1024x2304), reservando, localizando e liberando células como as da grade. Verificar com testes de reserva, capacidade (pelo menos 12) e coordenadas UV sem sangramento entre células.
- [x] 6.2 Adicionar ao `LumeBridge.java` a chamada `dockIcon` (ícone de 128x128 em quadrado arredondado, com ícone adaptativo desenhado por inteiro e recortado) e à ponte Rust o tratamento correspondente, com cache em disco por tipo e limpeza ao atualizar ou remover o app. Verificar com testes do protocolo em Rust e um teste instrumentado ou manual com ícones claros, escuros e transparentes.
- [x] 6.3 Carregar os ícones de dock só dos favoritos e aos poucos, mostrando a cor e a inicial enquanto não chegam, e liberá-los com o contexto ao ir para segundo plano. Verificar com um teste de que o primeiro quadro é desenhado antes dos ícones e que a textura some no segundo plano.
- [x] 6.4 Rever a spec `app-catalog` após a implementação e conferir a medição de memória do atlas maior (até 45 MB). Verificar com `dumpsys meminfo` registrado na descrição da tarefa. *(Atlas de 1024x2304 com a faixa do dock: PSS de 17 a 20 MB com os 12 favoritos carregados, bem abaixo dos 45 MB; a memória gráfica não aparece no PSS desta TV, e a conta do projeto dá cerca de 30 MB no total.)*

## 7. Dock com Magnification

- [x] 7.1 Adicionar os seletores `.dock` e `.dock-label` a `lume-css`, `build.rs`, `theme.rs`, `theme.css` e `docs/theme-format.md` (fundo translúcido, raio, preenchimento, espaço, sombra, cor e fonte do balão). Verificar com `cargo test -p lume-css` e os blocos de exemplo da documentação.
- [x] 7.2 Implementar o layout do dock em `app/home.rs` (ou módulo próprio): barra flutuante centralizada acima da borda inferior da área segura, até 12 itens sem rolagem, com tamanho base, espaço e preenchimento como constantes no código. Verificar com testes de geometria: 12 itens cabem dentro da área segura mesmo no pico de ampliação.
- [x] 7.3 Implementar a Magnification: posição de foco e amplitude animadas, escala por tabela de distância, vizinhos empurrados e base ancorada, com os fatores como constantes no código. Verificar com testes das escalas (1,6 / 1,27 / 1,09 / 1,0 no item do meio de cinco, normal com o foco fora do dock) e da animação sem salto.
- [x] 7.4 Desenhar os ícones quadrados sobre a cor do app (com o fallback de cor e inicial) e o balão com o nome do app em foco, dentro da área segura e com o nome truncado. Verificar com testes de posição do balão (item na borda, nome longo) e capturas com 3, 5 e 12 favoritos.
- [x] 7.5 Ligar a navegação: Esquerda e Direita dentro do dock, Cima para a aba ativa, Baixo sem efeito, `entry_down` da barra ao primeiro item, Voltar na raiz ao primeiro item, OK abre o app, e o dock vazio com um único item "Adicionar favoritos". Verificar com testes de foco em `app/tests.rs` para cada cenário da spec `home-screen` e `launcher-shell`.
- [x] 7.6 Garantir que, terminada a animação, o laço não pede mais quadros e que a tela Início ociosa desenha só o relógio. Verificar com um teste do cálculo de prazo do laço e medição de quadros no desktop.

## 8. Integração

- [x] 8.1 Gerar as capturas finais das telas Início (3, 5 e 12 favoritos), Apps, Configurações, edição e Sobre a 1280x720 e 1920x1080 com o `lume-desktop` e conferir cada cenário das specs alteradas (`home-screen`, `home-editor`, `launcher-shell`, `apps-screen`, `app-catalog`, `theming`, `wallpaper`, `resource-efficiency`, `localization`).
- [x] 8.2 Rodar no aparelho de referência: orçamento de memória em primeiro e segundo plano, retorno a quente e a frio, 60 fps na navegação contínua do dock e da grade, e `scripts/verify-apk.py` com o limite de 6 MB. Verificar com os números anotados na descrição da tarefa. *(Primeiro plano 17 a 20 MB e segundo plano 12,2 MB de PSS; retorno a quente sem passar de 12 ms de espera no `am start -W`, 127 ms de ponta a ponta medidos antes; abertura a frio 373 ms; navegação contínua sem quadros perdidos no dock e na grade; `verify-apk.py` passou com 3,15 MB. Detalhes em `docs/device-notes.md`.)*
- [x] 8.3 Atualizar os `Purpose` de `home-screen` e `home-editor` nas specs principais (hoje citam o carrossel) e as imagens de mockup que ainda mostram o carrossel no `README`. Verificar com `openspec validate dock-aurora-redesign --strict` e uma busca por "carrossel" que só encontra as specs arquivadas e as remoções registradas. *(Os `Purpose` das specs principais citam o dock; `openspec validate --strict` passa e a busca por "carrossel" só encontra remoções registradas e a compatibilidade com arquivos antigos; as imagens de `mockup/` são arquivos de design e ficam como estão, e o README usa capturas novas em `docs/screenshots/`.)*

## 9. Retorno de um app aberto

- [x] 9.1 Guardar, ao abrir um app (ou as Configurações do Android) pelo lume, a tela, o foco e a rolagem, e restaurá-los no próximo Home; descartar o ponto ao pressionar qualquer outra tecla, ao usá-lo e quando o app não abre (`app/mod.rs`). Verificar com testes de `app/tests.rs`: aba Apps com foco no YouTube, item do dock, grade rolada, tile de Configurações do Android, item de Configurações do lume, uso único, tecla antes do Home e app que não abre.
- [x] 9.2 Verificar na TV: abrir um app pela aba Apps e pelo dock, voltar pelo botão Home do controle e conferir a tela e o foco. Verificar com o resultado anotado na descrição da tarefa.
- [x] 9.3 Desenhar a engrenagem (glifo do ícone `SETTINGS`) no tile de Configurações do Android da grade, no lugar do nome dentro do tile (`app/apps.rs`). Verificar com um teste de `app/tests.rs` (um glifo grande no tile) e uma captura da aba Apps.
- [x] 9.4 Trocar a aba assim que ela recebe o foco com Esquerda ou Direita, sem OK (`app/mod.rs`, `horizontal`). Verificar com testes de `app/tests.rs`: Direita da aba Início mostra Apps com o foco na aba, Esquerda volta, OK na aba não muda nada, passar pela aba a caminho da engrenagem, Baixo entra no conteúdo da aba exibida.
- [x] 9.5 Corrigir o engasgo do dock do primeiro para o segundo ícone e suavizá-lo: a posição 0 da animação era descartada pela limpeza de valores em repouso e a animação recomeçava no destino; o balão do nome passa a deslizar junto com o foco; o carregamento de ícones espera o fim das animações, um ícone por volta; a animação passa a 210 ms. Verificar com testes de `app/tests.rs` e `app/dock.rs` e medição na TV.
- [x] 9.6 Colorir o sublinhado da aba: azul com o foco na barra superior (abas ou engrenagem) e branco com o foco no conteúdo, com a transição do foco (`app/topbar.rs`). Verificar com testes de `app/tests.rs`: dock, grade, abas, engrenagem e a cor intermediária durante a animação.
- [x] 9.7 Nomear as cores por papel (`--primary`, `--on-primary`, `--secondary`, com `--accent` e `--on-accent` como apelidos) e trocar o contorno de foco de branco para o azul primário em tiles, itens de Configurações, linhas do editor, chips e anel dos cartões do editor; documentar os papéis em `docs/theme-format.md`. Verificar com testes de `theme.rs` (papéis, apelidos e todos os contornos de foco na cor primária), `cargo test -p lume-css` (blocos de exemplo da documentação) e capturas da aba Apps.

## 10. Ajustes finais pedidos pelo autor

- [x] 10.1 Trocar a camada de escurecimento de tela cheia por um papel de parede desenhado já escurecido (imagem e estrelas pelo fator restante, sobre o fundo `--bg`), porque a camada custava 54% dos quadros da grade na TV. Verificar com os testes de contraste (Rust e GL) e a medição de quadros na TV (de 54% para 0%).
- [x] 10.2 Fazer o tile do lume na grade ser um app como os outros: retangular, com o logo, nome "Lume", em ordem alfabética, abrindo as Configurações do lume; remover o tile "Configurações" e a string `apps_settings_tile`. Verificar com testes de `app/tests.rs` (ordem, navegação, OK e Voltar).
- [x] 10.3 Escrever "Lume" com L maiúsculo ao lado do logo na barra superior.
- [x] 10.4 Destacar em branco o nome do app em foco na grade (`.tile-label:focus`). Verificar com teste de `app/tests.rs` e captura.
- [x] 10.5 Suavizar a troca de abas com um fade de cerca de 180 ms do conteúdo (`DisplayList::fade_from`, `Anim::TabFade`), mantendo a barra e o papel de parede fixos e o laço ocioso depois. Verificar com teste de `app/tests.rs` e medição na TV (0 quadros de 33 ms).
- [x] 10.6 Ordem alfabética sempre ativa, pelo nome mostrado no idioma atual, sem diferenciar maiúsculas e acentos (`sort_key`). Verificar com o teste da ordem da grade.
