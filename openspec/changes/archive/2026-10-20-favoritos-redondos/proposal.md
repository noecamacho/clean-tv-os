# Proposal

## Why

No Android e no Google TV os ícones dos apps costumam ser circulares, enquanto o dock do Lume hoje desenha os favoritos sempre como quadrados de cantos arredondados. Além disso, a tela `Configurações › Tela Início` tem vãos verticais e horizontais largos que deixam uma sobra à direita do painel (cabendo só 5 cartões por linha) e empurram os botões inferiores para perto do fim da tela. Oferecer um botão de liga/desliga para **Favoritos redondos** na tela `Configurações › Tela Início`, junto com o enxugamento dos vãos do editor na régua de 4/8, permite escolher o formato dos ícones do dock mantendo toda a tela compacta e alinhada.

## What Changes

- **Opção "Favoritos redondos" (`Configurações › Tela Início`):**
  - Uma linha `.editor-row` no topo do conteúdo do editor, acima do painel de favoritos, com o rótulo "Favoritos redondos" ("Round favorites" em inglês) à esquerda e uma chave Liga/Desliga à direita.
  - O foco inicial ao abrir `Configurações › Tela Início` começa na linha "Favoritos redondos"; pressionar OK alterna entre ligado e desligado e grava imediatamente em `config.v1` (chave `round_favorites`).
  - O padrão de primeiro uso é **desligado** (quadrado de cantos arredondados, como hoje), e "Restaurar padrão" também devolve essa opção para desligado.
- **Formato dos ícones no dock (`Tela Início`):**
  - Desligado (padrão): cada item do dock aparece como quadrado de cantos arredondados (`raio = 0.22 * lado`).
  - Ligado: cada item do dock (fundo, imagem do ícone e fantasma de remoção) aparece como círculo (`raio = 0.5 * lado`), mudando na hora sem precisar regerar o cache em disco dos ícones.
- **Vãos enxutos em `Configurações › Tela Início` (na régua de 4/8):**
  - O conteúdo do editor começa em `y = 184` (`32 px` abaixo da linha-base do subtítulo em `152`), a linha "Favoritos redondos" tem `64 px` de altura e fica a `16 px` do painel de favoritos.
  - Dentro do painel de favoritos: enchimento vertical de `24 px` (mantendo `32 px` nas laterais), `32 px` entre a linha do cabeçalho e os cartões, vão entre cartões reduzido de `24 px` para `16 px` (cabendo **6 favoritos por linha** nos `1088 px` úteis, sem sobra à direita: `6 * 168 + 5 * 16 = 1088`), e `20 px` entre os cartões e a seção "Adicionar app:".
  - O vão entre o painel de favoritos e a linha de botões ("Restaurar padrão" e "Concluído") passa de `40 px` para `24 px`.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `home-editor`: adiciona a opção "Favoritos redondos" no topo da tela `Configurações › Tela Início` (foco inicial nela, persistida em `config.v1`, restaurada por "Restaurar padrão") e atualiza o layout para os vãos enxutos com 6 cartões por linha.
- `home-screen`: os itens do dock respeitam a preferência "Favoritos redondos" (quadrado de cantos arredondados por padrão ou circular quando ligado).
- `app-catalog`: o requisito "Ícone do app no dock" acompanha o formato configurado (quadrado arredondado ou circular) sem depender de reprocessar o arquivo de cache quando a opção muda.
- `design-system`: atualiza as medidas do editor em "Canvas, margens e grade" e especifica a chave Liga/Desliga em `.editor-row`.

## Impact

- Rust: `home/config.rs` (`round_favorites: bool` em `HomeConfig`, serializado em `config.v1` com fallback `false` para arquivos existentes), `app/editor.rs` (linha "Favoritos redondos", foco inicial e constantes de layout enxutas), `app/dock.rs` (raio `0.22` ou `0.5` para fundo, imagem e ghost), `render/list.rs` / `render/batch.rs` / `render/shaders.rs` (recorte arredondado/circular da imagem do dock na GPU ou máscara de raio), testes em `home/config.rs`, `app/editor.rs`, `app/dock.rs` e `app/tests.rs`.
- Java: `LumeBridge.java` (`renderDockIcon` entrega o quadrado completo do ícone no cache, deixando o arredondamento/círculo para a GPU, ou aplica o raio sem invalidar o cache).
- Tema e traduções: `themes/lume/theme.css` (se necessário ajustar `.editor-row`), `android/app/src/main/res/values/strings.xml` e `values-pt-rBR/strings.xml` (`editor_round_favorites`).
