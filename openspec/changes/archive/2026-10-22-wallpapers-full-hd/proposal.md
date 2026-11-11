# Proposal

## Why

O catálogo de papéis de parede promete alta definição, mas 9 dos 10 papéis vêm de masters JPEG de 1280x720 (cor 4:2:0, isto é, 640x360) ampliados com Lanczos: em Full HD não há detalhe real, e o scaler da TV, ao levar a imagem ao painel 4K, só realça a falta dele. Na TV 720p a GPU reduz a textura de 1920x1080 em 1,5x com filtro linear e sem mipmap, o que gera moiré nas linhas finas e devolve as faixas de cor que o dither feito em 1080p evitava. E o APK, com 36,8 MB de 40, leva os papéis de parede duas vezes, uma dentro do `.so` de cada arquitetura. O autor está produzindo novos masters em alta resolução; este change prepara o projeto para usá-los com a maior qualidade possível, sem perder desempenho.

## What Changes

- **Masters novos, em 3840x2160**, sem perda e com cor 4:4:4, fornecidos pelo autor. A origem e a licença de cada imagem passam a ser registradas. O mínimo aceito é 1920x1080 nativo: o gerador recusa master menor e master ampliado (detector de "falso HD").
- **Gerador em 16 bits**: calibra o contraste do topo e do dock, reduz o master por média para 1920x1080 e aplica o dither só no último passo.
- **No APK, 1920x1080**, a resolução do buffer do Lume; o scaler da TV amplia para 4K. Uma única cópia, em `assets/`, lida pelo `AAssetManager` do NDK (em Rust, sem Java), em vez de embutida em cada `.so`. **BREAKING** para quem troca papel de parede: os arquivos e o formato guardado mudam.
- **Formato guardado escolhido por medida** entre deflate, PNG e QOI (sem perda e de decodificação rápida), com um teste de tamanho e tempo de decodificação na TCL e na AIWA antes da decisão. Formatos com perda ficam fora: decodificam mais devagar e destroem o dither.
- **Buffer menor que 1920x1080 (TV 720p)**: no carregamento, a GPU reduz a imagem uma vez, com filtro de área e dither refeito na resolução final, para uma textura do tamanho exato do buffer; depois cada quadro amostra um texel por pixel. Na AIWA a textura cai de 6,2 para 2,7 MB.
- **Cache do papel de parede pronto**: a imagem decodificada do papel de parede ativo fica em cache no armazenamento do app; ao acordar (inclusive depois que a TCL encerra o processo), o Lume só lê o cache e envia à GPU, sem decodificar (a redução para 720p continua na GPU, a cada carga). O cache é refeito quando muda o papel de parede ou a versão do app. Medido: leitura de 21 a 62 ms contra 124 a 200 ms de decodificação.
- **Troca no editor sem travar**: a decodificação roda fora do laço de quadros; o papel atual fica na tela até o novo estar pronto, e a troca é direta (sem crossfade, que exigiria camada translúcida de tela cheia).
- **Spec corrigida** onde não bate mais com o projeto: o limite de 1 MB de memória gráfica e de 1,5 MB no APK, a resolução mínima de 640x360 com ampliação linear, a frase "ampliado pela GPU em telas 4K" (quem amplia é o scaler da TV) e o formato "Raw 24-bit RGB linear" como requisito.
- Quantidade, nomes e temas do catálogo ficam como hoje até os masters chegarem; se mudarem, o change inclui a atualização do catálogo, das traduções e das miniaturas.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `wallpaper`: requisitos de definição (1 texel por pixel no buffer, pré-filtragem em 720p, sem moiré nem faixas de cor), de custo de recursos (números reais de memória gráfica, uma cópia no APK, retorno sem custo novo), de catálogo (formato fora do requisito, identificadores preservados) e de troca (sem travar a navegação); novos requisitos de masters em alta resolução (sem falso HD), de origem das imagens e de cache do papel de parede pronto.
- `resource-efficiency`: o requisito do APK passa a exigir uma cópia só dos papéis de parede, fora das bibliotecas nativas, mantendo o limite de 40 MB.

## Impact

- Ferramentas: `tools/wallpaper/generate_wallpapers.py` e `tools/wallpaper/aurora06.py` (16 bits, redução por média, recusa de falso HD), `tools/wallpaper/masters/` (masters novos, fora do APK).
- Rust: `crates/lume-core/build.rs` (sai o embutimento em cada `.so`), `crates/lume-core/src/wallpaper/` (leitura do asset e decodificação), `render/gles2.rs` (redução por FBO no carregamento, textura no tamanho do buffer), `crates/lume-app/src/android/` (`AAssetManager`), `lume-desktop` (leitura dos mesmos arquivos).
- Empacotamento: `android/app/build.gradle` e `scripts/container-build.sh` (assets no APK), `scripts/verify-apk.py` (limite de 40 MB).
- Possível crate nova de decodificação (PNG ou QOI), registrada na tela Sobre com a licença.
- Testes: fidelidade (erro médio de no máximo 1/255 em 1080p e 2/255 em 720p contra referência Lanczos), banding, falso HD, contraste do topo e do dock, tamanho do APK.
- Medidas nas duas TVs, a TCL e a AIWA, antes e depois: tempo de carregamento, retorno a quente, troca no editor, quadros perdidos e PSS.
- Documentação: `docs/wallpaper.md`, e a decisão fixa "nenhuma foto de terceiros" do `CLAUDE.md` alinhada com a origem dos novos masters.
