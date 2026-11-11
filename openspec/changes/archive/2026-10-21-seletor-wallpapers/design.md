# Design

## Context

O Lume possui atualmente um único papel de parede estático obtido via `crate::wallpaper::current()`, embutido durante a compilação a partir de `aurora-640x360.rgb` (hoje os papéis são 1920x1080: ver `proposal.md`, "Qualidade para TVs HD a 4K"). O renderer OpenGL ES 2.0 (`gles2.rs`) faz o upload dessa textura uma única vez em `ensure_wallpaper()`.
No modo edição (`editor.rs`), há uma linha para alternar "Favoritos redondos", um painel de gerenciamento de favoritos e dois botões no rodapé: "Restaurar padrão" (à esquerda) e "Concluído" (à direita). Como toda mutação emite `Effect::SaveConfig` imediatamente, o botão "Concluído" é redundante.

## Goals / Non-Goals

**Goals:**
- Prover um catálogo de 10 papéis de parede elegantes e calibrados para contraste mínimo de 4.5:1 nas áreas de interface crítica.
- Manter o tempo de decodificação na CPU em 0 ms e zero alocações adicionais no heap, usando formato Raw 24-bit RGB linear embutido.
- Manter o consumo de VRAM estritamente dentro do orçamento (máximo 1 textura ativa de wallpaper na GPU, ~675 KB a 1 MB).
- Implementar seletor por Mini-Cards (178x100) acima do painel de favoritos no modo edição, com navegação suave por D-Pad.
- Eliminar o botão redundante "Concluído", adaptando o grafo de foco do D-Pad para navegar diretamente ao botão "Restaurar padrão" ao descer a partir dos apps.
- Persistir o identificador do papel de parede em `config.v1` (`wallpaper=<id>`), garantindo retrocompatibilidade.

**Non-Goals:**
- Não permitir importação de papéis de parede externos pelo usuário (USB ou download) nesta versão.
- Não manter animações contínuas de fundo com a tela ociosa, preservando a eficiência energética e os ciclos de GPU.

## Decisions

### 1. Formato de Imagem: Raw 24-bit RGB Linear Embutido
- **Decisão**: Utilizar buffers de bytes brutos RGB (`&'static [u8]`) para os 10 papéis de parede, gerados em build time a partir dos masters de alta definição.
- **Alternativas consideradas**:
  - *PNG / WebP no runtime*: Exigiria crate de decodificação, alocaria de 4 a 8 MB de heap e consumiria 30 a 60 ms de CPU da TV ao inicializar ou trocar.
  - *JPEG*: Provoca artefatos severos de *color banding* e macroblocos em gradientes escuros sutis.
- **Vantagem**: 0 ms de decodificação na CPU, zero banding e o próprio APK (arquivo ZIP Deflate) comprime os 10 buffers sem perdas para cerca de ~150-180 KB cada no pacote final.

### 2. Catálogo e Identificadores Estáveis
- **Decisão**: Criar um enum `WallpaperId` com 10 variantes:
  - `Aurora` (`"aurora"`) — Clássica, 1280x720 com dithering
  - `MistyForest` (`"misty_forest"`)
  - `MidnightOcean` (`"midnight_ocean"`)
  - `ObsidianCanyon` (`"obsidian_canyon"`)
  - `SilkWave` (`"silk_wave"`)
  - `PrismaticTopo` (`"prismatic_topo"`)
  - `MirrorLake` (`"mirror_lake"`)
  - `BlackDunes` (`"black_dunes"`)
  - `GlacialMist` (`"glacial_mist"`)
  - `StardustEther` (`"stardust_ether"`)
- Cada variante mapeia para seu asset RGB, dimensões, estrelas (quando aplicável) e chave de tradução `StringId`.

### 3. Atualização Dinâmica da Textura na GPU
- **Decisão**: Em `crates/lume-core/src/render/gles2.rs`, rastrear o ID do wallpaper atualmente carregado na GPU. Se o wallpaper ativo mudar (ex: ao selecionar no editor), `ensure_wallpaper()` recria ou atualiza a textura com `glTexImage2D`, liberando ou sobrescrevendo a anterior. A memória gráfica alocada continua sendo de exatamente 1 textura ativa.

### 4. Layout e Foco dos Mini-Cards no Modo Edição
- **Decisão**: Inserir a linha de papéis de parede entre "Favoritos redondos" e o painel de favoritos.
  - Cada mini-card mede 178x100 px, com borda arredondada de 12 px (`--radius`) e gap de 16 px.
  - O D-Pad Esquerda/Direita percorre os mini-cards com rolagem horizontal caso necessário.
  - Pressionar OK atualiza `config.wallpaper`, emite `Effect::SaveConfig` e reflete o novo fundo no mesmo quadro.
  - Pressionar Baixo transfere o foco para o primeiro card de favorito no painel abaixo.

### 5. Remoção do Botão Concluído e Reestruturação do Foco Inferior
- **Decisão**: O ID `DONE` (4000) é removido. No rodapé, resta apenas `RESTORE` (4001) à esquerda.
  - Ao pressionar Baixo na última linha de chips de adicionar app, o foco direciona-se diretamente para `RESTORE`.
  - Ao pressionar Cima a partir de `RESTORE`, o foco retorna para a última linha de chips.
  - O usuário sai da tela pressionando o botão físico Voltar do controle remoto (`back_from_editor`).

## Risks / Trade-offs

- **[Tamanho do APK com 10 imagens]** → Cada imagem raw de gradiente suave comprime no ZIP do APK para ~150 KB. 10 papéis de parede somam cerca de 1.5 MB no APK, mantendo o APK do Lume em ~4.5 MB, bem abaixo do limite orçamentário de 6 MB da spec `resource-efficiency`.
- **[Troca de textura na GPU durante navegação]** → O upload de uma textura de 675 KB via `glTexImage2D` leva menos de 0.5 ms no barramento da TV, não provocando perda de quadros perceptível ao alternar.
