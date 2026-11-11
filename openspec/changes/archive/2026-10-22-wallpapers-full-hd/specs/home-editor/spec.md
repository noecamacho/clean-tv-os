## MODIFIED Requirements

### Requirement: Favoritos redondos
Acima do painel de favoritos, o modo edição SHALL exibir uma linha "Favoritos redondos" com uma chave de liga e desliga. O padrão de primeiro uso SHALL ser ligado (favoritos circulares no dock). Pressionar OK com o foco nessa linha SHALL alternar entre ligado e desligado (favoritos em quadrado de cantos arredondados) e gravar a configuração imediatamente. Um arquivo de configuração sem a chave SHALL ser lido como ligado.

#### Scenario: Padrão desligado no primeiro uso
(O título do cenário é o da spec antiga, mantido para a validação; o padrão agora é ligado.)
- **WHEN** o lume é aberto pela primeira vez e o usuário entra em "Tela Início"
- **THEN** a chave "Favoritos redondos" aparece ligada

#### Scenario: Ligar e desligar
(Alterna nos dois sentidos; o padrão é ligado.)
- **WHEN** o foco está em "Favoritos redondos" e o usuário pressiona OK
- **THEN** a chave passa para desligada, a alteração é gravada e o dock passa a desenhar os favoritos em quadrado de cantos arredondados

### Requirement: Restaurar padrão
O editor SHALL oferecer "Restaurar padrão", que, depois de uma confirmação, devolve os favoritos e a opção "Favoritos redondos" à configuração de primeiro uso. O histórico de uso SHALL ser mantido.

#### Scenario: Restaurar com confirmação
- **WHEN** o usuário seleciona "Restaurar padrão" e confirma
- **THEN** a configuração volta ao padrão de primeiro uso, com "Favoritos redondos" ligado

#### Scenario: Cancelar restauração
- **WHEN** o usuário seleciona "Restaurar padrão" e cancela
- **THEN** nada muda
