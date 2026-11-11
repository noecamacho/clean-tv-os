# Design

## Context

- A escala é uniforme: canvas lógico 1280x720 × `min(largura/1280, altura/720)`; 720p é 1x, 1080p é 1,5x, 4K é limitado a 1080p (1,5x) e ampliado pelo sistema; em outras proporções o conteúdo fica centralizado com sobras (spec `display-adaptation`). Portanto o texto é responsivo por construção; o que muda entre TVs é só a resolução e o tamanho físico.
- Hoje: 9 tamanhos (15, 16, 17, 18, 20, 22, 24, 28, 40) e piso `MIN_FONT_SIZE = 14`. Fora do tema: título 40 e subtítulo 18 (`page.rs`), cabeçalho de cartão 15 (`settings.rs`, `editor.rs`), marca 28 (`topbar.rs`), blocos 24/20/16 da Sobre (`settings.rs`).
- Barra superior hoje: abas e relógio 20 (linha-base 51 e 52), glifos 27 em botões de 48 (eixo y=44), `TABS_X` 228, `TAB_GAP` 36.

## Goals / Non-Goals

**Goals:** poucos níveis com degrau visível; texto nítido em 1x e 1,5x; legível de 32" a 65"; uma fonte única (tema).
**Non-Goals:** trocar a fonte (Plus Jakarta Sans) ou os pesos (500, 700, 800); densidade por tamanho físico da tela (o Android não informa a distância de visão; a resolução é o único sinal confiável); mudar medidas de layout além do que o texto maior exigir.

## Decisions

### 1. Seis níveis, todos pares

| Nível | Tamanho | Papel |
|---|---|---|
| Título | 40 | título de tela (800) |
| Marca | 28 | "Lume" na barra (800) |
| Principal | 22 | abas, relógio, seção, itens de Configurações, texto dos tiles, linhas do editor, títulos da Sobre |
| Corpo | 20 | texto base, corpo da Sobre |
| Rótulo | 18 | botão, balão, aviso, subtítulo, caminho, versão, rótulo de tile, chip, painel |
| Legenda | 16 | licenças de terceiros; cabeçalho de cartão em caixa alta |

Razão de UX: degraus de pelo menos 2 px entre vizinhos mantêm a hierarquia legível a 3 m, e o peso e a cor sustentam o resto (aba ativa 700 e branca, inativa 500 e mutada). Um papel (rótulo curto) tem um tamanho só. Os pares garantem pixel inteiro a 1,5x (16→24, 18→27, 20→30, 22→33, 28→42, 40→60); 15 e 17 davam 22,5 e 25,5.

### 2. Piso de 16, não de 14

