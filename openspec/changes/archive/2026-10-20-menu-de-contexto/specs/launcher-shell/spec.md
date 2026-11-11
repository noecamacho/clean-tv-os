## ADDED Requirements

### Requirement: Botão Menu
O Lume SHALL reconhecer o botão Menu do controle remoto e do teclado (`KEYCODE_MENU`). Com o foco num item que tem menu de contexto (um app da grade de Apps ou um favorito do dock), Menu SHALL abrir o menu de contexto desse item (capacidades `apps-screen` e `home-screen`). Em qualquer outro lugar (barra superior, tile "Lume", item "Adicionar favoritos", Configurações, Sobre, modo edição), Menu SHALL NOT fazer nada. Com um menu aberto, Menu ou Voltar SHALL fechá-lo sem executar nenhuma ação e devolver o foco ao item de origem; OK SHALL executar o item em foco e fechar o menu. Enquanto o menu estiver aberto, o foco SHALL ficar preso nele, e Home SHALL fechá-lo antes de seguir o comportamento normal de Home. O mesmo vale para o modo de mover um favorito (`home-screen`): ele prende as teclas, e Home o cancela antes de seguir.

#### Scenario: Abrir sobre um app
- **WHEN** o foco está num app da grade e o usuário pressiona Menu
- **THEN** o menu de contexto do app aparece junto do tile, com o foco no primeiro item

#### Scenario: Sem menu na barra superior
- **WHEN** o foco está na engrenagem e o usuário pressiona Menu
- **THEN** nada muda na tela

#### Scenario: Fechar sem ação
- **WHEN** o menu de contexto está aberto e o usuário pressiona Voltar
- **THEN** o menu fecha, nada é alterado e o foco volta ao item de origem

#### Scenario: Foco preso
- **WHEN** o menu está aberto com o foco no último item e o usuário pressiona Baixo
- **THEN** o foco permanece no menu
