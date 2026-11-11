# Spec Delta

## MODIFIED Requirements

### Requirement: Tela de Configurações do Lume
A tela "Configurações" do Lume SHALL ser aberta pelo tile "Lume" da grade de Apps, com os itens "Tela Início", "Mapeamento de controle", "Informações da TV" e "Sobre o Lume", sem item do Android e com o painel de estatísticas à direita (`device-stats`), caminho "Configurações" no topo e versão instalada (`platform-compatibility`) no canto inferior direito. Voltar SHALL retraçar o caminho de entrada passo a passo até o tile "Lume".

#### Scenario: Abrir pelo tile
- **WHEN** o foco está no tile "Lume" da grade de Apps e o usuário pressiona OK
- **THEN** a tela "Configurações" do Lume é exibida com o foco no primeiro item

#### Scenario: Painel de estatísticas visível
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** o painel de estatísticas aparece à direita dos itens, sem receber foco

#### Scenario: Caminho e versão
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** o caminho "Configurações" aparece no topo, e a versão instalada no canto inferior direito

#### Scenario: Versão de release com "v"
- **WHEN** o build instalado veio da tag `v0.3.0` e a tela "Configurações" ou a tela Sobre é exibida
- **THEN** a versão aparece como "v0.3.0", com um "v" só

#### Scenario: Versão de build local
- **WHEN** o build instalado foi gerado localmente no commit `4d8438b` com mudanças não commitadas
- **THEN** a versão aparece como "4d8438b-dirty", sem "v"

#### Scenario: Sem item do Android
- **WHEN** a tela "Configurações" do Lume é exibida
- **THEN** ela lista apenas "Tela Início", "Mapeamento de controle", "Informações da TV" e "Sobre o Lume"

#### Scenario: Voltar ao tile
- **WHEN** o usuário pressiona Voltar na tela "Configurações" do Lume
- **THEN** a aba Apps é exibida com o foco no tile "Lume"

#### Scenario: Ir para a edição da tela Início
- **WHEN** o usuário seleciona "Tela Início" em Configurações
- **THEN** o modo edição é aberto com o caminho "Configurações › Tela Início" e o título "Tela Início"

#### Scenario: Ir para o Mapeamento de controle
- **WHEN** o usuário seleciona "Mapeamento de controle" em Configurações
- **THEN** a tela de mapeamento é aberta com o caminho "Configurações › Mapeamento de controle" e o título "Mapeamento de controle"

#### Scenario: Voltar passo a passo
- **WHEN** o usuário está no modo edição ou no mapeamento de controle aberto pelas Configurações e pressiona Voltar duas vezes
- **THEN** a primeira volta à tela "Configurações" com o foco no item de origem ("Tela Início" ou "Mapeamento de controle"), e a segunda à aba Apps com o foco no tile "Lume"

#### Scenario: Tela Sobre
- **WHEN** o usuário escolhe "Sobre o Lume"
- **THEN** é exibida uma tela com a versão do build (a mesma do canto da tela "Configurações"), a licença (GPL-3.0-or-later), o endereço do código-fonte `https://github.com/douglaspands/lume-tv-launcher` e as licenças de terceiros (fonte, ícones, papéis de parede e bibliotecas)

### Requirement: Mapeamento centralizado e persistente de teclas
O Lume SHALL centralizar as teclas lógicas, constantes `KEYCODE_*`, padrões `DEFAULT_*` e atalhos (`cfg.<alvo>` e `app.<pacote>`) em `lume_core::keys`, persistindo o mapeamento ativo em `files/keys.v1`. Em `Shell::start`, se `keys.v1` não existir, o Lume SHALL gravar `KeyMap::default()`; se existir e for válido, SHALL carregá-lo sem regravar. Chaves vazias SHALL desabilitar a tecla customizável, e arquivos corrompidos SHALL ser autocurados.

#### Scenario: Primeiro uso cria o arquivo de mapeamento
- **WHEN** o Lume é iniciado pela primeira vez e `files/keys.v1` não existe
- **THEN** o Lume carrega as constantes padrão embutidas na compilação e grava `files/keys.v1` nos dados do app com todas as teclas padrão

#### Scenario: Uso do mapeamento salvo nos dados do app
- **WHEN** `files/keys.v1` existe com um código customizado (por exemplo, `menu=165`) e o Lume é iniciado
- **THEN** o Lume passa a reconhecer o código `165` como a tecla Menu sem reescrever `files/keys.v1`

#### Scenario: Persistência de atalhos de configurações e aplicativos
- **WHEN** o usuário associa a tecla `174` ao atalho de configuração `sleep` e a tecla `183` ao aplicativo `com.google.android.youtube.tv`
- **THEN** `files/keys.v1` persiste `cfg.sleep=174` e `app.com.google.android.youtube.tv=183` e recarrega esses atalhos no próximo início

#### Scenario: Uma tecla acionada por múltiplas teclas do controle remoto
- **WHEN** `files/keys.v1` associa múltiplos códigos a uma mesma tecla (por exemplo, `menu=82,174,165` ou `ok=23,66,160,96`)
- **THEN** o Lume reconhece qualquer um dos códigos físicos enviados pelo controle remoto (`82`, `174` ou `165`) como a ação correspondente (Menu), mantendo o arquivo válido sem reescrita

#### Scenario: Desabilitar uma tecla com valor vazio
- **WHEN** `files/keys.v1` define uma tecla com valor vazio (por exemplo, `menu=`)
- **THEN** nenhum código aciona aquela tecla e o arquivo é mantido como válido

#### Scenario: Autocura de arquivo corrompido por queda de energia
- **WHEN** `files/keys.v1` está truncado (sem `#end`) ou com cabeçalho inválido ao iniciar o Lume
- **THEN** o Lume restaura todas as teclas para o padrão da compilação e regrava imediatamente um `files/keys.v1` íntegro no disco

#### Scenario: Autocura de linha inválida ou tecla ausente
- **WHEN** `files/keys.v1` tem estrutura válida e customiza `menu=165`, mas a linha de `ok` contém um valor não numérico (`ok=23,abc`) ou a chave `home` está ausente
- **THEN** o Lume preserva `menu=165`, restaura `ok` e `home` para seus padrões da compilação e regrava o `files/keys.v1` corrigido no disco
