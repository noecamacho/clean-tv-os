# Design

## Context

Ver `proposal.md` (Why). O estado atual que molda a solução:

- `tools/wallpaper/generate_wallpapers.py` gera `crates/lume-core/assets/wallpaper/<id>-1920x1080.rgb` (RGB cru, 6,2 MB cada), `<id>-stars.txt` e as miniaturas; a Aurora vem do `aurora06.py`. A calibração de contraste trabalha em 8 bits sobre a imagem já ampliada.
- `crates/lume-core/build.rs` comprime cada `.rgb` com deflate e o embute com `include_bytes!`; a `lume-core` entra no `liblume.so` de cada arquitetura, por isso o APK leva tudo duas vezes (cerca de 15 MB por `.so`).
- `lume_core::wallpaper` descomprime com `miniz_oxide` a cada criação de textura; `render/gles2.rs` (`ensure_wallpaper`) envia o RGB à GPU com `GL_LINEAR` e sem mipmap, e o papel de parede é desenhado pelo `IMAGE_FRAGMENT`, cortado para cobrir a tela.
- O contexto gráfico é liberado ao ir para segundo plano (PSS de até 20 MB) e a TCL costuma encerrar o processo quando um app pesado abre; o retorno muitas vezes é uma abertura a frio.
- No Android, o `android-activity` já dá acesso ao `AAssetManager` (`AndroidApp::asset_manager`) e ao diretório interno do app (`internal_data_path`, usado pelo `storage.rs`), tudo em Rust.
- O `lume-desktop` (`tools/lume-desktop`) usa o mesmo renderer e lê os papéis de parede da `lume-core`.

## Goals / Non-Goals

**Goals:**
- Imagem 1:1 no buffer em qualquer resolução, a partir de masters com detalhe real (ver specs).
- Nenhum custo novo por quadro e nenhum custo novo no retorno (ver `Custo de recursos do papel de parede`).
- APK com uma cópia de cada papel de parede.

**Non-Goals:**
- Renderizar em 4K: o buffer continua limitado a 1920x1080 (`display-adaptation`).
- Papel de parede escolhido pelo usuário a partir de arquivo próprio.
- Animação ou transição entre papéis de parede.
- Mudar o catálogo (quantidade, nomes, temas) antes de os masters chegarem; se mudar, entra neste change como tarefa própria.

## Decisions

### 1. Pipeline de geração em 16 bits a partir de masters 4K

O gerador lê o master (PNG de 16 bits, TIFF ou PNG de 8 bits, sem perda), converte para ponto flutuante e faz, nesta ordem: calibração de contraste do topo e do dock (pelo pico de luminância, como hoje), redução por média para 1920x1080 (filtro de área; de 3840x2160 é exatamente 2:1), dither triangular de ±1 nível e quantização para 8 bits. A Aurora segue o mesmo caminho: é um master 4K do autor (o `aurora06.py` foi removido) e o gerador a reduz. A calibração sobre o master evita recalibrar no tamanho final.

Alternativas: guardar só em 3840x2160 (sem ganho no painel, porque o buffer é 1080p, e 4 vezes o custo); calibrar depois de reduzir em 8 bits (perde a precisão que o dither aproveita).

### 2. Detector de falso Full HD

O gerador reduz o master já em 1920x1080 para 1280x720 (Lanczos), amplia de volta (Lanczos) e mede a diferença média por canal na luminância. Abaixo de um limiar calibrado com os masters antigos de 720p (que falham, de 0,07 a 0,16) e com a Aurora em 4K (que passa, com 0,22): o limiar é 0,19, uma folga estreita porque a Aurora é suave por natureza, o master é recusado. O limiar fica numa constante do gerador, com o valor medido registrado em `docs/wallpaper.md`. A recusa de master menor que 1920x1080 é uma checagem direta.

### 3. Formato guardado: escolhido por medida

Candidatos, todos sem perda e com decodificação em Rust puro: deflate do RGB cru (o atual, `miniz_oxide`), PNG com filtros (crate `png`) e QOI (crate `qoi`). Uma medida com 2 masters reais nas duas TVs compara o tamanho no APK e o tempo de decodificação. Critério: o menor tempo de decodificação na AIWA (a TV mais fraca) cujo catálogo inteiro caiba no APK com o limite de 40 MB e pelo menos 5 MB de folga; empate, o menor arquivo. Com o cache da decisão 6, a decodificação só acontece na primeira vez, na troca e na mudança de resolução, então o critério pesa a troca no editor.

Formatos com perda (JPEG, WebP com perda) ficam fora: decodificam mais devagar que os candidatos e apagam o dither.

### 4. Uma cópia em `assets/` do APK

