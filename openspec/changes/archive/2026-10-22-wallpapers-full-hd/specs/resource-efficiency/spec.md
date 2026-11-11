## MODIFIED Requirements

### Requirement: Tamanho do pacote
O APK de release SHALL ter no máximo 40 MB, contendo as duas arquiteturas suportadas. Os papéis de parede de alta definição SHALL entrar no APK uma única vez, fora das bibliotecas nativas de cada arquitetura, para que o limite não cresça com o número de arquiteturas.

#### Scenario: Verificação no CI
- **WHEN** o CI gera o APK de release
- **THEN** o build falha se o arquivo passar de 40 MB

#### Scenario: Papéis de parede fora das bibliotecas nativas
- **WHEN** o APK de release é verificado pelo script do projeto
- **THEN** a verificação falha se um papel de parede aparecer dentro de uma biblioteca nativa ou mais de uma vez no pacote
