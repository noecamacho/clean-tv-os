# Proposal

## Why

Ao abrir ou voltar ao Lume, a TV mostra primeiro uma tela lisa (a janela inicial do sistema, cerca de 400 ms na TCL), depois a aurora entra de uma vez e os ícones aparecem aos poucos, porque são liberados em segundo plano e recarregados a cada volta. O resultado é uma entrada brusca e uma interface que se completa à vista do usuário. O foco desta change é a experiência do usuário: a abertura e a troca entre o Lume e os outros apps precisam ser suaves e, ao mesmo tempo, breves. Terceira das três changes do release v0.3.0.

## What Changes

- O splash é todo do Lume, em Rust: a janela inicial do sistema continua só com a cor `--bg`, e o primeiro quadro do Lume mostra o logo centralizado sobre a mesma cor, com um fade curto de entrada (120 ms). O Lume segura essa tela até os ícones visíveis da tela de destino estarem carregados (teto de 600 ms).
- Em seguida a interface é revelada: a aurora surge do fundo, o logo some e o conteúdo (dock ou grade) entra com um deslocamento curto: cerca de 300 ms na abertura a frio e 220 ms no retorno a quente.
- Sem tempo mínimo: se tudo estiver pronto, o splash dura só o necessário.
- Qualquer tecla durante o splash encerra o splash na hora e é aplicada à interface.
- Vale para a abertura a frio e para o retorno a quente, inclusive quando o retorno restaura outra tela (ex.: a grade de Apps rolada).
- Sem versão no splash, por enquanto.
- Sem barra ou indicador de carregamento nesta versão, porque o splash é breve. O splash já fica preparado para isso: ele sabe quanto do carregamento terminou e reserva o espaço abaixo do logo, para que uma orientação ao usuário possa ser acrescentada depois sem refazer o splash.
- Ajuste visual pedido no caminho: o texto dos itens do menu de contexto passa de 20 com peso 700 para 18 com peso 500, o mesmo dos nomes dos apps, e as laterais dos itens passam de 32 para 24 da borda do cartão (enchimento de 24 para 16, largura mínima de 240 para 200); a largura do menu acompanha o texto medido. Na grade, o menu fica a 8 do anel de foco do tile, com o topo alinhado ao do anel (antes ficava a 24 do tile em repouso e alguns pixels abaixo), e o cartão ganha uma sombra próxima e em volta, que o separa de um tile branco; no dock, a 4 da caixa do nome do favorito (era 16).
- Sem camada translúcida de tela cheia: a aurora é revelada pela cor com que o papel de parede já é desenhado, e só a área do logo fica translúcida.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `launcher-shell`: novo requisito do splash na abertura e no retorno.
- `design-system`: componente splash (logo, posição, revelação e durações).
- `resource-efficiency`: o retorno rápido passa a distinguir o primeiro quadro (o splash) da interface pronta, com teto para a revelação.

## Impact

- Android: nada muda na camada Java nem nos recursos (a janela inicial continua com `windowBackground` liso).
- Rust: `app/mod.rs` e `shell/mod.rs` (fase de splash, espera pelos ícones visíveis, teclas durante o splash), `app/painter.rs` (intensidade do papel de parede animável), `lume-app/src/android/mod.rs` (log de "interface pronta"); testes em `app/tests.rs`.
- Tema: `themes/lume/theme.css` (tamanho do logo e durações do splash).
- Medição: novo log de "interface pronta" e medidas em `docs/device-notes.md` nas duas TVs.
- Fora do escopo: manter os ícones do dock em segundo plano (pode vir depois, reduzindo a espera no retorno a quente).
