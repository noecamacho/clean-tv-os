# Proposal

## Why

A tela Início ainda segue o mockup inicial: um carrossel grande no topo e uma linha de tiles 16:9. Queremos uma identidade visual mais limpa e mais próxima de launchers desktop (macOS e GNOME): papel de parede de aurora boreal, favoritos num dock com Magnification e uma barra superior com menos ruído visual. O carrossel dá pouco valor para o custo (rotação, slides, histórico de contexto, editor) e brigaria com o papel de parede.

## What Changes

- **BREAKING** Remover o carrossel: slides, rotação automática, indicadores, linha de contexto, painel "Apps no carrossel" e intervalo no editor, e a ordenação/ocultação de seções (sobra uma única seção).
- **BREAKING** A linha "Seus apps favoritos" vira um **dock**: barra translúcida arredondada, flutuante, centralizada embaixo, só com os favoritos, sem título e sem rolagem horizontal.
  - Cada favorito é um **ícone quadrado arredondado** (a plataforma passa a fornecer o ícone do app, não o banner 16:9).
  - **Magnification**: o item focado cresce (âncora na base do dock), os vizinhos crescem menos, em fatores fixos definidos no código, com animação. Sem contorno no dock.
  - O nome do app focado aparece num balão acima do ícone.
- Adicionar um **papel de parede de aurora boreal nas cores da marca** (azul de destaque sobre o petróleo), único e não configurável nesta versão, em todas as telas. A base suave é uma textura de 640x360 gerada pelo próprio projeto (sem imagem de terceiros, por `tools/wallpaper/aurora06.py`), ampliada com filtro linear. As estrelas são desenhadas em vetor na resolução de renderização, para o papel de parede ficar tão nítido quanto a interface em telas 4K (renderizadas a 1080p). A própria imagem já é escura no topo e embaixo. O carregamento fica atrás de um ponto único, para uma spec futura permitir a troca.
- Grade da aba Apps: de 5 para 6 colunas, com tiles cerca de 18% menores (de 211x119 para cerca de 172x97), ocupando toda a largura sem sobra.
- Barra superior: nenhuma aba tem contorno ao redor do nome, nem em foco. O sublinhado fica sob a aba ativa e é azul enquanto o foco está na barra superior; quando o foco desce ao conteúdo (dock ou grade), ele fica branco, mostrando que as abas não estão sob controle. Com o foco na barra, acompanha a aba em foco, e a tela também troca assim que a aba recebe o foco, sem precisar de OK. A aba sem sublinhado fica em cinza claro, como hoje.
- Botão de engrenagem: o contorno de foco passa de branco para o azul de destaque (`--accent`), alinhado ao sublinhado das abas.
- Cores por papel: o tema passa a ter `--primary` (azul `#4DB3FF`, foco e controle), `--on-primary` e `--secondary` (branco `#F2F5F7`, neutro e texto); `--accent`/`--on-accent` viram apelidos. O contorno de foco dos tiles, itens de Configurações, linhas e chips passa de branco a azul primário.
- Retorno de um app: ao abrir um app (pelo dock, pela grade de Apps ou pelo item de Configurações do Android) e voltar ao lume pelo botão Home, o lume volta à tela, ao foco e à rolagem em que estava (por exemplo, a aba Apps com o foco no ícone do YouTube), em vez de ir ao início.
- Configuração: as chaves `sections`, `section.*` e `carousel.*` do `config.v1` deixam de existir. Arquivos antigos continuam legíveis, e o lume ignora o que sobrou.

## Capabilities

### New Capabilities
- `wallpaper`: papel de parede de aurora boreal nas cores da marca, único, nítido em 720p, 1080p e 4K, desenhado atrás de todas as telas, com ponto único de carregamento preparado para uma troca futura.

### Modified Capabilities
- `home-screen`: sai o carrossel; os favoritos passam a ser um dock com ícones quadrados, Magnification e balão com o nome.
- `home-editor`: saem a organização de seções e a configuração do carrossel; "Restaurar padrão" e a persistência passam a tratar só os favoritos.
- `launcher-shell`: ao voltar de um app aberto pelo lume, o Home devolve a tela e o foco de antes; Voltar na raiz leva o foco ao primeiro item do dock; a aba em foco não tem contorno; a engrenagem tem contorno azul no foco.
- `apps-screen`: grade de 6 colunas com tiles cerca de 18% menores.
- `app-catalog`: desinstalar um favorito não menciona mais o carrossel.
- `theming`: o tema padrão inclui o papel de parede e o dock; saem os seletores do carrossel.
- `resource-efficiency`: saem os prazos e requisitos do carrossel, e a textura do papel de parede entra nos orçamentos de memória.
- `localization`: o exemplo de tela Início em inglês não cita mais o carrossel.

## Impact

- Código: `crates/lume-core/src/app/{home,editor,topbar,apps,mod}.rs`, `home/{slides,config}.rs`, `usage.rs` (contexto dos slides), `app_icons.rs` e atlas, `i18n` e `lume-strings` (strings do carrossel), `theme.rs`, `themes/lume/theme.css`, `docs/theme-format.md`, `lume-css` (seletores).
- Android: `LumeBridge.java` passa a fornecer o ícone quadrado do app em vez do banner.
- Novo asset e script gerador do papel de parede, mais um novo ponto de desenho no renderer.
- Persistência: leitura tolerante ao `config.v1` antigo, sem aumentar a versão.
- Limites a manter: 45 MB de PSS em primeiro plano, 20 MB em segundo plano, APK até 6 MB, 60 fps na navegação.