A 720p (a AIWA é 1280x720 de ~32 a 43") 1 unidade é 1 px, e 14 px é pequeno para quem senta a 2 a 3 m. Com piso 16 o menor texto (licenças) é o menor da escala, e tudo o mais cresce. Em 1080p/4K o mesmo texto vira 24 px, proporcional à tela maior. O aviso de build (`MIN_FONT_SIZE`) sobe para 16 e o teste de tema confere todos os componentes.

### 3. Responsividade: o que se verifica

Não há breakpoints: a escala uniforme é a regra. A verificação é por captura, no desktop (`--size`), em 1280x720, 1920x1080, 3840x2160 (limitado a 1080p), 1024x768 (4:3) e 2560x1080 (21:9), nas telas Início, Apps, Configurações, Sobre e editor, e na TCL (1080p) e na AIWA (720p). Critérios: nada fora da área segura (32/18), nenhum texto novo com reticências nem sobreposição, a composição idêntica em 16:9 (só a escala muda) e sobras centralizadas nas outras proporções.

### 4. Barra superior numa régua só

Eixo central y=44 para logo, marca, abas, relógio e botões. Revisão do autor na TV: com relógio de 22 e glifos de 24 a ponta direita ficou pequena perto do logo e da marca. Decisão: as duas pontas têm a mesma altura, logo (40) e marca (28) à esquerda, ícones e relógio à direita, com o relógio de 28 na linha-base da marca (56) Ícones: com glifo de 48 o desenho tinha os mesmos 40 do logo, mas parecia maior e sem margem (o traço vai de ponta a ponta, o logo é um bloco cheio de cantos arredondados); a correção ótica é um glifo de 36, com desenho de 30 a 32 (cerca de 80% do logo), centrado em y=44, em botões de 48 com 16 de vão (o vão de grupo da régua). As abas ficam em 22 com linha-base 52: o sublinhado já as destaca, e o tamanho menor as separa das pontas. Validação final dos espaços (medidos na tinta, com `scripts/measure-ink.py`): logo → marca era 14, virou 16 (marca em x=118); marca → abas, 48 (abas em x=236); abas com 32 entre os textos (36 estava fora da série). O relógio tinha 80 reservados, mas a hora mais larga mede 90 em 24 h (132 em 12 h), então invadia o vão e encostava na engrenagem. Testamos algarismos tabulares (largura fixa, vão constante de 34), mas o autor preferiu o desenho da fonte padrão no relógio. Decisão inicial: algarismos da fonte padrão, com espaço reservado igual à hora mais larga; depois revista (ver 7). Na TV o relógio parecia menor que a marca com a mesma altura de tinta (31 e 32 px a 1,5x): o peso 700 tem traço mais fino que o 800 da marca; o relógio passou a 800. Uma fonte monoespaçada padronizada fica definida como token (abaixo) para quando um uso a pedir. Um teste confere todas as horas do dia nos dois formatos. Razão de UX: a barra lida de longe se ancora em duas pontas de mesmo peso, e o meio (abas) é o controle. `TABS_X` e `TAB_GAP` podem precisar de ajuste porque as abas ficam cerca de 10% mais largas.

### 5. Tokens no tema

Cada nível vira variável de `:root` (`--font-title`, `--font-brand`, `--font-primary`, `--font-body`, `--font-label`, `--font-caption`) e os componentes a usam; o Rust lê o componente (`.page-title`, `.page-subtitle`, `.card-heading`, `.about-heading`, `.about-body`, `.about-license` e a marca) em vez de números. Cabeçalho de cartão (caixa alta, 800, espaçamento 1,2) vira um componente único usado por `settings.rs` e `editor.rs`.

### 6. Régua de layout para todas as telas

Pedido do autor: espaços, alinhamentos e tamanhos que façam sentido em todas as telas, e uma especificação para criar telas novas certas de primeira. Unidade de 4 (meio passo da grade de 8 usual em interfaces de TV), espaços entre blocos em múltiplos de 8 e uma série fechada (4 a 64); colunas derivadas da largura de 1152 (6×172 com vão 24, 2×552 com vão 48); linhas fixas para os dois modelos de tela (principal sob a barra e secundária com cabeçalho); alturas de componente e de linha de texto múltiplas de 4. Razão de UX: com poucos valores repetidos, o olho lê o ritmo como intencional, e uma tela nova é composta escolhendo valores da régua em vez de inventá-los. Exceções só derivadas (tile 16:9, ampliação do dock), registradas na spec. As medidas atuais fora da régua (tarefa 2.5) mudam de 1 a 3 unidades, sem alterar a composição.

### 7. Ícones e relógio com o peso do texto das abas

O autor achou os ícones com cara de desenho animado e fora do tema. Medido na TV: o traço do Lucide no glifo de 36 era 3 (5 px a 1080p), mais grosso que o texto das abas (4 px). Comparamos seis coleções (Lucide, Tabler, Phosphor, Iconoir, Heroicons, Material Symbols) e testamos o Tabler na TV; o autor preferiu o desenho do Lucide, com o peso do texto. A fonte pronta do Lucide só tem traço 2, então o script passou a desenhar a fonte a partir dos SVGs do Lucide, com traço de 1,75 (pontas e cantos redondos como os do Lucide, contornos no mesmo sentido para a sobreposição não abrir buracos): no glifo de 36 o traço tem 3,9 px a 1080p, igual ao das abas, e o desenho mantém de 30 a 32 de altura. O relógio também segue o texto das abas: em 28, o peso 500 tem o traço que o 22 tem em 700 (4,2 px medidos, contra 4,0 de "Apps" e 4,6 de "Início"); com 700 ou 800 ele media 6 px, como a marca. Para o vão até o relógio ser sempre o de 34 entre os ícones, os botões passaram a acompanhar a largura da hora mostrada, em vez de reservar o espaço da hora mais larga (que deixava o vão variar de 34 a 68).

## Risks / Trade-offs

- [Texto maior estoura medidas] rótulo de tile, chip e painel sobem de 16 para 18 → medir por captura; ajustar enchimentos na constante do módulo e registrar na spec; chips quebram linha por regra existente.
- [Menos níveis tira nuance] 15/17 e 24 deixam de existir → aceitável: cada papel tem o nível mais próximo, e a hierarquia por peso e cor permanece.
- [Piso 16 afeta a Sobre] a lista de licenças (16 hoje) não muda; só o que estava em 14/15 sobe.

## Migration Plan

Sem migração no aparelho. Nenhuma. Esta change é independente da `versao-do-build`.
