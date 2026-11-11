# Proposal

## Why

Usuários de Smart TV (especialmente em modelos de marcas como TCL, AIWA, Xiaomi, Philco, e em TV boxes com processadores Amlogic, Realtek ou MediaTek) frequentemente não têm acesso fácil aos dados técnicos reais do seu aparelho. As telas nativas de configurações do Android espalham ou mascaram informações importantes, como o chipset real (SoC), arquitetura (32 vs 64 bits), renderizador gráfico (GPU), versão de firmware do fabricante e o endereço IP local na rede.

Oferecer uma tela dedicada de **Informações da TV** nas Configurações do Lume, disposta inteiramente na vertical com navegação suave pelo controle remoto, permite ao usuário conhecer a ficha técnica completa do seu aparelho de forma didática, sem custo de desempenho.

## What Changes

- **Novo item nas Configurações (`ITEM_TV_INFO`):**
  - Adicionado na coluna de itens da tela de Configurações, entre "Tela Início" e "Sobre o Lume", com o rótulo "Informações da TV" ("TV Information").
- **Tela secundária `Informações da TV` (`Screen::TvInfo`):**
  - Cabeçalho de navegação padrão: breadcrumb `Configurações › Informações da TV` e título `Informações da TV`.
  - Disposição 100% vertical na largura útil de 1152 px, com máscara de corte e rolagem suave via D-pad (`Down` e `Up`), sem foco em itens internos (apenas leitura).
  - Seções estruturadas:
    1. **Aparelho:** Marca, Modelo, Fabricante, Dispositivo/Placa.
    2. **Processamento & Gráficos:** Processador (SoC), CPU, Núcleos, Arquitetura (ABI), GPU e Versão OpenGL ES.
    3. **Memória & Armazenamento:** Memória RAM total, Perfil de memória (Padrão ou Low-RAM), Armazenamento interno total e disponível.
    4. **Sistema & Firmware:** Versão do Android, Nível de API, Versão de firmware da TV (build do fabricante), Patch de segurança e Versão do kernel Linux.
    5. **Tela & Exibição:** Resolução da interface, Taxa de atualização (Hz) e Densidade de tela (DPI).
    6. **Rede & Conexão:** Tipo de conexão ativa (Wi-Fi ou Cabo Ethernet) e Endereço IP local.
- **Coleta estática e eficiente (Rust-First):**
  - GPU e versão do OpenGL são consultadas diretamente em Rust (`glGetString`).
  - CPU e memória total são lidos em Rust de `/proc/cpuinfo` e `/proc/meminfo`; o kernel vem de `uname` (o `/proc/version` é bloqueado para apps desde o Android 10).
  - Resolução da interface, taxa de atualização e densidade vêm de um comando `display` do `LumeBridge.java` (única parte em Java, autorizada pelo autor: não há equivalente no NDK).
  - Tipo de conexão e IP vêm de `SIOCGIFCONF` em Rust; no Android 11 só funciona com a permissão `INTERNET` (normal, concedida na instalação), que o Lume passa a declarar sem abrir conexões.
  - Propriedades de hardware e sistema (SoC, modelo, build do fabricante) são consultadas via NDK (`__system_property_get`) sem custo.
  - Leitura estática única ao entrar na tela: sem polling periódico em repouso, zero quadros perdidos na navegação.

## Capabilities

### Modified Capabilities

- `device-stats`: expande a cobertura de dados da TV para além das estatísticas dinâmicas (memória/disco/cpu), definindo o contrato de informações técnicas estáticas do aparelho.
- `design-system`: especifica o item "Informações da TV" na tela de Configurações e o layout da tela secundária vertical com rolagem via D-pad na régua de 4 px.

## Impact

- Rust:
  - `crates/lume-core/src/app/settings.rs`: novo item `ITEM_TV_INFO`, estado de rolagem e desenho da tela vertical de informações.
  - `crates/lume-core/src/stats.rs` (ou novo módulo `device_info.rs`): structs e rotinas de leitura de dados estáticos da TV.
  - `crates/lume-core/src/app/mod.rs`: nova variante `Screen::TvInfo`.
- Recursos: novas chaves de tradução em `values/strings.xml` e `values-pt-rBR/strings.xml` (e nos novos idiomas da change de localização).
- Uma permissão nova no Android: `INTERNET` (normal, sem diálogo), só para listar as interfaces de rede; o Lume não abre conexões.
- Java: comando `display` em `LumeBridge.java`.
