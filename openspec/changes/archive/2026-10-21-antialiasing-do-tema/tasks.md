# Tasks

## 1. Base de medida e testes de borda

- [x] 1.1 Medir na TCL, com o build da `main`, os quadros de 33 ms ou mais na navegação da grade de Apps e do dock e a PSS em primeiro plano, e registrar em `docs/device-notes.md` como base do change; verificar que a nota cita a TV só pelo nome
- [x] 1.2 Adicionar ao demo do `lume-desktop` imagens sintéticas (ícone branco opaco e banner de cor sólida) para os favoritos e os tiles, só no desktop, e verificar com `cargo run -p lume-desktop -- --size 1920x1080 --favorites 3 --screenshot home h.png` que o dock mostra os ícones sintéticos
- [x] 1.3 Escrever em `tools/lume-desktop/tests/` os testes de borda das specs (anel da cor do app, fio escuro entre contorno e banner, largura da transição em 1280x720 e 1920x1080) e verificar que falham no código atual com `cargo test -p lume-desktop`

## 2. Uma única borda por forma

- [x] 2.1 Adicionar a cor de fundo ao vértice de imagem (`render/list.rs`, `render/batch.rs`) e a composição sobre ela com o fade só na imagem no `IMAGE_FRAGMENT`, e verificar com os testes de `render/batch.rs` que o vértice leva a cor e o raio
- [x] 2.2 No dock (`app/dock.rs`), desenhar o favorito com ícone numa passada só (sem o `fill` por baixo), mantendo o `fill` e a letra para o item sem ícone, e verificar que o teste do anel do favorito redondo passa e o fade de entrada continua mostrando a cor do app
- [x] 2.3 Na grade (`app/mod.rs`), compor banner e ícone sobre a cor do tile e tirar o fundo da passada de caixas quando há imagem, e verificar que o teste do anel do tile passa e o número de chamadas de desenho por quadro não aumenta (contagem em `Frame::build`)

## 3. Cantos dos banners no shader

- [x] 3.1 Remover do `LumeBridge.drawBanner` o `sourceBitmap`, o `BitmapShader`, o `drawRoundRect` e o `TILE_RADIUS_RATIO`, desenhando o banner direto na célula, e verificar que o APK compila com `scripts/container-build.sh release`
- [x] 3.2 Recortar o tile com banner com `image_rounded` e o raio `--radius`, e verificar que o teste de cantos do banner passa em 1280x720 e 1920x1080

## 4. Contorno de foco sem fresta

- [x] 4.1 Fazer o anel de foco de tiles com imagem avançar por baixo da borda da imagem até ela ficar opaca (duas rampas do buffer, por causa do recorte meia rampa para dentro), e verificar que o teste do fio escuro passa em 1280x720 e 1920x1080 e que a largura visível do contorno é a de antes (o afastamento de 3 px e o nome 4 px abaixo foram experimentados e desfeitos pelo autor)
- [x] 4.2 Atualizar o requisito "Foco e movimento" da `design-system`, se a medida do contorno mudar (não mudou), e verificar com `openspec validate antialiasing-do-tema --strict`

## 5. Largura da suavização e dither

- [x] 5.1 Adicionar `--edge-softness: 1.25px` ao `themes/lume/theme.css`, lido pelo tema e passado como uniforme aos shaders de forma e de imagem, e verificar com um teste de `theme.rs` que o valor chega ao renderer e com o teste de largura da transição
- [x] 5.2 Documentar `--edge-softness` em `docs/theme-format.md` e verificar que `cargo test -p lume-css` (exemplos do documento) passa
- [x] 5.3 Adicionar o dither ordenado 4x4 de ±0,5/255 no `RECT_FRAGMENT` (formas com degradê) e no `IMAGE_FRAGMENT` (só o papel de parede), e verificar com um teste no `lume-desktop` que a cor média por região de 16x16 muda no máximo 1/255 e que `cargo test -p lume-desktop --test wallpaper --test contrast` continua passando
- [x] 5.4 Suavizar os ícones da fonte de ícones com a rampa de `--edge-softness` na rasterização (pedido do autor na TCL; o texto não muda) e subir `--edge-softness` para 1,5, e verificar com os testes de `text/mod.rs` e o teste de largura da transição

## 6. Células do atlas no tamanho do pico

- [x] 6.1 Calcular em `app_icons.rs` o lado das células do dock e dos tiles pela escala corrente (múltiplo de 4) e o tamanho do atlas, e verificar com testes unitários os tamanhos de 1280x720 e 1920x1080 da tabela do `design.md`
- [x] 6.2 Recriar o atlas quando a escala muda e verificar com o teste de troca de resolução que os ícones voltam no novo tamanho
- [x] 6.3 Medir a PSS em primeiro plano na TCL e na AIWA com o atlas novo e registrar em `docs/device-notes.md`; se a TCL passar de 50 MB, voltar as células dos tiles ao tamanho atual e ajustar o requisito da `display-adaptation` no mesmo change

## 7. Verificação nas TVs

- [x] 7.1 Instalar na TCL e na AIWA, capturar com `adb exec-out screencap -p` o dock com favoritos redondos e a grade com foco, e verificar que não há anel nem fio escuro na captura
- [x] 7.2 Calibrar `--edge-softness` na TCL entre 1 e 1,5 olhando o painel 4K, registrar o valor escolhido no tema e verificar que os testes de largura continuam passando — 1,5, escolhido pelo autor na TCL
- [x] 7.3 Medir de novo os quadros de 33 ms ou mais na grade e no dock na TCL e verificar que ficam em no máximo 1% e não pioram em relação à base da tarefa 1.1 — grade 7 de 459 (1,5%), igual à base (7 de 458, já acima de 1%); dock 0 de 454; aceito pelo autor
- [x] 7.4 Rodar `cargo test --workspace && cargo clippy --workspace --all-targets` e `cargo fmt --all --check`, e verificar que ficam limpos
- [x] 7.5 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume): o change não traz componente de terceiros, licença, endereço ou versão nova; registrar "Sobre revisado, sem mudança" e verificar com `cargo run -p lume-desktop -- --screenshot about a.png` — Sobre revisado, sem mudança

## Workflow follow-up

- Arquivar com `/opsx:archive` e mesclar por squash na `main` quando o autor pedir.
