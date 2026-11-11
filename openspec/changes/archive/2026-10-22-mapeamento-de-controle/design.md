# Design

## Context

O Lume já possui `lume_core::keys::KeyMap` persistido em `files/keys.v1` com suporte a múltiplos códigos Android (`i32`) por tecla lógica (`Up`, `Down`, `Left`, `Right`, `Ok`, `Back`, `Home`, `Menu`), deduplicação, resolução automática de conflitos e autocura em disco (ver `proposal.md - Why`). Contudo, falta:
1. Uma tela em `Configurações › Mapeamento de controle` seguindo a régua de 4 px do `design-system`.
2. Extensão do modelo de dados em `keys.rs` para representar não apenas teclas de navegação, mas também **Comandos do Lume** (`Menu`, `Inputs`), **Atalhos de Configurações** (20 telas: as de configuração do Android TV e três telas internas do Lume) e **Atalhos de Aplicativos** (pacotes do catálogo).


A primeira versão da tela (chips `×`, `+ Tecla` e botões "Restaurar" em cada linha, modal escuro sem escurecimento) foi revisada em 2026-10-22 após inspeção das capturas: só 5 dos 10 atalhos de configuração e 6 apps eram alcançáveis (o laço de chips fazia `break` quando a linha enchia), os chips truncavam, o foco era 2D com dezenas de alvos pequenos, e o modal destoava do menu de contexto e do diálogo do editor. As decisões 3 e 5 a 8 abaixo substituem esse desenho.

## Goals / Non-Goals

**Goals:**
- Garantir que o usuário jamais perca a navegação da TV bloqueando qualquer alteração nas teclas vitais (`Up`, `Down`, `Left`, `Right`, `Ok`, `Back`, `Home` e teclas de `Power`/`Sleep`/`Wake`).
- Oferecer uma interface 100% operável por D-Pad (foco só vertical na tela), alinhada à régua de 4 px, reaproveitando componentes existentes (`.settings-item`, `.editor-row`, `.panel`, `.card-heading`, `.menu`, `.menu-item`, `.menu-note`, `.button-secondary`) e registrando só o que falta (`.keycap`, `--menu-danger`).
- Permitir remover teclas individuais (inclusive as que vêm por padrão em `Menu`, caso estejam invertidas no controle do usuário), adicionar novas teclas via escuta em tempo real, fazer **reset pontual** por linha e **reset de fábrica** no final da tela, tudo pelo menu de contexto da linha.
- Tornar alcançáveis todos os atalhos de configuração (20) e todos os aplicativos do catálogo, sem truncar rótulos, com listas que rolam.

**Non-Goals:**
- Remapear teclas de navegação (D-Pad, OK, Voltar, Home) ou desligar (Power).
- Disparar atalhos com o Lume em segundo plano: um serviço de acessibilidade para isso foi cogitado e tirado (nem toda TV deixa ligá-lo, como a AIWA, que não tem a tela de Acessibilidade); os atalhos valem com o Lume na tela.
- Criar macros de múltiplas teclas simultâneas ou cliques longos nesta versão (cada tecla adicionada funciona por clique simples `KeyAction::Down`).
- Migrar o `confirm_dialog` do editor para o cartão claro (ver Pendências).

## Decisions

### 1. Modelo Unificado em `lume_core::keys` e Formato de `files/keys.v1`

**Decisão:**
Expandir `KeyMap` mantendo 100% de retrocompatibilidade com arquivos `keys.v1` existentes:
- **Teclas Protegidas (imutáveis pela UI e sempre restauradas para `DEFAULT_*`)**:
  - `Up` (19), `Down` (20), `Left` (21), `Right` (22)
  - `Ok` (23, 66, 160, 96)
  - `Back` (4, 111, 97)
  - `Home` (3)
  - Códigos de hardware de energia reservados: `KEYCODE_POWER` (26), `KEYCODE_SLEEP` (223), `KEYCODE_WAKEUP` (224), `KEYCODE_TV_POWER` (177), `KEYCODE_SOFT_SLEEP` (276).
