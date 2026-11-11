# Tasks

## 1. Janela do sistema

- [x] 1.1 (Revista: splash todo em Rust; a janela do sistema fica lisa e o logo entra em fade de 120 ms, `--splash-logo-in`. A versão com VectorDrawable e retenção no Java foi feita, medida e retirada.) Criar `res/drawable/logo.xml` (VectorDrawable com as frações de `logo.rs`) e `res/drawable/splash.xml` (`layer-list` com `--bg` e o logo centralizado) e apontar `windowBackground` para ele; tamanho do logo no tema (`themes/lume/theme.css`); verificar com captura (`adb exec-out screencap -p`) da janela inicial na TCL, logo centralizado sobre o fundo

## 2. Fase de splash

- [x] 2.1 `phase` em `LumeApp` iniciada em `came_to_foreground`; desenho só do logo, no tamanho do tema; prazo de 600 ms em `next_timeout`; "ícones visíveis prontos" para dock e grade (falha conta como pronto); testes em `app/tests.rs`: segura enquanto falta ícone, solta ao carregar, solta em 600 ms, sem pedir quadros parado
- [x] 2.2 Preparar o indicador futuro: `ready`/`expected` e `progress()` na fase `Splash`, `draw_splash` com o gancho vazio `draw_splash_status`, e os tokens `--splash-status-top`/`--splash-status-height` no tema; testes de `progress()` (0, parcial, 1, e sem imagens esperadas = 1); verificar por captura que nada aparece abaixo do logo
- [x] 2.3 Tecla durante o splash encerra e é aplicada (Direita move o foco; Home segue as regras atuais); testes para os dois casos

## 3. Revelação

- [x] 3.1 `Anim::Reveal` dirigindo intensidade do papel de parede (`Painter::wallpaper` com intensidade), alfa do logo e deslocamento de 24 do conteúdo; 300 ms a frio e 220 ms a quente, ease-out, durações no tema; testes dos valores no início, meio e fim; capturas no `lume-desktop` em três momentos conferidas contra o design system (`lume-desktop --splash MS`)
- [x] 3.2 Log `interface ready` no `lume-app` no fim da revelação ou no corte por tecla; verificar no `logcat` da TCL

## 4. Verificação integrada

- [x] 4.1 Na TCL e na AIWA: abertura a frio e 10 retornos a quente (Início e grade rolada); conferir por captura que não há salto nem quadro preto entre a janela do sistema e o splash e que nenhum ícone surge depois da revelação
- [x] 4.2 Medir primeiro quadro, interface pronta e quadros perdidos (`dumpsys SurfaceFlinger --latency`) e registrar em `docs/device-notes.md`; metas da spec `resource-efficiency` e no máximo 1% de quadros de 33 ms ou mais
- [x] 4.3 Revisar a tela Sobre: se entrou algum componente de terceiros novo (o logo é do próprio Lume), conferir que aparece com a licença; senão registrar "Sobre revisado, sem mudança"; conferir com `cargo run -p lume-desktop -- --screenshot about a.png`. Sobre revisado, sem mudança (nenhuma dependência nova; logo e VectorDrawable são do Lume)
- [x] 4.5 Menu de contexto: itens com tamanho 18 e peso 500 (como os nomes dos apps); laterais de 24 (enchimento 16, largura mínima 200), conferidas por captura em pt-BR e en, no dock e na grade: 23 à esquerda e 28 à direita (a folga de 4 fica à direita); no dock, ~18 do cartão até as letras do nome; na grade, menu a 8 do anel de foco e com o topo no topo do anel; sombra do cartão 0 4 24 (era 0 16 40), para separar de tile branco; no dock, distância à caixa do nome de 16 para 4
- [x] 4.4 `openspec validate splash --strict`
