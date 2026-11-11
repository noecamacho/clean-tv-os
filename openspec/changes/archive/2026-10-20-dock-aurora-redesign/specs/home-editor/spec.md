# Spec Delta

## MODIFIED Requirements

### Requirement: Acesso ao modo edição
O modo edição SHALL ser aberto pelo item "Tela Início" da tela Configurações do lume (engrenagem), ou pelo item "Adicionar favoritos" do dock quando não há favoritos. A tela SHALL mostrar o caminho "Configurações › Tela Início › Editar", o título "Editar tela Início", uma instrução curta e o botão "Concluído".

#### Scenario: Entrar e sair
- **WHEN** o usuário abre "Editar tela Início" e depois seleciona "Concluído"
- **THEN** a tela Início volta a ser exibida, já com as alterações feitas

#### Scenario: Voltar equivale a Concluído
- **WHEN** o usuário pressiona Voltar no nível principal do modo edição
- **THEN** as alterações já feitas são mantidas e a tela Início é exibida

### Requirement: Restaurar padrão
O editor SHALL oferecer "Restaurar padrão", que, depois de uma confirmação, devolve os favoritos à configuração de primeiro uso. O histórico de uso SHALL ser mantido.

#### Scenario: Restaurar com confirmação
- **WHEN** o usuário seleciona "Restaurar padrão" e confirma
- **THEN** a configuração volta ao padrão de primeiro uso

#### Scenario: Cancelar restauração
- **WHEN** o usuário seleciona "Restaurar padrão" e cancela
- **THEN** nada muda

### Requirement: Persistência da configuração
Cada alteração feita no editor SHALL ser gravada no armazenamento interno antes da próxima interação, de forma que sobreviva ao fechamento do processo, a uma reinicialização da TV ou a uma falta de energia durante a gravação (nesse caso, prevalece a configuração anterior completa ou a nova completa, nunca uma mistura corrompida). Um arquivo gravado por uma versão anterior, que ainda traga seções e carrossel, SHALL continuar legível: os favoritos são mantidos e o restante é ignorado.

#### Scenario: Reinício da TV
- **WHEN** o usuário reordena os favoritos e a TV é desligada da tomada e religada
- **THEN** os favoritos aparecem na nova ordem

#### Scenario: Arquivo de configuração ilegível
- **WHEN** o arquivo de configuração está corrompido ou é de uma versão desconhecida
- **THEN** o lume inicia com a configuração padrão sem travar e registra o problema no log

#### Scenario: Arquivo de uma versão com carrossel
- **WHEN** o arquivo de configuração contém seções, apps do carrossel e intervalo, além dos favoritos
- **THEN** o lume abre com os mesmos favoritos, na mesma ordem, e ignora as chaves do carrossel e das seções

## REMOVED Requirements

### Requirement: Organizar seções
**Reason**: A tela Início tem uma única seção (o dock); não há mais o que subir, descer ou ocultar.
**Migration**: Nenhuma. Seções ocultas ou reordenadas em versões anteriores deixam de ter efeito.

### Requirement: Configurar o carrossel
**Reason**: O carrossel foi removido da tela Início.
**Migration**: Nenhuma. Os apps e o intervalo do carrossel configurados antes são ignorados.
