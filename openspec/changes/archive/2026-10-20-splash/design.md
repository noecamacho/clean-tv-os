# Design

## Context

- `LumeTheme` (`res/values/styles.xml`) usa `windowBackground` = `#0F171E` liso e nenhuma animação de janela.
- Ao ir para segundo plano o Lume libera o renderer, o atlas de glifos e a textura de ícones (`Shell::sleep`, `destroy_window`); ao voltar, `create_window` recria tudo e `Shell::wake` marca a tela suja. Os ícones são carregados alguns por volta do laço, dock primeiro, e **só quando `Shell::is_busy` é falso** (armadilha conhecida: carregar durante animação trava o dock).
- O logo é desenhado por código (`logo.rs`, SDF de retângulos arredondados e um círculo), sem textura: cabe no primeiro quadro.
- O papel de parede é desenhado com uma cor multiplicadora (`Painter::wallpaper`, `fade(Color::WHITE, keep)`) sobre o `--bg` em que o quadro é limpo.
- Medidas atuais (TCL): primeiro quadro a quente 75 a 110 ms depois da janela; ~400 ms de troca de janelas do sistema antes disso.

## Goals / Non-Goals

**Goals:** splash todo em Rust (premissa do projeto: Java só quando não há opção); não mostrar a interface incompleta; revelar sem custo de quadros.

**Non-Goals:** manter ícones em segundo plano; animar o logo durante a espera; mostrar a versão; desenhar indicador de carregamento (só a preparação, ver 2b); mexer no orçamento de 300 ms (a decisão pendente sobre ele continua separada).

## Decisions

### 1. Splash todo no Lume, com o logo entrando em fade

A janela inicial do sistema continua com `windowBackground` liso (`--bg`). O primeiro quadro do Lume desenha o logo sobre a mesma cor, com um fade de entrada de 120 ms (`--splash-logo-in`); assim a passagem da janela do sistema para o Lume é a mesma cor, e o logo surge em vez de saltar.

Alternativas descartadas:
- Logo também na janela do sistema (`windowBackground` com `layer-list` e um VectorDrawable igual ao `logo.rs`): mostrava o logo desde o primeiro instante, mas o `NativeActivity` informa ao sistema que a janela foi desenhada antes do primeiro quadro do Lume, o que expunha um quadro preto. A correção exigia segurar essa resposta no `LumeActivity` (Java), e nas medidas na TCL a retenção coincidiu com a revelação travada. O autor preferiu o splash todo em Rust.
- A API `SplashScreen` do Android 12, que não existe no Android 9 a 11 suportado.

### 2. Fase de splash no app

```
 wake --> Splash{desde} --(ícones visíveis prontos | 600 ms | tecla)--> Revelando{t} --(fim)--> Normal
                                                  \--(tecla)-------------------------------> Normal
```

`LumeApp` ganha `phase: Splash | Revealing | Normal`, iniciada em `came_to_foreground`. Na fase `Splash` o desenho é só o logo; nenhuma animação está ativa, então `is_busy` fica falso e o laço carrega os ícones normalmente. O teto de 600 ms entra como prazo em `next_timeout` (o laço acorda nele sem pedir quadros). A pergunta "ícones visíveis prontos?" é respondida pela própria tela de destino (dock: todos os favoritos; grade: os tiles na área visível), contando também os que falharam como prontos.

### 2b. Preparado para um indicador de carregamento

A prioridade é a experiência: transições suaves e curtas. Por isso não há indicador agora. A extensão fica pronta em três pontos, sem desenhar nada:

- **Progresso como dado:** a fase `Splash` guarda `ready` e `expected` (imagens prontas e esperadas na tela de destino) e expõe `progress() -> f32`. Os testes cobrem o valor. Um indicador futuro só lê esse número.
- **Desenho em camadas:** o desenho do splash fica numa função própria (`draw_splash`), com o logo e um gancho vazio `draw_splash_status(progress, área)`. Um indicador futuro (barra, pontos, texto "Carregando apps…") entra ali sem mexer no logo, na espera ou na revelação.
- **Espaço reservado no tema:** os tokens `--splash-status-top` e `--splash-status-height` definem a faixa abaixo do logo. Ela fica vazia e não muda a posição do logo.

