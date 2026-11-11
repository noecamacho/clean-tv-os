## ADDED Requirements

### Requirement: Mapeamento centralizado e persistente de teclas
O Lume SHALL centralizar as teclas lógicas, constantes `KEYCODE_*` e padrões `DEFAULT_*` em `lume_core::keys`, persistindo o mapeamento ativo em `files/keys.v1`. Em `Shell::start`, se `keys.v1` não existir, o Lume SHALL gravar `KeyMap::default()`; se existir e for válido, SHALL carregá-lo sem regravar. Chaves vazias SHALL desabilitar a tecla, e arquivos ou linhas corrompidas SHALL ser autocurados e regravados em disco.

#### Scenario: Primeiro uso cria o arquivo de mapeamento
- **WHEN** o Lume é iniciado pela primeira vez e `files/keys.v1` não existe
- **THEN** o Lume carrega as constantes padrão embutidas na compilação e grava `files/keys.v1` nos dados do app com todas as 8 teclas

#### Scenario: Uso do mapeamento salvo nos dados do app
- **WHEN** `files/keys.v1` existe com um código customizado (por exemplo, `menu=165`) e o Lume é iniciado
- **THEN** o Lume passa a reconhecer o código `165` como a tecla Menu sem reescrever `files/keys.v1`

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

### Requirement: Perfil de teclas embutido na compilação e limpeza de dados
O build do Lume SHALL aceitar opcionalmente a variável `LUME_KEYS_FILE` apontando para um arquivo no formato `lume=1 ... #end`. Quando informada, o build SHALL validar o arquivo em tempo de compilação e embutir seus valores como os padrões `DEFAULT_*` do binário; quando omitida, SHALL usar as constantes originais do Lume. Ao limpar os dados do app no Android, o próximo início SHALL recriar `files/keys.v1` a partir dos `DEFAULT_*` do binário.

#### Scenario: Limpar dados do app em compilação padrão
- **WHEN** o usuário limpa os dados do Lume nas configurações do Android em uma compilação padrão (sem `LUME_KEYS_FILE`) e abre o Lume novamente
- **THEN** o arquivo antigo some na limpeza e o Lume recria `files/keys.v1` com as constantes padrão originais do Lume

#### Scenario: Compilação com arquivo embutido e limpeza de dados
- **WHEN** o Lume foi compilado com `LUME_KEYS_FILE` definindo um mapeamento específico para uma TV, o usuário altera `files/keys.v1` e depois limpa os dados do app no Android
- **THEN** ao abrir o Lume novamente, `files/keys.v1` é recriado com o mapeamento que foi embutido na compilação via `LUME_KEYS_FILE`
