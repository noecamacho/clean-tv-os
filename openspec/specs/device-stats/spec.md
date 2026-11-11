# device-stats Specification

## Purpose
Mostra, na tela de Configurações do Lume, estatísticas da TV (memória, disco e CPU) lidas só enquanto essa tela está aberta, para o usuário acompanhar a saúde do aparelho sem custo no uso normal do launcher.

## Requirements

### Requirement: Painel de estatísticas
A tela de Configurações do Lume SHALL exibir, à direita dos itens, um painel ("Status da TV") com a memória (usada e total, com barra), o armazenamento de dados (usado e total, com barra) e o clock da CPU ("Clock da CPU" / "CPU clock", exibindo a frequência média atual por núcleo e a frequência máxima por núcleo no formato `<atual> / <máximo>`, ex.: `1,35 GHz / 1,50 GHz` ou `750 MHz / 1,50 GHz`, com barra proporcional). O painel SHALL estar sempre visível nessa tela, sem interruptor, e SHALL NOT receber foco. A leitura SHALL NOT exigir nenhuma permissão nova do Android.

#### Scenario: Painel visível
- **WHEN** o usuário abre a tela de Configurações do Lume
- **THEN** o painel à direita mostra a memória, o armazenamento e o clock da CPU da TV no formato `<atual> / <máximo>`, e o foco está no primeiro item da lista

#### Scenario: Clock da CPU em GHz e MHz
- **WHEN** os núcleos da TV operam a 1350 MHz com máximo de 1500 MHz em português
- **THEN** a linha "Clock da CPU" exibe `"1,35 GHz / 1,50 GHz"` e preenche 90% da barra

#### Scenario: CPU indisponível
- **WHEN** o sistema não expõe a frequência de clock da CPU nem contadores de uso
- **THEN** a linha de clock da CPU mostra um traço ("—"), e a memória e o armazenamento continuam sendo exibidos

### Requirement: Leitura só com a tela visível
O Lume SHALL ler as estatísticas a cada 2 segundos apenas enquanto a tela de Configurações estiver visível. Ao sair dela (voltar, abrir a tela Sobre, abrir o modo edição) ou ao ir para segundo plano, o Lume SHALL parar de ler e SHALL NOT pedir quadros por causa do painel. Com o painel visível, o Lume SHALL manter no máximo 1% de quadros de 33 ms ou mais.

#### Scenario: Atualização periódica
- **WHEN** a tela de Configurações permanece aberta por 10 segundos
- **THEN** os valores do painel são atualizados a cada 2 segundos

#### Scenario: Sair da tela
- **WHEN** o usuário volta à grade de Apps
- **THEN** o Lume deixa de ler as estatísticas, e o laço volta a não pedir quadros além do relógio

#### Scenario: Segundo plano
- **WHEN** o usuário abre outro app com a tela de Configurações aberta
- **THEN** o Lume não faz nenhuma leitura enquanto está em segundo plano

### Requirement: Tela de informações técnicas da TV
O Lume SHALL oferecer, a partir da tela de Configurações, uma tela dedicada ("Informações da TV") exibindo a ficha técnica do aparelho na vertical, contendo dados de aparelho, processamento, gráficos, memória, sistema, firmware, tela e conexão de rede.

#### Scenario: Acesso pela tela de Configurações
- **WHEN** o usuário seleciona o item "Informações da TV" na lista de Configurações
- **THEN** a tela de Informações da TV é exibida com o caminho de navegação "Configurações › Informações da TV" e os dados técnicos lidos

#### Scenario: Dado indisponível
- **WHEN** o sistema da TV não expõe determinado campo (como modelo do SoC ou patch de segurança)
- **THEN** a linha correspondente exibe um traço ("—") sem quebrar o alinhamento da tela

#### Scenario: Rolagem com controle remoto
- **WHEN** o usuário pressiona as setas para baixo ou para cima no controle remoto
- **THEN** a ficha técnica rola verticalmente de forma suave, sem sair dos limites da área visível

#### Scenario: Leitura única ao abrir
- **WHEN** a tela de Informações da TV é aberta
- **THEN** a plataforma lê a ficha uma vez, com a interface parada, e não volta a ler enquanto a tela fica aberta

#### Scenario: Conexão de rede
- **WHEN** a TV tem uma interface de rede com endereço IPv4
- **THEN** a ficha mostra o tipo (Wi-Fi, cabo Ethernet ou outro) e o endereço, preferindo o cabo ao Wi-Fi
