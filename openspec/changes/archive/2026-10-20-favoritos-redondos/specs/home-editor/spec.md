# Spec Delta

## ADDED Requirements

### Requirement: Favoritos redondos
Acima do painel de favoritos, o modo edição SHALL exibir uma linha "Favoritos redondos" com uma chave de liga e desliga. O padrão de primeiro uso SHALL ser desligado (favoritos em quadrado de cantos arredondados). Pressionar OK com o foco nessa linha SHALL alternar entre ligado (favoritos circulares no dock) e desligado e gravar a configuração imediatamente.

#### Scenario: Padrão desligado no primeiro uso
- **WHEN** o lume é aberto pela primeira vez e o usuário entra em "Tela Início"
- **THEN** a chave "Favoritos redondos" aparece desligada

#### Scenario: Ligar e desligar
- **WHEN** o foco está em "Favoritos redondos" e o usuário pressiona OK
- **THEN** a chave passa para ligada, a alteração é gravada e o dock passa a desenhar os favoritos em formato circular

## MODIFIED Requirements

### Requirement: Restaurar padrão
O editor SHALL oferecer "Restaurar padrão", que, depois de uma confirmação, devolve os favoritos e a opção "Favoritos redondos" à configuração de primeiro uso. O histórico de uso SHALL ser mantido.

#### Scenario: Restaurar com confirmação
- **WHEN** o usuário seleciona "Restaurar padrão" e confirma
- **THEN** a configuração volta ao padrão de primeiro uso, com "Favoritos redondos" desligado

#### Scenario: Cancelar restauração
- **WHEN** o usuário seleciona "Restaurar padrão" e cancela
- **THEN** nada muda

### Requirement: Persistência da configuração
Cada alteração feita no editor SHALL ser gravada no armazenamento interno antes da próxima interação, de forma que sobreviva ao fechamento do processo, a uma reinicialização da TV ou a uma falta de energia durante a gravação (nesse caso, prevalece a configuração anterior completa ou a nova completa, nunca uma mistura corrompida). Um arquivo gravado por uma versão anterior, que ainda traga seções e carrossel ou não traga a chave de favoritos redondos, SHALL continuar legível: os favoritos são mantidos, "Favoritos redondos" assume desligado quando ausente e o restante é ignorado.

#### Scenario: Reinício da TV
- **WHEN** o usuário reordena os favoritos ou liga "Favoritos redondos" e a TV é desligada da tomada e religada
- **THEN** os favoritos aparecem na nova ordem e com o formato escolhido

#### Scenario: Arquivo de configuração ilegível
- **WHEN** o arquivo de configuração está corrompido ou é de uma versão desconhecida
- **THEN** o lume inicia com a configuração padrão sem travar e registra o problema no log

#### Scenario: Arquivo de uma versão com carrossel
- **WHEN** o arquivo de configuração contém seções, apps do carrossel e intervalo, além dos favoritos
- **THEN** o lume abre com os mesmos favoritos, na mesma ordem, com "Favoritos redondos" desligado, e ignora as chaves do carrossel e das seções

### Requirement: Botões do modo edição
Os botões "Restaurar padrão" (secundário) e "Concluído" (primário) SHALL ficar na mesma linha sob o painel de favoritos, em lados opostos da tela: "Restaurar padrão" na margem esquerda e "Concluído" na direita, ambos com 48 de altura. Ao abrir o modo edição o foco SHALL começar na linha "Favoritos redondos". Pressionar Baixo a partir dos favoritos e dos apps a adicionar SHALL levar o foco a "Concluído"; chegar a "Restaurar padrão" SHALL exigir Esquerda a partir de "Concluído", e Direita volta a ele. Cima a partir da linha de botões SHALL voltar aos apps a adicionar.

#### Scenario: Baixo vai para Concluído
- **WHEN** o foco está nos apps a adicionar e o usuário pressiona Baixo na última linha
- **THEN** o foco vai para "Concluído", e "Restaurar padrão" não recebe foco

#### Scenario: Restaurar pela esquerda
- **WHEN** o foco está em "Concluído" e o usuário pressiona Esquerda
- **THEN** o foco vai para "Restaurar padrão", e Direita o devolve a "Concluído"

#### Scenario: Foco inicial
- **WHEN** o modo edição é aberto
- **THEN** o foco está na linha "Favoritos redondos"
