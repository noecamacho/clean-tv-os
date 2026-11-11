# Spec Delta

## MODIFIED Requirements

### Requirement: Atualização após instalar ou remover
O catálogo SHALL refletir apps instalados, atualizados ou removidos sempre que o lume volta ao primeiro plano, sem que o usuário precise reiniciar o lume.

#### Scenario: Abertura sem escanear tudo
- **WHEN** o lume é aberto de novo, com o catálogo em cache e sem reinicialização do aparelho
- **THEN** o lume consulta só as mudanças de pacotes desde a última vez, sem ler os rótulos de todos os apps

#### Scenario: App instalado com o lume em segundo plano
- **WHEN** o usuário instala o Globoplay pela loja e depois pressiona Home
- **THEN** o Globoplay aparece na grade de Apps

#### Scenario: Favorito desinstalado
- **WHEN** um app que estava nos favoritos é desinstalado
- **THEN** ele é removido dos favoritos e do dock, e as demais posições mantêm a ordem relativa

## ADDED Requirements

### Requirement: Ícone do app no dock
Cada favorito do dock SHALL exibir o ícone do app em formato quadrado de cantos arredondados, e não o banner de TV. Quando o ícone tem transparência, ele SHALL aparecer sobre a cor do app. Os ícones SHALL ser carregados aos poucos, sem atrasar o primeiro quadro nem a resposta às teclas. Enquanto o ícone não estiver disponível, o dock SHALL mostrar a cor do app com a inicial do nome. As imagens do dock SHALL ficar na mesma textura gráfica das imagens da grade de Apps, criada só com o lume em primeiro plano e liberada ao ir para segundo plano.

#### Scenario: App com banner de TV nos favoritos
- **WHEN** o Netflix, que declara banner de TV, é um favorito
- **THEN** o dock mostra o ícone quadrado do Netflix, e a grade de Apps continua mostrando o banner

#### Scenario: Ícones carregando
- **WHEN** o lume abre e os ícones ainda não foram carregados
- **THEN** o dock é desenhado de imediato com a cor e a inicial de cada app, e cada um dá lugar ao ícone assim que ele fica pronto

#### Scenario: Segundo plano
- **WHEN** outro app passa ao primeiro plano
- **THEN** a textura dos ícones é liberada, e os ícones são carregados de novo na próxima vez que o lume ficar visível
