## MODIFIED Requirements

### Requirement: Permissões mínimas
O lume SHALL NOT solicitar a permissão de internet, nenhuma permissão perigosa (runtime) e nem `QUERY_ALL_PACKAGES`. A única permissão declarada SHALL ser `REQUEST_DELETE_PACKAGES` (permissão normal, concedida na instalação), usada apenas para abrir o desinstalador do sistema a pedido do usuário. A visibilidade dos apps SHALL ser declarada por consultas de intent (LEANBACK_LAUNCHER e LAUNCHER).

#### Scenario: Inspeção do manifesto
- **WHEN** o manifesto final do APK é inspecionado (`aapt dump permissions`)
- **THEN** não aparecem `android.permission.INTERNET` nem `QUERY_ALL_PACKAGES`, e a única permissão listada é `android.permission.REQUEST_DELETE_PACKAGES`

#### Scenario: Nenhum tráfego de rede
- **WHEN** o lume é usado em todas as telas
- **THEN** o processo não abre nenhuma conexão de rede

#### Scenario: Desinstalar no Android 9 e 11
- **WHEN** o usuário escolhe "Desinstalar" no menu de contexto no Android TV 9 ou 11
- **THEN** o diálogo de desinstalação do sistema aparece, sem nenhum pedido de permissão
