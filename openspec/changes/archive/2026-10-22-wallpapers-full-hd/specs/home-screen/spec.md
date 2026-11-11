## MODIFIED Requirements

### Requirement: Dock de favoritos
Os favoritos configurados SHALL ser exibidos na ordem configurada num dock: uma barra translúcida de cantos arredondados, flutuante, centralizada horizontalmente e acima da borda inferior da área segura. Cada favorito SHALL aparecer como o ícone do app em formato circular (padrão) ou quadrado de cantos arredondados (quando "Favoritos redondos" está desligado), sobre a cor do app quando o ícone tem transparência. O dock SHALL mostrar todos os favoritos (até 12) lado a lado, sem rolagem horizontal, e SHALL ficar dentro da área segura. OK sobre um favorito SHALL abrir o app.

#### Scenario: Favoritos padrão no primeiro uso
- **WHEN** o lume é aberto pela primeira vez com Netflix, YouTube e Spotify instalados
- **THEN** o dock mostra os apps conhecidos instalados, na ordem Netflix, YouTube, Prime Video, Spotify, Disney+, Max, Globoplay, limitados a 5, em formato circular

#### Scenario: Doze favoritos
- **WHEN** existem 12 favoritos
- **THEN** todos aparecem no dock ao mesmo tempo, dentro da área segura e sem rolagem, mesmo com o item focado ampliado

#### Scenario: Abrir pelo dock
- **WHEN** o foco está num ícone do dock e o usuário pressiona OK
- **THEN** o app daquele ícone é aberto

#### Scenario: Favoritos em formato circular
(O título do cenário é o da spec antiga, mantido para a validação; cobre os dois formatos.)
- **WHEN** a opção "Favoritos redondos" está desligada
- **THEN** cada item do dock é desenhado como um quadrado de cantos arredondados
