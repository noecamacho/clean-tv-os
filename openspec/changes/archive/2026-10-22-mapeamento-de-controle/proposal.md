# Proposal

## Why

Controles remotos de diferentes fabricantes de Android TV (como TCL e AIWA/KTC) e controles universais enviam códigos distintos ou invertidos para botões como Menu, Favorito, Entrada, Guia, botões coloridos e Canal/Volume, além de possuírem teclas extras subutilizadas. Hoje o Lume já persiste `files/keys.v1`, mas não oferece uma interface visual para que o usuário personalize ou remova teclas problemáticas, crie atalhos rápidos para menus específicos de configuração da TV (como o Sleep Timer / Desligamento automático, Imagem, Som, Wi-Fi) ou acione aplicativos diretamente.

## What Changes

- **Novo item "Mapeamento de controle" em Configurações do Lume**: Adicionado à coluna esquerda da tela Configurações (entre "Tela Início" e "Informações da TV"), abrindo a tela secundária `Configurações › Mapeamento de controle`.
- **Proteção estrita das teclas vitais de navegação e sistema**: As teclas Direcionais (`Up`, `Down`, `Left`, `Right`), Confirmar (`Ok`/`Enter`), Voltar (`Back`/`Escape`), Início (`Home`) e Energia (`Power`/`Sleep`/`Wake`) são bloqueadas contra modificação ou remoção, impedindo que o usuário perca o controle da TV.
- **Três blocos de mapeamento na interface**:
  1. **Comandos do Lume**: Permite adicionar ou remover individualmente teclas para ações como *Menu de contexto* e *Entradas da TV*, com opção de **reset pontual** por ação quando modificada em relação ao padrão da compilação.
  2. **Atalhos de Configurações**: Permite criar atalhos de uma tecla para abrir menus específicos de configuração da TV (20 no total: *Timer / Sleep*, *Imagem*, *Som*, *Rede e Wi-Fi*, *Bluetooth*, *Aplicativos*, *Armazenamento*, *Idioma*, *Data e hora*, *Teclado*, *Legendas*, *Localização*, *Segurança*, *Opções do desenvolvedor*, *App de início padrão*, *Sobre a TV* e *Configurações do Android*) e telas internas do Lume (*Configurações do Lume*, *Editar Tela Início*, *Informações da TV*). O atalho é escolhido numa tela própria, "Escolher atalho", uma lista com rolagem; cada atalho tem a cascata de telas das TVs (conferida na AIWA) e termina nas Configurações gerais. Há remoção individual de teclas e reset/remoção pontual do atalho.
  3. **Atalhos de Aplicativos**: Permite vincular uma ou mais teclas do controle remoto para abrir diretamente qualquer aplicativo instalado no catálogo, com remoção individual de teclas e remoção pontual do atalho.
- **Modal amigável de captura de tecla**: Ao adicionar uma tecla, uma caixa de diálogo escuta o controle remoto, exibe o nome amigável e o código da tecla, rejeita teclas reservadas com mensagem explicativa (só `Voltar` cancela a captura; `OK` é reservado) e resolve conflitos automaticamente.
- **Atalhos só com o Lume em primeiro plano**: o serviço de acessibilidade para atalhos em 2º plano (`LumeKeyService`) foi tirado da change: nem toda TV deixa ligá-lo (a AIWA não tem a tela de Acessibilidade), então os atalhos valem enquanto o Lume está na tela.
- **Restauração do mapeamento de fábrica**: Botão no rodapé da tela com diálogo de confirmação que restaura todas as teclas ao padrão da compilação e limpa os atalhos customizados.

## Capabilities

### New Capabilities
- `remote-mapping`: Gerenciamento visual e persistente de mapeamento de teclas do controle remoto, regras de teclas protegidas, captura interativa de teclas, atalhos para configurações específicas da TV/Lume, atalhos para aplicativos, restauração pontual/total.

### Modified Capabilities
- `launcher-shell`: Inclusão do item "Mapeamento de controle" na tela de Configurações do Lume, expansão de `files/keys.v1` para suportar atalhos de configurações e de aplicativos mantendo retrocompatibilidade, e disparo desses atalhos pelo controle remoto.
- `design-system`: Registro das métricas de layout na régua de 4 px para a tela "Mapeamento de controle", linhas de mapeamento com chips de teclas removíveis, botão de reset pontual, modal de escuta de tecla e quarto item na lista de Configurações.

## Impact

- **`crates/lume-core`**: Expansão de `keys.rs` (nomes amigáveis de `KEYCODE_*`, classificação de teclas reservadas, mapeamento de ações de configuração e de pacotes de apps em `keys.v1`), nova tela `app/keymap.rs` (ou módulo dedicado em `app/`), atualização de `app/settings.rs` e `shell/mod.rs`.
- **`crates/lume-desktop`**: Suporte a `--screenshot keymap` e simulação de teclas na nova tela para inspeção visual e medição na régua.
- **`android/`**: `LumeBridge.java` abre as telas de configuração das TVs por cascata (cada atalho tem as suas), e o `AndroidManifest.xml` declara em `<queries>` os pacotes de configurações, sem o que o Android 11 esconde essas telas do `resolveActivity`.
- **`res/values/strings.xml` e `res/values-pt-rBR/strings.xml`**: Textos em inglês e português do Brasil para a nova tela, nomes de teclas e diálogos.
