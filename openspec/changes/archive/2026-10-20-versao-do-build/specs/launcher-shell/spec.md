## MODIFIED Requirements

### Requirement: Tela de Configurações do Lume
A tela "Configurações" do Lume SHALL ser aberta pelo tile "Lume" da grade de Apps, com os itens "Tela Início" (abre o modo edição) e "Sobre o Lume". Ela SHALL NOT ter item para as configurações do Android, nem ser aberta pela engrenagem. A tela SHALL exibir, à direita dos itens, o painel de estatísticas da TV (capacidade `device-stats`). A tela SHALL exibir o caminho "Configurações" e, no canto inferior direito, a versão do build instalado (capacidade `platform-compatibility`, requisito "Versão do build"). Uma versão de release SHALL aparecer com "v" na frente (ex.: "Versão v0.3.0"); uma versão de build local ou `dev` SHALL aparecer como é (ex.: "Versão 4d8438b-dirty"). As telas abaixo dela SHALL exibir o caminho de navegação a partir de "Configurações" (ex.: "Configurações › Tela Início"). Voltar SHALL retraçar o caminho de entrada, um passo por vez: da tela de edição para a tela "Configurações" (com o foco em "Tela Início") e dela para o tile "Lume" da aba Apps. Voltar da tela SHALL devolver o foco ao tile "Lume" na aba Apps.

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
- **THEN** ela lista apenas "Tela Início" e "Sobre o Lume"

#### Scenario: Voltar ao tile
- **WHEN** o usuário pressiona Voltar na tela "Configurações" do Lume
- **THEN** a aba Apps é exibida com o foco no tile "Lume"

#### Scenario: Ir para a edição da tela Início
- **WHEN** o usuário seleciona "Tela Início" em Configurações
- **THEN** o modo edição é aberto com o caminho "Configurações › Tela Início" e o título "Tela Início"

#### Scenario: Voltar passo a passo
- **WHEN** o usuário está no modo edição aberto pelas Configurações e pressiona Voltar duas vezes
- **THEN** a primeira volta à tela "Configurações" com o foco em "Tela Início", e a segunda à aba Apps com o foco no tile "Lume"

#### Scenario: Tela Sobre
- **WHEN** o usuário escolhe "Sobre o Lume"
- **THEN** é exibida uma tela com a versão do build (a mesma do canto da tela "Configurações"), a licença (GPL-3.0-or-later), o endereço do código-fonte `https://github.com/douglaspands/lume-tv-launcher` e as licenças de terceiros (fonte, ícones e bibliotecas)
