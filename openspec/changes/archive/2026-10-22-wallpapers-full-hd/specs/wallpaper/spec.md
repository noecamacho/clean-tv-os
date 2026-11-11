## MODIFIED Requirements

### Requirement: Definição em qualquer resolução
O papel de parede SHALL ser desenhado com um texel por pixel do buffer de renderização: em 1920x1080 (inclusive em TVs 4K, em que o scaler da TV amplia o buffer), direto da imagem de 1920x1080; em buffers menores, de uma imagem reduzida uma vez para o tamanho exato do buffer, com filtro de área e dither refeito nessa resolução. As estrelas fazem parte da imagem do master (a lista de estrelas vetoriais de cada papel de parede é vazia no catálogo atual; o renderizador continua a desenhar pontos de 1 ou 2 pixels do buffer quando a lista não é vazia). O papel de parede SHALL NOT mostrar faixas de cor, blocos nem moiré.

#### Scenario: TV 4K
- **WHEN** a superfície de exibição informa 3840x2160 e o lume renderiza em 1920x1080
- **THEN** o papel de parede é desenhado 1:1 a partir da imagem de 1920x1080, com as estrelas que fazem parte do master reduzidas por média junto com o resto da imagem

#### Scenario: Fidelidade à referência
- **WHEN** o papel de parede é renderizado a 1920x1080, sem escurecimento
- **THEN** a diferença média absoluta por canal em relação à imagem do catálogo é de no máximo 1 em 255

#### Scenario: TV 720p
- **WHEN** a superfície tem 1280x720 pixels
- **THEN** a diferença média absoluta por canal em relação a uma redução Lanczos da imagem do catálogo para 1280x720 é de no máximo 2 em 255

#### Scenario: Linhas finas em 720p
- **WHEN** um papel de parede com linhas finas (como a topografia) é renderizado em 1280x720
- **THEN** a imagem não mostra moiré: a energia de alta frequência da renderização não passa da energia da redução Lanczos de referência em mais de 10%

#### Scenario: Degradês sem faixas
- **WHEN** um degradê escuro do papel de parede é renderizado em 1280x720 ou em 1920x1080
- **THEN** a imagem tem dither na resolução do buffer, e nenhuma faixa contínua de pixels com o mesmo valor ao longo do degradê passa de 32 px de largura

### Requirement: Custo de recursos do papel de parede
O papel de parede SHALL ocupar no máximo 6,5 MB de memória gráfica em 1920x1080 e 3 MB em 1280x720 (uma textura do tamanho do buffer), mais a imagem de origem só enquanto a redução está em curso. O APK SHALL levar uma única cópia de cada papel de parede. A textura SHALL ser criada só com o lume em primeiro plano e liberada com o resto da memória gráfica ao ir para segundo plano. O papel de parede SHALL NOT manter animação contínua nem provocar quadros com a tela ociosa.

#### Scenario: Segundo plano
- **WHEN** outro app passa ao primeiro plano
- **THEN** a textura do papel de parede é liberada com o contexto gráfico, e nenhuma cópia da imagem fica na memória do processo

#### Scenario: Retorno
- **WHEN** o usuário pressiona Home com o lume em segundo plano
- **THEN** o papel de parede é recriado e aparece já no primeiro quadro completo

#### Scenario: Retorno sem custo novo
- **WHEN** o retorno a quente e a abertura a frio são medidos na TCL e na AIWA depois do change
- **THEN** cada um fica no máximo 10 ms acima da medida feita antes do change, dentro dos limites de `resource-efficiency`

#### Scenario: Tela ociosa
- **WHEN** a tela Início está visível e sem interação
- **THEN** o papel de parede não provoca nenhum quadro além do relógio

#### Scenario: Uma cópia no APK
- **WHEN** o APK de release é inspecionado
- **THEN** cada papel de parede aparece uma única vez, fora das bibliotecas nativas de cada arquitetura

### Requirement: Catálogo de papéis de parede de alta definição
O lume SHALL oferecer um catálogo de papéis de parede de alta definição embutidos, cada um guardado em 1920x1080 sem perda e com dither nos degradês escuros. O catálogo tem 6 papéis de parede, produzidos pelo autor em 3840x2160: a Aurora (o padrão do projeto), três paisagens de natureza noturna (montanhas estreladas, lago espelhado, floresta enevoada), uma fachada moderna de vidro e uma grade synthwave. Cada papel de parede SHALL possuir um identificador estável para persistência e um rótulo traduzido.

#### Scenario: Contraste do topo com os detalhes da imagem
- **WHEN** o papel de parede é mostrado sem escurecimento atrás da barra superior (a tela Início)
- **THEN** a imagem não leva sombra no topo nem na base, por decisão do autor, e o texto da barra superior pode ficar abaixo de 4,5:1 de contraste sobre os papéis mais claros

#### Scenario: Listagem de papéis de parede no editor
- **WHEN** o usuário abre o seletor de papéis de parede no modo edição
- **THEN** todas as opções do catálogo aparecem disponíveis com suas miniaturas de prévia e rótulos

#### Scenario: Identificador preservado
- **WHEN** um papel de parede existente tem a imagem trocada por uma versão nova
- **THEN** o identificador continua o mesmo, e a escolha gravada em `config.v1` continua valendo

