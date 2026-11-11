# Tasks

## 1. Tema e spec

- [x] 1.1 Em `themes/lume/theme.css` e `crates/lume-css`: tokens `--font-title`, `--font-brand`, `--font-primary`, `--font-body`, `--font-label`, `--font-caption`; componentes novos (`.page-title`, `.page-subtitle`, `.card-heading`, `.about-heading`, `.about-body`, `.about-license`, marca) e todos os existentes na escala de 6 níveis; verificar com `cargo test -p lume-css -q`
- [x] 1.2 `MIN_FONT_SIZE` de 14 para 16 em `lume-css` e `lume-core/src/theme.rs`, aviso de build para tamanho fora da escala e testes do tema; verificar com `cargo test --workspace -q`

## 2. Telas leem o tema

- [x] 2.1 `app/page.rs`: título e subtítulo pelos componentes (sem 40.0 e 18.0 no código)
- [x] 2.2 `app/settings.rs` e `app/editor.rs`: cabeçalho de cartão por um componente só (sem 15.0), blocos da Sobre pelos componentes (sem 24.0, 20.0, 16.0); linhas do editor e títulos da Sobre em 22
- [x] 2.3 `app/topbar.rs`: marca pelo token, abas e relógio em 22 com linha-base 52, glifos de 24, retângulo de foco da aba ajustado e `TABS_X`/`TAB_GAP` reajustados; verificar `grep -rnE "size: *[0-9]+\.0|font_size = [0-9]" crates/lume-core/src` sem números de fonte soltos
- [x] 2.3.1 Barra revisada na TV pelo autor: relógio de 28 na linha-base da marca, glifos de 36 em botões de 48 com 16 de vão, marca a 16 do logo, abas em x=236 com 32 de vão, relógio na fonte padrão com espaço reservado pela hora mais larga do formato; métricas na tabela do requisito "Barra superior" e `scripts/measure-ink.py` para medir a tinta
- [x] 2.3.2 Ícones Lucide redesenhados pelo script com traço 1,75 (`LUCIDE_STROKE`, `--icons-only`), relógio em peso 500 e botões acompanhando a largura da hora; traços medidos iguais aos das abas
- [x] 2.4 Rótulo de tile, chip, painel e valores do painel de estatísticas em 18; ajustar enchimentos e medidas na constante do módulo se o texto maior apertar, e registrar na spec (conferido por captura: nada cortou nem apertou, sem ajuste)

- [x] 2.5 Régua de layout (requisito "Régua de layout"): alinhar as medidas fora da régua, cada uma na constante do módulo ou no tema: espaço do dock 14→16, enchimento do balão 8/18→8/16, `gap` dos botões 10→8, `.editor-row` 71→72, chip 42 (código) e 33 (tema)→40 com raio 20, subtítulo 154→152, Apps título 119→116 e grade 143→144 (passo da grade 148,5→148), Sobre 150/190→152/192 e alturas de linha 1,9×→32/40/44, separador do caminho 10→8; um teste confere que as constantes de layout são múltiplas de 4

## 3. Verificação de responsividade

- [x] 3.1 Capturas no desktop (Início, Apps, Configurações, Sobre, editor) em 1280x720, 1920x1080 e 3840x2160, conferindo a mesma composição em unidades lógicas, o eixo y=44 e a linha-base 52 da barra (nenhum item fora do eixo por mais de 1 px) e nenhum texto novo com reticências (feito: as 5 telas nas 3 resoluções; a barra mede o mesmo em unidades lógicas com 1 de diferença, ícones e logo centrados em 43,5 a 44, sem reticências novas, inclusive em inglês)
- [x] 3.2 Capturas em 1024x768 (4:3) e 2560x1080 (21:9): conteúdo centralizado, dentro da área segura (32/18), sobras com o fundo do tema (feito: conteúdo de 56,7 a 1182,7 em 21:9 e de 57,5 a 1182,5 em 4:3 no canvas, como em 16:9; sobras com o papel de parede)
- [x] 3.3 Instalar na TCL (1080p) e na AIWA (720p, `low_ram`) e conferir a olho e por captura; quadros perdidos e PSS dentro do orçamento da spec `resource-efficiency` (feito: instalado nas duas; quadros de 33 ms ou mais: 0 em 116 intervalos de animação na TCL e na AIWA; PSS na TCL: 25,0 MB em primeiro plano e 19,9 MB em segundo, com Graphics 0, dentro do orçamento e no limite do segundo plano; na AIWA o PSS não pôde ser medido porque o sistema da TV desligou o `dumpsys` com "Service dumps disabled due to hung system process")

## 4. Fechamento

- [x] 4.1 Spec `design-system` e `display-adaptation` coerentes com o tema (valores de `theme.css` iguais aos da spec) e `docs/theme-format.md` atualizado com os tokens novos (feito: valores do tema conferidos com a spec; `docs/theme-format.md` com os componentes novos, a escala de 6 tamanhos, os tokens e a régua)
- [x] 4.2 Revisar a tela Sobre (versão, endereço, licença, terceiros) e atualizá-la se houve novidade; senão registrar "Sobre revisado, sem mudança" (Sobre revisado, sem mudança: os ícones continuam do Lucide, ISC, só redesenhados com traço 1,75; versão e endereço iguais)
- [x] 4.3 `openspec validate escala-tipografica --strict` (feito)
