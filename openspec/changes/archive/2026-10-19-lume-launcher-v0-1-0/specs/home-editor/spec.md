# Spec Delta

## Purpose

Define o modo edição da tela Início (mockup "Editar tela Início"), em que o usuário organiza seções, carrossel e favoritos, e a persistência dessa configuração.

## ADDED Requirements

### Requirement: Acesso ao modo edição
O modo edição SHALL ser aberto pelo item "Tela Início" da tela Configurações do lume (engrenagem), ou pelo tile "Adicionar favoritos" quando não há favoritos. A tela SHALL mostrar o caminho "Configurações › Tela Início › Editar", o título "Editar tela Início", uma instrução curta e o botão "Concluído".

#### Scenario: Entrar e sair
- **WHEN** o usuário abre "Editar tela Início" e depois seleciona "Concluído"
- **THEN** a tela Início volta a ser exibida, já com as alterações feitas

#### Scenario: Voltar equivale a Concluído
- **WHEN** o usuário pressiona Voltar no nível principal do modo edição
- **THEN** as alterações já feitas são mantidas e a tela Início é exibida

### Requirement: Organizar seções
Cada seção da tela Início (carrossel de destaque e apps favoritos) SHALL aparecer como uma linha numerada com botões para subir, descer e ocultar/exibir. O botão subir da primeira seção e o botão descer da última SHALL ficar desabilitados.

#### Scenario: Mover seção
- **WHEN** o usuário seleciona "descer" na seção do carrossel, que está na posição 1
- **THEN** o carrossel passa para a posição 2, e a tela Início passa a mostrar os favoritos primeiro

#### Scenario: Ocultar seção
- **WHEN** o usuário oculta a seção de favoritos
- **THEN** a seção aparece como oculta no editor e não é exibida na tela Início

### Requirement: Configurar o carrossel
A seção do carrossel SHALL oferecer "Apps no carrossel", com um chip de seleção para cada app instalado da lista de apps conhecidos e para cada favorito, e "Atualizar" com as opções 3 s, 5 s, 10 s, 30 s e 1 min. Pelo menos um app SHALL permanecer selecionado.

#### Scenario: Desmarcar o último app
- **WHEN** só o Netflix está selecionado e o usuário tenta desmarcá-lo
- **THEN** o Netflix continua selecionado e aparece a dica "Pelo menos um app"

#### Scenario: Mudar intervalo
- **WHEN** o usuário seleciona "10 s"
- **THEN** o chip "10 s" fica ativo, e o carrossel passa a girar a cada 10 segundos

### Requirement: Configurar favoritos
A seção de favoritos SHALL listar os favoritos numerados, cada um com botões para mover à esquerda, mover à direita e remover, e uma área "Adicionar app" com os apps do catálogo que ainda não são favoritos. O limite SHALL ser de 12 favoritos.

#### Scenario: Reordenar favorito
- **WHEN** o usuário move o "Spotify" (posição 4) para a esquerda
- **THEN** o Spotify passa para a posição 3, e o app que estava em 3 passa para a 4

#### Scenario: Adicionar favorito
- **WHEN** o usuário seleciona "+ Chrome" em "Adicionar app"
- **THEN** o Chrome entra no fim da lista de favoritos e sai da área "Adicionar app"

#### Scenario: Limite atingido
- **WHEN** já existem 12 favoritos
- **THEN** a área "Adicionar app" fica desabilitada e mostra "Limite de 12 favoritos"

### Requirement: Restaurar padrão
O editor SHALL oferecer "Restaurar padrão", que, depois de uma confirmação, devolve seções, carrossel e favoritos à configuração de primeiro uso. O histórico de uso SHALL ser mantido.

#### Scenario: Restaurar com confirmação
- **WHEN** o usuário seleciona "Restaurar padrão" e confirma
- **THEN** a configuração volta ao padrão de primeiro uso

#### Scenario: Cancelar restauração
- **WHEN** o usuário seleciona "Restaurar padrão" e cancela
- **THEN** nada muda

### Requirement: Persistência da configuração
Cada alteração feita no editor SHALL ser gravada no armazenamento interno antes da próxima interação, de forma que sobreviva ao fechamento do processo, a uma reinicialização da TV ou a uma falta de energia durante a gravação (nesse caso, prevalece a configuração anterior completa ou a nova completa, nunca uma mistura corrompida).

#### Scenario: Reinício da TV
- **WHEN** o usuário reordena os favoritos e a TV é desligada da tomada e religada
- **THEN** os favoritos aparecem na nova ordem

#### Scenario: Arquivo de configuração ilegível
- **WHEN** o arquivo de configuração está corrompido ou é de uma versão desconhecida
- **THEN** o lume inicia com a configuração padrão sem travar e registra o problema no log
