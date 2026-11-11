# app-catalog Specification

## Purpose
Mantém o catálogo dos apps instalados que podem ser abertos, com nome e cor de cada um, e o histórico local de uso que o lume registra ao abri-los.

## Requirements

### Requirement: Descoberta de apps
O catálogo SHALL incluir todo app instalado que tenha uma atividade de entrada para TV (LEANBACK_LAUNCHER) ou, na falta dela, uma atividade de entrada padrão (LAUNCHER) **desde que o app não seja de sistema**. Apps de sistema que só têm a atividade padrão (serviços, telas de configuração e menus de fábrica, como o serviço do controle remoto ou o OTA) SHALL NOT aparecer, como no launcher original da TV. O próprio lume SHALL ficar de fora do catálogo.

#### Scenario: App de TV instalado
- **WHEN** o Netflix está instalado com uma atividade LEANBACK_LAUNCHER
- **THEN** o Netflix aparece no catálogo

#### Scenario: App de sistema sem atividade de TV
- **WHEN** um app de sistema tem apenas a atividade de entrada padrão (LAUNCHER), como o serviço do controle remoto
- **THEN** ele não aparece no catálogo

#### Scenario: App instalado pelo usuário sem atividade de TV
- **WHEN** um app instalado pelo usuário tem apenas a atividade de entrada padrão (LAUNCHER)
- **THEN** ele aparece no catálogo

#### Scenario: Lume excluído
- **WHEN** o catálogo é montado
- **THEN** o lume não aparece na lista de apps

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

### Requirement: Identidade visual do app
Cada app SHALL ter um nome de exibição (o rótulo do sistema) e uma cor de fundo de tile. Para apps conhecidos (pelo menos Netflix, YouTube, Prime Video, Disney+, Max, Spotify, Globoplay, Play Store e Chrome), a cor SHALL vir de uma tabela fixa alinhada ao mockup. Para os demais, a cor SHALL ser derivada do ícone do app e ajustada para manter o texto branco legível.

#### Scenario: App conhecido
- **WHEN** o Spotify está instalado
- **THEN** o tile do Spotify usa o verde definido no tema

#### Scenario: App desconhecido
- **WHEN** um app sem cor tabelada é instalado
- **THEN** o tile usa uma cor derivada do ícone, com contraste de pelo menos 4.5:1 em relação ao texto branco

### Requirement: Imagem do app no tile
O tile de cada app SHALL parecer o do launcher de TV do Android: se o app declara um banner de TV (retangular, 16:9), o tile SHALL exibir o banner cobrindo o tile inteiro, com os mesmos cantos arredondados do tile; se o app não tem banner, o tile SHALL exibir o ícone do app, centralizado sobre a cor do tile. As imagens SHALL ser carregadas aos poucos, sem atrasar o primeiro quadro nem a resposta às teclas. Enquanto a imagem não estiver carregada, ou se o app não tiver nenhuma, o tile SHALL exibir o nome do app. As imagens ocupam uma única textura, criada só com o lume em primeiro plano e liberada com o resto da memória gráfica ao ir para segundo plano.

#### Scenario: App com banner de TV
- **WHEN** o Netflix, que declara um banner de TV, está instalado
- **THEN** o tile do Netflix exibe o banner retangular em todo o tile, e não um ícone quadrado

#### Scenario: App sem banner de TV
- **WHEN** um app só tem ícone
- **THEN** o tile exibe o ícone centralizado sobre a cor do tile

#### Scenario: Imagens carregando
- **WHEN** o lume abre e as imagens ainda não foram carregadas
- **THEN** a tela é desenhada de imediato com os nomes nos tiles, e cada nome dá lugar à imagem do app assim que ela fica pronta

#### Scenario: Segundo plano
- **WHEN** outro app passa ao primeiro plano
- **THEN** a textura das imagens é liberada, e as imagens são carregadas de novo na próxima vez que o lume ficar visível

#### Scenario: Imagens em cache
- **WHEN** o lume carrega a imagem de um app cuja imagem já foi preparada e que não foi atualizado
- **THEN** a imagem vem de um arquivo de cache no armazenamento do lume, sem abrir os recursos do outro app

#### Scenario: App atualizado ou removido
- **WHEN** um app é atualizado ou removido
- **THEN** a imagem antiga dele não é mais usada, e o arquivo de cache obsoleto é apagado

### Requirement: Abrir app
Selecionar um app (OK sobre o tile) SHALL abrir o app pela sua atividade de entrada de TV, ou pela atividade de entrada padrão quando não houver uma de TV.

#### Scenario: Abrir pelo tile
- **WHEN** o foco está no tile "Prime Video" e o usuário pressiona OK
- **THEN** o Prime Video é aberto

#### Scenario: App removido entre a exibição e o clique
- **WHEN** o usuário seleciona um app que acabou de ser desinstalado
- **THEN** o lume não trava, exibe um aviso breve "App não encontrado" e atualiza o catálogo

### Requirement: Histórico local de uso
O lume SHALL registrar, apenas no armazenamento interno do dispositivo, o horário da última abertura e as contagens de abertura dos últimos 7 dias de cada app aberto pelo lume, e o horário em que cada app foi visto pela primeira vez. Esses dados SHALL NOT sair do dispositivo.

#### Scenario: Registro de abertura
- **WHEN** o usuário abre o YouTube pelo lume
- **THEN** a última abertura do YouTube passa a ser o horário atual, e a contagem semanal aumenta em 1

#### Scenario: Histórico apagado ao limpar dados
- **WHEN** o usuário limpa os dados do lume nas configurações do Android
- **THEN** o histórico de uso deixa de existir

### Requirement: Ícone do app no dock
Cada favorito do dock SHALL exibir o ícone do app no formato configurado (quadrado de cantos arredondados por padrão, ou circular com "Favoritos redondos" ligado), e não o banner de TV. Quando o ícone tem transparência, ele SHALL aparecer sobre a cor do app no formato configurado do dock, acompanhando o formato dos demais favoritos. Os ícones SHALL ser carregados preferencialmente na resolução mais alta disponível no aplicativo (com limite de tentativas e fallback limpo para a densidade padrão do sistema) e rasterizados com filtragem bilinear e anti-aliasing. Os ícones SHALL ser carregados aos poucos, sem atrasar o primeiro quadro nem a resposta às teclas, e a troca entre quadrado arredondado e circular SHALL acontecer na hora, sem exigir releitura dos ícones já em cache. Enquanto o ícone não estiver disponível, o dock SHALL mostrar a cor do app com a inicial do nome. As imagens do dock SHALL ficar na mesma textura gráfica das imagens da grade de Apps, criada só com o lume em primeiro plano e liberada ao ir para segundo plano.

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

#### Scenario: Carregamento em alta resolução
- **WHEN** um app possui ícones em alta densidade (como `xxhdpi` ou superior)
- **THEN** o dock carrega o asset em alta resolução, rasterizando com filtragem suave e preservando a definição e nitidez

#### Scenario: Ícone com transparência acompanha os demais
- **WHEN** um app possui ícone com transparência ou formato não adaptativo (como o Globoplay)
- **THEN** ele é exibido sobre a base geométrica correspondente ao formato global do dock, acompanhando os demais aplicativos