Uma regra para o futuro, registrada aqui: um indicador só deve aparecer quando a espera passar de um limiar (por exemplo, 300 ms), para não piscar nas aberturas rápidas. Isso exigiria rever o teto de 600 ms, porque a espera deixaria de ser invisível.

### 2c. Carregamento dos ícones em lotes

Medido nas duas TVs: buscar e enviar à GPU os 14 a 16 ícones custa 30 a 90 ms no total (o cache em disco é de pixels crus, ~2 ms por ícone). O que fazia os ícones surgirem um a um era a ordem do trabalho: um ícone por volta do laço e um quadro inteiro entre eles. Agora cada volta carrega quantos couberem num orçamento (40 ms com o splash parado, 16 ms com a interface parada, 6 ms durante um fade leve) e a tela é desenhada uma vez por lote. O fade de entrada do logo e o fade dos ícones não bloqueiam o carregamento (`LumeApp::is_moving`); os demais movimentos continuam bloqueando, pela armadilha do dock. Durante o splash, o lote para 60 ms antes do teto.

O splash espera as imagens das duas abas: o dock e a primeira tela da grade (ou a parte visível da grade restaurada). Um ícone que ainda chegar depois entra com fade de 150 ms (`Anim::IconIn`), e a cor e a inicial do app saem sob ele.

Alternativas descartadas: uma thread de carregamento (duas threads chamariam o `LumeBridge`, que tem estado estático, e o ganho medido seria pequeno) e guardar os pixels do dock na RAM em segundo plano (economizaria ~25 ms por retorno).

### 3. Revelação como uma animação só

Uma animação de 0 a 1 (`Anim::Reveal`, valor inicial passado em `animate_to`, pela armadilha do `prune`) dirige as três partes: intensidade do papel de parede (`keep` da cor multiplicadora), alfa do logo (1 − t, só na área dele) opacidade da interface (t, em cada elemento, como o fade das abas) e deslocamento do conteúdo (24·(1 − t)); a barra superior só faz fade. Sem o fade, o conteúdo apareceria inteiro de uma vez sob o logo. Durante a revelação nenhum ícone novo é carregado (`is_busy` verdadeiro), mas todos os visíveis já estão prontos ou já esgotaram o teto.

### 4. Tecla durante o splash

A tecla leva a fase direto para `Normal` e é processada normalmente no mesmo quadro, inclusive Menu. Home é a exceção: segue as regras atuais (restaurar ou ir ao início) sem encerrar o splash, porque é ela que traz o Lume de volta e costuma chegar depois do primeiro quadro; se encerrasse, o retorno a quente nunca mostraria o splash. Para que a tecla encontre o foco certo, a tela de destino é diagramada a cada quadro do splash (foco, rolagem, tiles visíveis) e só o desenho dela é descartado; as animações dessa tela são assentadas, para que o laço continue carregando ícones.

### 5. Medição

O log `first frame` continua; um novo `interface ready: X ms after the window, Y ms after the process started` sai no fim da revelação (ou no corte por tecla). As medidas vão para `docs/device-notes.md`.

## Risks / Trade-offs

- [O logo aparece um pouco depois do Home (a janela do sistema mostra só a cor por 100 a 400 ms)] → aceito pelo autor em troca do splash todo em Rust; o fade de entrada evita o salto.
- [O splash atrasa a interface no retorno a quente] → sem tempo mínimo; com ícones vindos do cache em disco a espera esperada é de dezenas de ms. Se a medida mostrar espera longa, manter os ícones do dock em segundo plano (fora do escopo) é o próximo passo.
- [Revelação perde quadros na GPU da TV] → nenhuma camada de tela cheia; medir com `dumpsys SurfaceFlinger --latency` durante 10 retornos (meta: no máximo 1% de quadros de 33 ms ou mais).
