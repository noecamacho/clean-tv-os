## MODIFIED Requirements

### Requirement: Origem das imagens
Cada imagem do catálogo SHALL ter a origem (quem a produziu ou de onde veio) e a licença registradas, por imagem, na documentação do papel de parede, e SHALL poder ser redistribuída sob os termos do projeto. O projeto SHALL NOT afirmar que uma imagem é gerada pelos scripts em `tools/wallpaper/` se esses scripts não a reproduzem.

#### Scenario: Imagem nova no catálogo
- **WHEN** um master novo entra no catálogo
- **THEN** a documentação do papel de parede registra quem o produziu ou de onde veio, como e sob qual licença

#### Scenario: Imagem de terceiros
- **WHEN** uma imagem do catálogo veio de uma fonte externa
- **THEN** a documentação registra o autor, o endereço de origem e a licença dessa imagem, e a licença permite a redistribuição

## ADDED Requirements

### Requirement: Masters em Git LFS
Os masters em `tools/wallpaper/masters/` SHALL ser guardados com Git LFS (`.gitattributes` rastreando `tools/wallpaper/masters/*.png`), e nenhum passo de build, de teste ou de CI SHALL depender do conteúdo deles: o APK usa só os arquivos em `assets/`.

#### Scenario: Checkout sem LFS
- **WHEN** o repositório é clonado ou o CI faz checkout sem baixar objetos LFS
- **THEN** os masters aparecem como ponteiros e `cargo test --workspace` e o build do APK funcionam sem alteração

#### Scenario: Master novo
- **WHEN** um PNG novo é adicionado em `tools/wallpaper/masters/`
- **THEN** o Git o guarda como objeto LFS, e não como blob no histórico
