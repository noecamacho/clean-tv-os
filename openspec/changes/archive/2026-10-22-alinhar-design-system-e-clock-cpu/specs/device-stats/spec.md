# Spec Delta

## MODIFIED Requirements

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
