# Spec Delta

## MODIFIED Requirements

### Requirement: Ícone do app no dock
Cada favorito do dock SHALL exibir o ícone do app no formato configurado (quadrado de cantos arredondados por padrão, ou circular com "Favoritos redondos" ligado), e não o banner de TV. Quando o ícone tem transparência, ele SHALL aparecer sobre a cor do app. Os ícones SHALL ser carregados aos poucos, sem atrasar o primeiro quadro nem a resposta às teclas, e a troca entre quadrado arredondado e circular SHALL acontecer na hora, sem exigir releitura dos ícones já em cache. Enquanto o ícone não estiver disponível, o dock SHALL mostrar a cor do app com a inicial do nome. As imagens do dock SHALL ficar na mesma textura gráfica das imagens da grade de Apps, criada só com o lume em primeiro plano e liberada ao ir para segundo plano.

#### Scenario: App com banner de TV nos favoritos
- **WHEN** o Netflix, que declara banner de TV, é um favorito
- **THEN** o dock mostra o ícone do Netflix no formato configurado, e a grade de Apps continua mostrando o banner

#### Scenario: Ícones carregando
- **WHEN** o lume abre e os ícones ainda não foram carregados
- **THEN** o dock é desenhado de imediato com a cor e a inicial de cada app, e cada um dá lugar ao ícone assim que ele fica pronto

#### Scenario: Segundo plano
- **WHEN** outro app passa ao primeiro plano
- **THEN** a textura dos ícones é liberada, e os ícones são carregados de novo na próxima vez que o lume ficar visível

#### Scenario: Troca de formato sem recarregar ícones
- **WHEN** os ícones do dock já estão carregados e o usuário alterna "Favoritos redondos"
- **THEN** os ícones passam a ser exibidos no novo formato imediatamente, sem voltar às iniciais