- **Alvos Mapeáveis (`KeyTarget`)**:
  1. **Comandos do Lume (`CommandTarget`)**:
     - `Menu` (chave `menu` em `keys.v1`, padrão `DEFAULT_MENU = [82, 174]`)
     - `Inputs` (chave `inputs` em `keys.v1`, padrão `DEFAULT_INPUTS = [178]`, correspondente a `KEYCODE_TV_INPUT`)
  2. **Atalhos de Configurações (`SettingShortcut`)**:
     Persistidos em `keys.v1` com prefixo `cfg.<id>=<codigos>`:
     - TV: `cfg.sleep` (Timer / Sleep), `cfg.picture` (Imagem), `cfg.sound` (Som), `cfg.wifi` (Rede e Wi-Fi), `cfg.bluetooth` (Bluetooth), `cfg.apps` (Aplicativos)
     - Sistema: `cfg.storage` (Armazenamento), `cfg.language` (Idioma), `cfg.datetime` (Data e hora), `cfg.keyboard` (Teclado), `cfg.captions` (Legendas), `cfg.location` (Localização), `cfg.security` (Segurança e restrições), `cfg.developer` (Opções do desenvolvedor), `cfg.home_app` (App de início padrão), `cfg.about` (Sobre a TV), `cfg.android` (Configurações gerais do Android)
     - Lume: `cfg.lume_settings` (Configurações do Lume), `cfg.lume_editor` (Editar Tela Início), `cfg.lume_tvinfo` (Informações da TV)
  3. **Atalhos de Aplicativos (`AppShortcut`)**:
     Persistidos em `keys.v1` com prefixo `app.<package>=<codigos>` (ex.: `app.com.google.android.youtube.tv=183,166`).

**Por quê:**
Usar prefixos `cfg.` e `app.` no próprio `files/keys.v1` evita criar um segundo arquivo de armazenamento, aproveita a escrita atômica e a autocura de `storage::save`, e mantém um único arquivo de configuração.

### 2. Nomes Amigáveis de Teclas (`keycode_label`)

**Decisão:**
Mapear os códigos `KEYCODE_*` mais comuns de controles de TV e universais para nomes traduzidos (`StringId` + código numérico, ex.: `Menu (82)`, `Favorito (174)`, `Entrada (178)`, `Guia (172)`, `Info (165)`, `Legenda (175)`, `TV (170)`, `Vermelho (183)`, `Verde (184)`, `Amarelo (185)`, `Azul (186)`, `Canal + (166)`, `Canal - (167)`, `Volume + (24)`, `Volume - (25)`, `Mudo (164)`, `Play/Pause (85)`, `Parar (86)`, `Avançar (90)`, `Retroceder (89)`, `0`–`9` (7–16), `F1`–`F12`), com fallback para `Tecla {código}` (`Key {code}`) para códigos proprietários. No `.keycap` o nome e o código são desenhados separados (nome em `--text`, código em `--text-muted`), sem os parênteses.

### 3. Fluxo de Captura de Tecla no Shell / App

**Decisão:**
Quando a tela `Screen::KeyMap` está com o modal de captura aberto (`listening: Some(KeyTarget)`), `LumeApp` informa que está capturando tecla bruta (`LumeApp::is_listening_key(&self) -> bool`).
- No Android (`Runtime::process_input`) e no desktop (`lume-desktop`), todo `KeyAction::Down` entrega o código físico bruto `code: i32` para `Shell::raw_key(code, now_ms)` antes da tradução normal quando `is_listening_key()` é verdadeiro (ou `Shell::raw_key` unifica a entrada).
- O modal **não tem botão nem alvo de foco**: enquanto ele está aberto, a árvore de foco da tela fica vazia (`update_focus` SHALL aceitar uma lista vazia; `LISTENING_CANCEL` deixa de existir). Voltar é o cancelar.
- Regras dentro do modal de captura:
  1. Se `code` pertence a `Back` (`4, 111, 97`), cancela o modal sem modificar nada. É a única forma de cancelar.
  2. Se `code` é uma tecla reservada (`Up`, `Down`, `Left`, `Right`, **`Ok` (`23, 66, 160, 96`)**, `Home`, `Power`/`Sleep`/`Wake`), mantém o modal aberto e troca a dica "Voltar para cancelar" pelo aviso de tecla reservada na mesma faixa do cartão. A tecla nunca é gravada.
  3. Caso contrário, adiciona `code` ao `KeyTarget` (removendo-o de qualquer outro alvo se já estava em uso), fecha o modal, emite `Effect::SaveKeys` e mostra um toast confirmando a atribuição.

