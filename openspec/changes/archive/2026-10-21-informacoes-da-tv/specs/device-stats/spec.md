# device-stats Specification Delta

## ADDED Requirements

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
