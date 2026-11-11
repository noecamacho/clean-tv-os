# resource-efficiency Specification

## Purpose
Torna mensurável o requisito central do lume, que é consumir o mínimo de memória e de processamento em TVs com 2 GB de RAM, sobretudo quando outro app (ex.: um player de vídeo) está em primeiro plano.

## Requirements

### Requirement: Orçamento de memória em primeiro plano
Com o lume visível e ocioso na tela Início, o TOTAL PSS do processo (medido por `dumpsys meminfo`, incluindo Graphics/GL) SHALL ser de no máximo 64 MB no dispositivo de referência (TCL 55C715) com renderização em 1080p e 30 apps instalados. O limite subiu de 45 MB porque o papel de parede passou a ter 1920x1080 (textura de 6,2 MB, contra 0,7 MB) e o catálogo de 10 papéis.

#### Scenario: Medição em primeiro plano
- **WHEN** o lume está na tela Início há 30 segundos sem interação
- **THEN** `adb shell dumpsys meminfo <pacote>` reporta TOTAL PSS ≤ 64 MB

### Requirement: Orçamento de memória em segundo plano
Quando outro app passa ao primeiro plano, o lume SHALL liberar superfície, contexto gráfico, texturas e atlas, e o TOTAL PSS do processo SHALL cair para no máximo 20 MB em até 10 segundos no dispositivo de referência (ou a memória privada `Private Dirty` SHALL cair para no máximo 10 MB em dispositivos com `ro.config.low_ram=true`, onde páginas compartilhadas do sistema Android inflam o PSS).

#### Scenario: Assistindo a outro app
- **WHEN** o usuário abre o YouTube pelo lume e espera 10 segundos
- **THEN** `dumpsys meminfo` do lume reporta TOTAL PSS ≤ 20 MB no dispositivo de referência (ou `Private Dirty` ≤ 10 MB em dispositivo `low_ram`), sem memória de GL/Graphics atribuída

#### Scenario: Pressão de memória
- **WHEN** o sistema envia um aviso de pouca memória (trim memory) com o lume em segundo plano
- **THEN** o lume libera todos os caches recriáveis, mantendo só a configuração e o catálogo em memória

### Requirement: Retorno rápido
Ao pressionar Home, a parte do retorno sob controle do Lume (a partir da entrega da janela nativa pelo sistema até o primeiro quadro do splash aparecer e o Lume estar respondendo ao controle) SHALL levar no máximo 150 ms quando o processo do lume ainda existe (retorno a quente) e no máximo 800 ms de ponta a ponta quando o processo foi encerrado (abertura a frio), no dispositivo de referência. A interface completa (fim da revelação do splash, ver `launcher-shell`) SHALL estar na tela no máximo 600 ms mais a duração da revelação depois do primeiro quadro. O Lume SHALL registrar no log o tempo do primeiro quadro e o da interface pronta.

#### Scenario: Retorno a quente
- **WHEN** o lume está em segundo plano, com recursos gráficos liberados, o usuário pressiona Home e o sistema entrega a janela nativa
- **THEN** o primeiro quadro (o splash) aparece em até 150 ms após a criação da janela, e a tela Início completa em até 960 ms depois dele

#### Scenario: Abertura a frio
- **WHEN** o processo do lume foi encerrado e o usuário pressiona Home
- **THEN** o primeiro quadro aparece em até 800 ms, usando o catálogo em cache sem esperar pela consulta completa de apps, e a tela Início completa em até 1200 ms depois dele

#### Scenario: Log da interface pronta
- **WHEN** a revelação do splash termina
- **THEN** o log do Lume registra o tempo desde a janela e desde o início do processo

### Requirement: Renderização sob demanda
O lume SHALL desenhar quadros apenas quando algo visível muda (entrada do usuário, animação em curso, virada de minuto do relógio, mudança no catálogo). Ocioso, SHALL NOT desenhar quadros nem manter threads acordadas em laço. O papel de parede e o dock SHALL NOT manter animação contínua.

#### Scenario: Tela ociosa
- **WHEN** a tela Início está visível, sem interação
- **THEN** o lume desenha no máximo 1 quadro por minuto (relógio), e o uso de CPU do processo fica abaixo de 1% em média

#### Scenario: Outras telas ociosas
- **WHEN** a aba Apps, as Configurações, a tela Sobre ou o editor estão visíveis, sem interação
- **THEN** o lume desenha no máximo 1 quadro por minuto (relógio)

#### Scenario: Magnification terminada
- **WHEN** a animação de Magnification do dock termina
- **THEN** o lume deixa de desenhar quadros até a próxima mudança visível

### Requirement: Fluidez da navegação
Animações de foco, rolagem e Magnification do dock SHALL rodar a 60 quadros por segundo no dispositivo de referência, com no máximo 1% de quadros perdidos durante a navegação contínua, mesmo com o papel de parede desenhado atrás do conteúdo. Na grade de Apps, o desenho dos tiles SHALL agrupar todas as caixas, todas as imagens e todos os textos em chamadas de desenho contíguas (sem partir o lote de texto no meio por causa do tile do próprio launcher), e as telas com rolagem (grade de Apps, editor, informações da TV e Sobre) SHALL descartar o desenho de itens fora da área visível (`viewport culling`).