**Por quê:** a regra anterior (Voltar **ou OK** cancelam) deixava o OK, a tecla que abriu o modal, fechá-lo sem querer. Com Voltar como único cancelar e OK reservado, o modal não precisa de botão, e a regra "tecla reservada nunca é gravada" vale sem exceção.

### 4. Intents de Configuração (`LumeBridge`)

**Decisão:**
- Comando de ponte `"open-setting"` (com o nome do atalho, o mesmo de `keys.v1` sem o `cfg.`), implementado em `LumeBridge.java` como uma **cascata**: cada atalho tem uma lista de `Intent`s (a ação padrão do Android, a tela do app de configurações da TV por nome de classe, ações específicas de fabricante) e abre a primeira que leva a uma tela de verdade; termina em `Settings.ACTION_SETTINGS`, que toda TV tem. Uma tentativa é pulada quando só um stub do sistema ("Nenhum aplicativo pode executar esta ação", pacote `com.google.android.tv.frameworkpackagestubs`) ou o seletor "Abrir com" (pacote `android`) a responde.
- O Android 11 esconde as telas de configurações do `resolveActivity` de apps sem visibilidade, então o manifest declara em `<queries>` os pacotes `com.android.tv.settings`, `com.android.settings` e `com.google.android.permissioncontroller` (App de início padrão) e o dos stubs. Sem isso todas as tentativas falhavam e todo atalho caía nas Configurações gerais.
- As telas conferidas na AIWA (KTC/MediaTek, Android 11) estão em `docs/device-notes.md`; numa TV com firmware diferente a cascata cai nas Configurações gerais para o que ela não tiver.

### 5. Layout da tela "Mapeamento de controle" na Régua de 4 px (linhas e keycaps)

A tela é uma tela secundária (`LEFT = 64`, `WIDTH = 1152`) que reaproveita o padrão de Configurações e do editor: três painéis de linhas. O foco na tela é só vertical (cima/baixo); OK numa linha abre o menu de contexto (decisão 6).

Tabela de métricas no canvas lógico `1280x720` (`N` = atalhos de configuração mapeados, `M` = apps mapeados):

| Item | Caixa (x, y, l, a) | Regra |
|---|---|---|
| Caminho (`breadcrumb`) | `x=64`, linha-base `52` | `"Configurações › Mapeamento de controle"` |
| Título (`.page-title`) | `x=64`, linha-base `116` | `"Mapeamento de controle"`, 40 px peso 800 |
| Subtítulo (`.page-subtitle`) | `x=64`, linha-base `152` | Instrução curta ("Personalize botões do controle para comandos do Lume, ajustes da TV e apps."), 18 px peso 500; sem a lista de teclas protegidas |
| Painel 1: Comandos do Lume (`.panel`) | `x=64, y=212, w=1152, h=216`, enchimento `32` H / `24` V | No início do conteúdo (58 abaixo da linha-base do subtítulo); 2 linhas |
| Cabeçalho (`.card-heading`, cor `--primary`) | `x=96`, `y=painel+12`, `h=32` (centrado entre o topo do painel e a 1ª linha, a 56) | Caixa alta, como "APARÊNCIA" e "FAVORITOS" do editor |
| Linha de mapeamento (`.editor-row`) | `x=96, w=1088, h=64`, passo `72`, a 1ª em `painel+56` | Fundo `--surface-row`, enchimento `24`; rótulo 22/700 em `--text` à esquerda; `.keycap`s alinhados à direita, o último terminando em `x=1160` |
| Keycap (`.keycap`) | `h=40`, raio `20`, enchimento `16`, vão `16` entre keycaps, centrado na linha (12 acima e abaixo) | Nome 18/700 `--text` + código 16/500 `--text-muted`, vão `8` entre os dois; largura **medida** com `p.measure` (nunca estimada); não focável |
| Linha "Adicionar atalho/aplicativo" | Mesma caixa da linha (`h=64`), contorno `2` `--border`, sem fundo | Glifo "+" de 24, vão `8`, texto 18/700; em foco, fundo `--surface-focus` e o anel azul de `.editor-row:focus`; omitida quando não resta nada para adicionar |
| Painel 2: Atalhos de configurações | `x=64, y=452, w=1152, h=144+72·N` | 24 abaixo do painel 1; `N` linhas mapeadas + a linha "Adicionar atalho" (sem ela, `h=72+72·N`) |
| Painel 3: Atalhos de aplicativos | `x=64`, 24 abaixo do painel 2, `w=1152, h=144+72·M` | Idem, com "Adicionar aplicativo" |
| Botão "Restaurar padrão de fábrica" | `x=64`, 24 abaixo do painel 3, `h=48` | `.button-secondary`, abre o diálogo de confirmação |

