# Proposal

## Why

O Lume usa 9 tamanhos de texto (15, 16, 17, 18, 20, 22, 24, 28, 40); quatro deles (15 a 18) diferem de 1 px e, a 3 m de uma TV, se leem como um nível só, mas custam inconsistência: o mesmo papel (rótulo curto) aparece em 16 (tile), 17 (caminho) e 18 (balão do dock). Parte dos valores está fixa no código (título 40, subtítulo 18, cabeçalho de cartão 15 duplicado em `settings.rs` e `editor.rs`, marca 28, blocos da tela Sobre), contra a regra de fonte única do tema. E os tamanhos ímpares (15, 17) caem em meio pixel a 1,5x (1080p), o que borra o texto. A escala também precisa servir a TVs de 720p a 4K e a proporções diferentes de 16:9. Diretriz do autor: distância, tamanho e alinhamento são prioridade no frontend.

## What Changes

- A escala de texto passa a ter **6 níveis**: 40 (título de tela), 28 (marca), 22 (abas, relógio, seção, itens de lista, texto dos tiles, linhas do editor, títulos da Sobre), 20 (corpo), 18 (botão, balão, aviso, subtítulo, caminho de navegação, versão, rótulo de tile, chip, painel) e 16 (licenças de terceiros e cabeçalho de cartão em caixa alta). Saem 15, 17 e 24.
- **Barra superior uniforme** (movida da `versao-do-build`; revisada pelo autor na TV): tudo no eixo y=44; as duas pontas com a mesma altura, logo e marca à esquerda, ícones e relógio à direita (relógio de 28 na linha-base da marca, glifos de 36, com desenho de cerca de 80% da altura do logo e margem em cima e embaixo, em botões de 48 com 16 de vão); abas em 22 (o sublinhado as marca). Requisito "Barra superior" do design system. Hoje abas e relógio são 20, os glifos 27 e há 1 px de diferença de linha-base.
- Todos os tamanhos são inteiros **e pares**, para que o texto caia em pixel inteiro nas escalas reais (1x em 720p, 1,5x em 1080p).
- **Piso de texto 16 unidades lógicas** (hoje 14): em 720p é o menor tamanho legível a distância de TV; acima disso a escala só cresce com a resolução (a escala é uniforme, ver `display-adaptation`).
- Todo tamanho, peso e espaçamento de letras vira **token do `theme.css`**; o Rust deixa de fixar números de fonte (título, subtítulo, cabeçalho de cartão, marca, blocos da Sobre).
- **Responsividade verificada**: a mesma composição em 1280x720, 1920x1080, 4K (limitado a 1080p) e proporções diferentes de 16:9 (4:3 e 21:9, com sobras do fundo), sem texto cortado nem reticências novas.
- **Régua de layout** (pedido do autor): requisito novo do design system com unidade de 4, série de espaços, colunas, linhas fixas, tamanhos de componente, alturas de linha, regras de alinhamento e a receita de tela nova, para que toda tela (atual e futura) use as mesmas medidas; as medidas atuais fora da régua (dock 14, linha do editor 71, chip 33/42, subtítulo 154, Apps 119/143, Sobre 150/190) passam para ela.
- **Ícones e relógio com o peso do texto**: os ícones Lucide passam a ser desenhados pelo script com traço 1,75 (espessura do texto das abas no glifo de 36); o relógio fica em 28 com peso 500 (mesmo traço das abas) e os botões acompanham a largura da hora, com vão constante.
- A tela Sobre é revisada ao fim (última tarefa).

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `design-system`: o requisito "Tipografia" ganha a escala de 6 níveis, os papéis de cada nível, a regra de tamanhos inteiros e pares e a de tokens; novo requisito "Barra superior" (eixo, tamanhos, linha-base, glifos, espaços); novo requisito "Régua de layout" para todas as telas; o requisito de canvas remete a eles.
- `display-adaptation`: o tamanho mínimo de texto sobe de 14 para 16, e a escala passa a ter cenários de resolução e proporção.

## Impact

- Tema: `themes/lume/theme.css` (tokens de escala, componentes), `crates/lume-css` e `crates/lume-core/src/theme.rs` (piso de 16, testes).
- Telas: `app/page.rs`, `app/settings.rs`, `app/editor.rs`, `app/topbar.rs` (linhas-base, glifos de 24, posições das abas) e quem desenha o dock, os tiles e o painel de estatísticas, para ler os tokens e caber no texto maior (tile label, chip, painel de 16 para 18).
- Sem custo de memória nem de quadros esperado; os atlas de glifos já são gerados por tamanho.
- Independente da `versao-do-build`: nenhum arquivo em comum além da tela Sobre.