#### Scenario: Rolagem da linha de favoritos
- **WHEN** o usuário mantém a seta Direita pressionada sobre os 12 favoritos do dock
- **THEN** o `dumpsys gfxinfo`/medição de quadros mostra ≤ 1% de quadros perdidos

#### Scenario: Rolagem da grade de Apps
- **WHEN** o usuário mantém a seta Baixo pressionada na grade de Apps com 30 apps
- **THEN** a medição de quadros mostra ≤ 1% de quadros perdidos

#### Scenario: Lote contíguo de desenho na grade de Apps
- **WHEN** a grade de Apps é desenhada com apps antes e depois do tile "Lume"
- **THEN** o lote de texto dos tiles não é interrompido por comandos de retângulo do logo do tile "Lume"

### Requirement: Boas práticas de loop engineering (engenharia de laços)
Caminhos quentes de layout e renderização por quadro SHALL NOT realizar alocações dinâmicas na heap. Algoritmos de elipsização de texto SHALL operar em passo único sem strings intermediárias. Coleções com rolagem SHALL aplicar descarte antecipado de itens fora da janela visível (viewport culling) antes de medições e despacho. Varreduras de catálogo SHALL suportar interrupção antecipada (short-circuit) com cota e deduplicação de chaves.

#### Scenario: Zero alocações em laço quente de texto da tela Sobre
- **WHEN** a tela Sobre é desenhada a cada quadro exibindo 340+ linhas de licença
- **THEN** as quebras de linha e textos de licença vêm de cache estático pré-computado, com zero alocações na heap por quadro de renderização

#### Scenario: Elipsização em passo único
- **WHEN** um título ou rótulo longo de aplicativo necessita de elipsização com reticências
- **THEN** a largura e o ponto de corte são calculados em passo único em tempo linear sem alocar strings intermediárias durante a busca da quebra

#### Scenario: Culling vertical em laços de renderização de listas e grades
- **WHEN** uma lista ou grade com dezenas de itens sofre rolagem vertical
- **THEN** itens com limites superiores ou inferiores completamente externos à janela visível são descartados antes de qualquer cálculo de texto ou despacho para o renderizador

#### Scenario: Interrupção antecipada em varredura de catálogo
- **WHEN** o launcher solicita um lote restrito de ícones ou aplicativos prioritários
- **THEN** o laço de varredura interrompe o processamento imediatamente ao atingir o limite estipulado, sem percorrer nem calcular hashes para o restante da coleção

### Requirement: Boas práticas de graph engineering (grafo de renderização e recursos GPU)
O grafo de renderização SHALL agrupar primitivas contíguas do mesmo tipo e textura para minimizar trocas de programa e chamadas de desenho na GPU. Texturas de alta resolução SHALL ter upload fatiado no tempo em linhas por quadro. VBOs de redução e buffers frequentes SHALL ser reutilizados na memória de vídeo, operações de leitura de pixels da GPU SHALL inverter linhas in-place e atlas de texturas SHALL reciclar slots via lista de reuso.

#### Scenario: Batching contíguo de comandos no grafo de renderização
- **WHEN** uma tela composta por múltiplos elementos visuais (fundos, ícones e textos) é desenhada
- **THEN** o grafo de renderização consolida os comandos emitindo primeiramente todos os retângulos, em seguida todas as texturas e por fim todos os glifos de texto, evitando alternância repetida entre programas de shader

#### Scenario: Upload particionado de papel de parede no tempo
- **WHEN** uma nova imagem de papel de parede é enviada à GPU
- **THEN** as linhas de pixels são transferidas em fatias sucessivas de no máximo 90 linhas por quadro, distribuindo o upload ao longo de múltiplos quadros para não bloquear a thread principal

#### Scenario: Reutilização de buffers e locations de shader
- **WHEN** operações de redução de escala de imagem ou desenho em lote ocorrem a cada quadro
- **THEN** os VBOs compartilhados e os uniform locations cacheados são reutilizados diretamente sem invocar chamadas de criação de buffer ou busca dinâmica de uniforms na GPU

#### Scenario: Reciclagem de slots no atlas de texturas
- **WHEN** um ícone de aplicativo tem seu conteúdo liberado ou atualizado no atlas
- **THEN** seu slot de coordenadas é adicionado à lista de reuso para alocação subsequente, sem necessidade de reconstrução ou redimensionamento do atlas

### Requirement: Tamanho do pacote
O APK de release SHALL ter no máximo 40 MB, contendo as duas arquiteturas suportadas. Os papéis de parede de alta definição SHALL entrar no APK uma única vez, fora das bibliotecas nativas de cada arquitetura, para que o limite não cresça com o número de arquiteturas.

#### Scenario: Verificação no CI
- **WHEN** o CI gera o APK de release
- **THEN** o build falha se o arquivo passar de 40 MB

#### Scenario: Papéis de parede fora das bibliotecas nativas
- **WHEN** o APK de release é verificado pelo script do projeto
- **THEN** a verificação falha se um papel de parede aparecer dentro de uma biblioteca nativa ou mais de uma vez no pacote