### Requirement: Legibilidade sobre o papel de parede
Decisão do autor (2026-10-21): os papéis de parede SHALL NOT levar sombra assada no topo nem na base (o autor achou o efeito feio). O logo, as abas, a engrenagem e o relógio da barra superior, na tela Início, podem ficar abaixo de 4,5:1 de contraste sobre os papéis mais claros (por exemplo, montanhas estreladas, fachada de vidro, grade synthwave e lago espelhado); isso é uma escolha do autor, não uma pendência, e SHALL ser mudada só se o autor pedir. As telas de texto (Apps, Configurações, Sobre e modo edição) continuam escurecidas pelo requisito "Escurecimento nas telas de texto" e mantêm 4,5:1. O gerador oferece `--calibrate` para quem quiser a calibração antiga, e dois testes de contraste (`all_wallpapers_dark_top_and_bottom_for_ui` e `the_top_bar_texts_have_4_5_to_1_over_the_undimmed_wallpaper_on_home`) ficam com `#[ignore]` por causa desta decisão.

#### Scenario: Barra superior legível
- **WHEN** a parte mais clara de um papel de parede está sob a barra superior da tela Início
- **THEN** o contraste do texto da barra pode ser menor que 4,5:1, por decisão do autor, e as telas de texto continuam com pelo menos 4,5:1

### Requirement: Troca dinâmica de papel de parede
A seleção de papel de parede SHALL ser dinâmica: ao escolher um papel de parede no modo edição, a escolha SHALL ser persistida em `config.v1` na hora, e o fundo SHALL passar para a nova imagem assim que ela estiver pronta, sem travar a navegação. Enquanto a nova imagem é preparada, o papel anterior SHALL continuar na tela; a troca SHALL ser direta, sem transição translúcida de tela cheia.

#### Scenario: Troca de papel de parede no modo edição
- **WHEN** o usuário seleciona um novo papel de parede no modo edição
- **THEN** a preferência é gravada em `config.v1`, e o fundo passa para a nova imagem em até 300 ms na TCL

#### Scenario: Navegação durante a troca
- **WHEN** o usuário passa rapidamente por vários papéis de parede no seletor
- **THEN** nenhum quadro passa de 33 ms, e o fundo mostra o último escolhido quando a navegação para

#### Scenario: Inicialização com papel de parede salvo
- **WHEN** o lume é iniciado e existe um papel de parede válido configurado em `config.v1`
- **THEN** o papel de parede configurado é carregado e exibido atrás de todas as telas

#### Scenario: Padrão no primeiro uso ou configuração ausente
- **WHEN** o lume é aberto pela primeira vez ou o arquivo de configuração não especifica o papel de parede
- **THEN** o papel de parede padrão Aurora é utilizado automaticamente

## ADDED Requirements

### Requirement: Masters em alta resolução
Cada papel de parede do catálogo SHALL vir de um master com detalhe nativo de pelo menos 1920x1080 (o recomendado é 3840x2160), sem perda e com cor em resolução cheia, reduzido por média para 1920x1080. O projeto SHALL recusar um master menor que 1920x1080 ou que seja a ampliação de uma imagem menor.

#### Scenario: Master pequeno
- **WHEN** o gerador recebe um master de 1280x720
- **THEN** a geração falha e informa o arquivo e a resolução mínima

#### Scenario: Falso Full HD
- **WHEN** o gerador recebe um master de 1920x1080 que é a ampliação de uma imagem de 1280x720
- **THEN** a geração falha, porque reduzir o master a 1280x720 e ampliar de volta o reproduz quase sem diferença

#### Scenario: Master 4K
- **WHEN** o gerador recebe um master de 3840x2160 com detalhe real
- **THEN** a imagem do catálogo sai em 1920x1080, reduzida por média, com o dither aplicado só no fim

### Requirement: Origem das imagens
Cada imagem do catálogo SHALL ser gerada pelo projeto ou produzida pelo autor (os masters atuais saem dos scripts procedurais do autor em `tools/wallpaper/`), sem foto ou arte de terceiros, e SHALL ter a origem e a licença registradas na documentação do papel de parede.

#### Scenario: Imagem nova no catálogo
- **WHEN** um master novo entra no catálogo
- **THEN** a documentação do papel de parede registra quem o produziu, como e sob qual licença

### Requirement: Papel de parede pronto ao acordar
O lume SHALL guardar em cache, no armazenamento do app, a imagem já decodificada do papel de parede ativo (RGB cru em 1920x1080), de modo que, ao acordar, só leia o cache e a envie à GPU, sem decodificar de novo; a redução para um buffer menor, que custa poucos milissegundos de GPU, continua a ser feita a cada carga. O cache SHALL ser refeito quando o papel de parede, a versão da imagem ou o tamanho do buffer mudam, e a falta dele SHALL NOT impedir o papel de parede de aparecer.

#### Scenario: Retorno com cache
- **WHEN** o lume acorda com o mesmo papel de parede e a mesma versão da última vez
- **THEN** a imagem vem do cache, sem decodificação

#### Scenario: Troca invalida o cache
- **WHEN** o usuário escolhe outro papel de parede, ou o app é atualizado para outra versão
- **THEN** a imagem é decodificada de novo e o cache passa a guardar só a nova versão

#### Scenario: Cache apagado pelo sistema
- **WHEN** o Android apagou o cache do app e o lume acorda
- **THEN** o papel de parede é preparado a partir do APK e aparece no primeiro quadro completo, como sem cache