Sem atalhos mapeados (`N=M=0`) o conteúdo termina em y=836 e a tela rola; a rolagem leva a linha em foco para dentro da área segura (a conferir no emulador, ver tarefas).

### 6. Menu de contexto da linha

**Decisão:** OK numa linha de mapeamento abre o menu de contexto (`.menu`, `launcher-shell`/`design-system`), em vez de chips e botões dentro da linha. Itens, nesta ordem:
- **"Adicionar tecla"**: abre o modal de escuta para o alvo da linha.
- **"Remover <tecla>"**, um item por tecla da linha (ex.: "Remover Favorito 174"): remove só aquela tecla e grava `files/keys.v1`.
- **"Restaurar padrão"** (só em "Comandos do Lume"): volta o alvo ao `DEFAULT_*`. Quando já é igual ao padrão, o item aparece desabilitado com a nota "Já está no padrão" (`.menu-note`).
- **"Remover atalho"** (só em linhas de atalho `cfg.*` e `app.*`): limpa todas as teclas e a linha deixa a lista (o alvo volta ao seletor).

Uma linha **sem teclas** pula o menu: OK vai direto à escuta. O menu abre abaixo da linha, alinhado à borda direita dela (`x=1184`), 8 abaixo; se não couber na área segura (`y ≤ 702`), abre acima, 8 acima da linha; foco inicial no primeiro item.

**Por quê:** o design system manda usar o menu de contexto para a lista de ações sobre um item, e isso derruba o foco 2D (chips `×` de 8 px) para uma lista vertical de alvos grandes.

### 7. Seletores de "Adicionar atalho" e "Adicionar aplicativo"

**Decisão:** os dois são telas secundárias com rolagem, para caberem todos os itens (20 atalhos, qualquer número de apps), com o mesmo desenho do restante do app:
- **Atalho de configuração:** OK na linha "Adicionar atalho" abre a tela `Screen::KeyMapPickSetting` ("Configurações › Mapeamento de controle › Escolher atalho", título "Escolher atalho", subtítulo "Escolha a tela que a tecla vai abrir"): uma lista de linhas `.editor-row` de 64 (passo 72, `x=64, w=1152`, a partir de `y=212`), só com os atalhos ainda sem tecla, na ordem do catálogo (TV, sistema, Lume); o foco é vertical e a lista rola para manter a linha em foco visível (como o editor). OK escolhe, volta à tela de mapeamento e abre o modal de escuta; o atalho só passa a existir (e a linha só aparece) quando uma tecla é gravada; cancelar a escuta ou apertar Voltar na lista não cria nada e leva o foco à linha "Adicionar atalho".
- **Aplicativo:** OK na linha "Adicionar aplicativo" abre a tela `Screen::KeyMapPickApp` ("Configurações › Mapeamento de controle › Escolher aplicativo", título "Escolher aplicativo", subtítulo "Escolha o app que a tecla vai abrir"), uma tela secundária com a grade de tiles da aba Apps: 6 colunas de 172, vão 24, tiles 16:9, ordem alfabética pelo nome mostrado, conteúdo a partir de `y=212`, rolagem como na aba Apps, só com apps sem atalho. OK escolhe o app, volta à tela de mapeamento e abre o modal de escuta; Voltar retorna à tela de mapeamento com o foco na linha "Adicionar aplicativo".

**Por quê:** a lista de atalhos (primeiro 10, depois 20) não cabe num menu de contexto: com 56 por item passa dos 684 da área segura. Uma tela com rolagem não tem limite e reaproveita a rolagem que o editor e a grade de Apps já têm; o menu de contexto volta a ser só para as ações de uma linha.

