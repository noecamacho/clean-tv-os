## ADDED Requirements

### Requirement: Menu de contexto na grade
O menu de contexto de um app da grade (aberto pelo botão Menu, ver `launcher-shell`) SHALL ter, nesta ordem:

- "Adicionar ao Início", se o app não é favorito, ou "Remover do Início", se é. Com 12 favoritos, "Adicionar ao Início" SHALL aparecer desabilitado, com a indicação "Limite de 12 favoritos", e SHALL NOT receber foco.
- "Informações do app", que SHALL abrir a tela do app nas configurações do Android.
- "Desinstalar", apenas para apps que não são do sistema, que SHALL abrir o desinstalador do Android para o app.

Adicionar SHALL colocar o app no fim dos favoritos; remover SHALL tirá-lo dos favoritos. As duas mudanças SHALL ser gravadas como as do modo edição (`home-editor`) e o foco SHALL voltar ao tile do app. O tile "Lume" SHALL NOT ter menu de contexto. Quando um app desinstalado sai do catálogo, a grade SHALL ser atualizada e o foco SHALL ir para o tile que ocupar a mesma posição, ou para o último, se não houver.

#### Scenario: Adicionar ao Início
- **WHEN** o usuário abre o menu sobre o "Chrome", que não é favorito, e escolhe "Adicionar ao Início"
- **THEN** o Chrome entra no fim do dock, a mudança é gravada e o foco volta ao tile do Chrome

#### Scenario: Remover do Início
- **WHEN** o usuário abre o menu sobre o "YouTube", que é favorito
- **THEN** o primeiro item é "Remover do Início", e escolhê-lo tira o YouTube do dock

#### Scenario: Limite de favoritos
- **WHEN** já existem 12 favoritos e o usuário abre o menu sobre um app que não é favorito
- **THEN** "Adicionar ao Início" aparece desabilitado com "Limite de 12 favoritos", e o foco começa em "Informações do app"

#### Scenario: App do sistema
- **WHEN** o usuário abre o menu sobre um app do sistema (ex.: Configurações)
- **THEN** o menu não tem "Desinstalar"

#### Scenario: Desinstalar
- **WHEN** o usuário escolhe "Desinstalar" no menu de um app instalado pelo usuário e confirma no diálogo do Android
- **THEN** o app sai da grade e do dock, e o foco vai para o tile na mesma posição

#### Scenario: Tile do Lume
- **WHEN** o foco está no tile "Lume" e o usuário pressiona Menu
- **THEN** nenhum menu aparece
