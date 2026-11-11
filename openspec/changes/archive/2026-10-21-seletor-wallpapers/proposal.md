# Proposal

## Why

Atualmente o Lume possui um único papel de parede fixo (Aurora boreal a 640x360) e não oferece ao usuário a possibilidade de personalização visual da tela inicial. Além disso, a tela de edição da tela inicial possui um botão "Concluído" redundante, uma vez que todas as alterações já são persistidas imediatamente em disco e o botão "Voltar" do controle remoto é o padrão natural do Android TV para navegação de retorno.

Esta mudança introduz um catálogo de 10 papéis de parede elegantes e de alto contraste (a Aurora original refeita em 1280x720 com dithering, mais 9 novas imagens entre realismo/natureza e abstrato), adiciona uma seleção visual por Mini-Cards na tela de edição e remove o botão redundante "Concluído".

## What Changes

- **Catálogo de 10 Wallpapers de Alta Definição**: Inclusão de 10 opções no formato de máxima performance e nitidez (Raw 24-bit RGB linear sem perdas nem banding, com decodificação 0 ms na CPU e upload direto para a GPU via DMA):
  - *Aurora* (refeita em 1280x720 com dithering e 420 estrelas nítidas)
  - *Floresta Noturna*, *Ondas da Meia-Noite*, *Cânion Estrelado*, *Lago Espelhado*, *Dunas Negras*, *Névoa Glacial* (Realismo / Natureza)
  - *Fios de Luz Cósmica*, *Topografia Escura*, *Éter e Poeira Estelar* (Abstratos)
- **Garantia Estrita de Contraste**: Todos os 10 papéis de parede mantêm contraste mínimo garantido superior a 4.5:1 nas áreas críticas da interface (topo sob o relógio/abas e base sob o dock).
- **Seletor por Mini-Cards no Modo Edição**: Na tela de edição (Configurações › Tela Início), introdução de uma linha horizontal de mini-cards com prévia 16:9 de cada wallpaper, navegável por D-Pad, com troca de fundo em tempo real ao selecionar.
- **Remoção do Botão "Concluído"**: O botão redundante "Concluído" é removido da tela de edição. O rodapé mantém o botão "Restaurar padrão" e o usuário sai naturalmente pelo botão "Voltar" do controle remoto.
- **Persistência**: O wallpaper escolhido é gravado no arquivo de configuração `config.v1` (`wallpaper=<id>`) imediatamente a cada seleção, com fallback seguro para a Aurora clássica caso ausente.

## Capabilities

### Modified Capabilities
- `wallpaper`: Deixa de ser um papel de parede único e estático; passa a oferecer um catálogo de 10 wallpapers pré-definidos de alta definição com troca dinâmica em tempo de execução e calibração de contraste.
- `home-editor`: Adiciona o seletor de mini-cards de papel de parede acima do painel de favoritos e remove o botão "Concluído" do rodapé, tornando o botão "Voltar" a forma padrão de saída.
- `design-system`: Especifica as dimensões, espaçamentos e estados de foco dos mini-cards de seleção de papel de parede no canvas lógico 1280x720.

## Impact

- **Código Rust**: `crates/lume-core/src/wallpaper/mod.rs`, `crates/lume-core/src/render/gles2.rs`, `crates/lume-core/src/home/config.rs`, `crates/lume-core/src/app/editor.rs`, `crates/lume-core/src/app/painter.rs`.
- **Assets e Build**: `crates/lume-core/assets/wallpaper/`, `crates/lume-core/build.rs`, `tools/wallpaper/aurora06.py`.
- **Localização**: Novas strings para os nomes dos papéis de parede em `strings.xml` (en e pt-BR).
- **Sem quebra de compatibilidade**: Arquivos `config.v1` existentes continuam válidos (assumem `wallpaper=aurora` quando a chave estiver ausente).

## Qualidade para TVs HD a 4K

Os papéis são guardados em **1920x1080** (Full HD, a resolução própria da TV de referência; antes eram 640x360): a Aurora é gerada nesse tamanho e os 9 masters (JPEG de 1280x720) são ampliados com Lanczos na geração, o que dá mais nitidez que deixar a GPU ampliar a imagem pequena. Os masters não têm mais detalhe que isso; para ganhar detalhe real, basta substituí-los por versões em Full HD. Para os degradês escuros não formarem faixas em painéis grandes, o gerador aplica dithering triangular de ±1 nível (determinístico) na Aurora e onde a calibração escurece os pixels. A proteção do topo é calibrada pelo pico de luminância atrás da barra superior, e não pela média.

O custo é a textura de 6,2 MB do papel ativo na GPU (antes 0,7 MB) e um APK maior (cerca de 22 MB); por isso os orçamentos de `resource-efficiency` sobem (APK até 40 MB, PSS em primeiro plano até 64 MB).

## Splash

O logo passa a ser visto: entra em 300 ms (com crescimento de 0,96 a 1), fica no mínimo 600 ms (abertura a frio) ou 300 ms (retorno a quente) desde o primeiro quadro, e sai em 600 ms a frio e 360 ms a quente, com o logo saindo em 60% dessa duração.