### 8. Cartões claros de sobreposição: modal de escuta e diálogo de confirmação

**Decisão:** o modal de escuta de tecla e o diálogo "Restaurar o padrão de fábrica?" desta tela usam o cartão claro do menu de contexto (`--menu-surface`, raio 20, sombra `0 4 24` preto a 45%), **sem camada escura de tela cheia** (armadilha já resolvida: custou 54% dos quadros na TV) e sem animação contínua.

Modal de escuta (sem botão e sem foco; cancelar é o Voltar, decisão 3):

| Item | Caixa (dentro do cartão) | Regra |
|---|---|---|
| Cartão | `x=360, y=264, w=560, h=188` | Centrado; enchimento `32` |
| Título | `y 32 a 76` | "Pressione uma tecla", 28/800, `--menu-text` |
| Subtítulo | `y 76 a 116` | "Para: <alvo>", 18/500, `--menu-text-muted` |
| Faixa de dica/aviso | `y 116 a 156` | Dica "Voltar para cancelar" (18/500, `--menu-text-muted`); ao pressionar tecla reservada, aviso "Tecla reservada. Escolha outra, ou Voltar para cancelar." (18/700, `--menu-danger`); a altura do cartão não muda |
| Enchimento inferior | `32` | total `188` |

Diálogo de confirmação (foco inicial em "Cancelar"):

| Item | Caixa (dentro do cartão) | Regra |
|---|---|---|
| Cartão | `x=360, y=232, w=560, h=260` | Centrado; enchimento `32` |
| Título | `y 32 a 76` | "Restaurar o padrão de fábrica?", 28/800, `--menu-text` |
| Corpo | `y 76 a 156` | Até 2 linhas de 40, 18/500, `--menu-text-muted` |
| Botões | `h=48`, `y 180 a 228`, vão `16`, alinhados à direita (borda interna a `x=888`) | Largura = texto + 56, mínimo 140; "Cancelar" com contorno `2` `--menu-text-muted` e texto `--menu-text`; "Restaurar" com contorno `2` e texto `--menu-danger`; **em foco**, fundo `--primary` e texto `--on-primary`, sem anel (como o `.menu-item`) |

**Título do diálogo:** "Restaurar o padrão de fábrica?" (e equivalentes curtos nos outros idiomas): o texto "Restaurar o mapeamento de fábrica?" tem uns 510 de largura a 28/800 e a linha útil do cartão tem 496, então saía cortado com reticências. A linha única do título não muda de altura.

**Token novo:** `--menu-danger` `#B3261E` (perigo sobre o cartão claro; contraste de 5,97:1 sobre `#F2F5F7`, a conferir no teste de contraste do tema). O `--danger-fg` (`#FFB4BE`) continua só para superfícies escuras.

## Risks / Trade-offs

- **[Risk] Diferenças de tela de configuração entre marcas de TV** → **Mitigação**: cascata no `LumeBridge` (decisão 4), que pula stubs e seletores e termina nas Configurações gerais, com as telas da AIWA conferidas por `adb` e registradas em `docs/device-notes.md`; a TCL ainda deve ser conferida.
- **[Risk] Modal sem foco deixa o laço sem alvo** → **Mitigação**: `update_focus` aceita lista vazia, com teste; quando o modal fecha, o foco volta para a linha de origem (ou para a linha "Adicionar" se a escuta cancelada veio de um seletor).
- **[Risk] A rolagem da tela (cerca de 1,3 tela) não levar a linha em foco para a área visível** → **Mitigação**: validar no emulador `tv_api28` (o screenshot do `lume-desktop` não espera a animação de rolagem) e cobrir com teste de `scroll_y` seguindo o foco.
- **[Risk] Tela de escolha de aplicativo duplicar a grade de Apps** → **Mitigação**: reutilizar o desenho de tiles e a rolagem da aba Apps em modo "escolher", sem copiar a lógica de abertura.

## Pendências

- **Unificar os diálogos.** O `confirm_dialog` do editor (`editor.rs`) continua escuro e com camada escura de tela cheia; depois desta change o app passa a ter dois estilos de diálogo. Uma change futura migra o editor para o cartão claro (e remove a camada translúcida de tela cheia que a spec `design-system` já proíbe em outros lugares).