O `build.rs` deixa de embutir os papéis de parede. O empacotamento (`android/app/build.gradle`, `scripts/container-build.sh`) copia `crates/lume-core/assets/wallpaper/` para os `assets/` do APK, guardados sem nova compressão quando o formato já é comprimido. No Android, a `lume-app` lê o asset pelo `AAssetManager`; no desktop, o arquivo é lido do repositório. A `lume-core` recebe só os bytes (uma função de leitura injetada pela plataforma), sem saber de onde vêm. As estrelas e as miniaturas podem continuar embutidas: juntas, têm menos de 1,5 MB.

Sem Java: o `AAssetManager` é API do NDK.

### 5. Redução para buffers menores na GPU, uma vez

Quando o tamanho de cobertura do buffer é menor que 1920x1080 (TV 720p), a imagem de 1920x1080 é enviada à GPU como textura de origem, desenhada uma vez num framebuffer (FBO) do tamanho exato da cobertura com um shader de filtro de área (amostras bilineares ponderadas, que cobrem a área de 1,5x1,5 texels por pixel) e dither de ±0,5/255 nessa resolução, e a textura de origem é apagada logo depois. A partir daí, cada quadro amostra a textura de destino 1:1. A redução custa cerca de 1 ms de GPU e nada de CPU, o que importa na AIWA.

Alternativas: reduzir na CPU em Rust (dezenas de ms na CPU da AIWA, no caminho do retorno); mipmap (o GLES2 não faz mipmap de textura de tamanho fora de potência de 2 sem extensão, e o mipmap não acerta o tamanho exato); embutir uma versão 720p (dobra o tamanho do catálogo no APK).

Em superfícies de outra proporção, a imagem é cortada para cobrir, como hoje, e reduzida para o tamanho de cobertura; se a cobertura passar de 1920x1080 em uma dimensão, a textura é ampliada pela GPU com `GL_LINEAR`, como hoje.

### 6. Cache do papel de parede pronto

Depois de preparar a textura final (decodificada e, se preciso, reduzida), o lume grava os pixels prontos (RGB cru do tamanho do buffer, lidos do FBO com `glReadPixels` quando houve redução) num arquivo no diretório de cache do app (o diretório irmão `cache/` do `internal_data_path`, que o Android pode limpar e que não entra em backup). O nome do arquivo leva o identificador do papel de parede, um hash do asset e o tamanho do buffer; só o arquivo do papel ativo fica (os outros são apagados ao gravar). Ao acordar, se o arquivo existe e o nome bate, o lume o lê e o envia direto à GPU com `glTexImage2D`; se não, prepara a partir do asset e grava o cache. A gravação acontece numa thread de trabalho, fora do caminho do primeiro quadro.

Por que cache em disco, e não manter a imagem em memória: em segundo plano o contexto gráfico já é liberado (spec) e a PSS tem limite de 20 MB, que 6,2 MB de pixels comprometeriam; e a TCL encerra o processo, o que apaga qualquer cópia em memória. O arquivo sobrevive a isso, e a leitura de 6,2 MB do armazenamento ou do page cache do sistema é mais barata que decodificar e reduzir de novo. A medida da decisão 3 confirma o ganho; se a leitura do cache não for mais rápida que a decodificação na TV, o requisito do cache é revisto no mesmo change.

### 7. Preparação fora do laço de quadros

A decodificação (e a leitura do cache) roda numa thread de trabalho; o laço de quadros só envia os pixels prontos à GPU (e faz a redução da decisão 5). No retorno, o splash cobre o tempo de preparo, e o primeiro quadro completo já tem o papel de parede (spec). Na troca pelo editor, o papel anterior continua na tela até o novo estar pronto; escolhas que ficam velhas no meio da navegação são descartadas, e só a última é preparada.

## Risks / Trade-offs

- [Masters 4K pesam no repositório] → Os masters ficam em `tools/wallpaper/masters/`, fora do APK; se passarem de alguns MB cada, avaliar Git LFS com o autor antes de commitar.
- [Imagens com detalhe real comprimem pior] → A medida da decisão 3 confirma a folga dos 40 MB antes de gerar o catálogo inteiro.
- [`glReadPixels` do FBO é lento em algumas GPUs] → Acontece só ao gravar o cache, numa troca ou na primeira vez; medir na AIWA.
- [Cache corrompido ou de outra versão] → O nome leva o hash do asset e o tamanho; arquivo com tamanho errado é descartado e refeito.
- [O dither do gerador some na redução para 720p] → O shader de redução refaz o dither em 720p (decisão 5).
- [Pico de memória na redução] → Origem (6,2 MB) e destino (2,7 MB) só existem juntos durante a redução; a origem é apagada logo depois.

## Migration Plan

O identificador de cada papel de parede não muda, então a escolha gravada em `config.v1` continua valendo. O primeiro retorno depois da atualização não tem cache e prepara a imagem a partir do asset. Reverter o change é reverter o commit; o cache antigo fica órfão e é apagado na próxima gravação.

## Open Questions

- O diretório e o formato dos masters novos, que o autor informa quando estiverem prontos (as tarefas do gerador esperam por isso).
