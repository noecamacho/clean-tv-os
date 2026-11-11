# Tasks

## 1. Coleta de dados técnicos e strings

- [x] 1.1 Adicionar módulo de leitura estática de dados da TV em Rust (consultando NDK `__system_property_get` para `ro.product.*`, `ro.board.*`, `ro.build.*`, OpenGL para GPU/versão, `/proc/`, `uname` e `SIOCGIFCONF` para CPU/Kernel/IP; permissão `INTERNET` no manifesto)
- [x] 1.2 Adicionar chaves de texto para seções e rótulos da ficha técnica em `android/app/src/main/res/values/strings.xml` e `values-pt-rBR/strings.xml` (e demais idiomas ativos)
- [x] 1.3 Se necessário dados do framework Android (ex.: taxa de Hz de `Display` ou `isLowRamDevice`), adicionar comando leve na ponte `LumeBridge.java` e expor no Rust

## 2. Interface e navegação

- [x] 2.1 Adicionar `ITEM_TV_INFO` na lista de Configurações em `crates/lume-core/src/app/settings.rs` e nova variante `Screen::TvInfo` em `crates/lume-core/src/app/mod.rs`
- [x] 2.2 Implementar o desenho da tela `Informações da TV` em largura total (1152 px) na vertical, com máscara de corte, cabeçalho de navegação e rolagem suave via D-pad Cima/Baixo
- [x] 2.3 Garantir que o botão Voltar (Back / ESC) retorne à tela de Configurações mantendo o foco em `ITEM_TV_INFO`
- [x] 2.4 Atualizar os testes de régua de layout (`the_layout_is_on_the_ruler`) e testes de unidade em `crates/lume-core/src/app/tests.rs`

## 3. Verificação integrada

- [x] 3.1 Gerar capturas com `lume-desktop` da tela de Configurações (com 3 itens) e da tela `Informações da TV`, validando os alinhamentos e espaçamentos com `scripts/measure-ink.py`
- [x] 3.2 Validar na TV de referência (a TCL) e na segunda TV (a AIWA), conferindo a exibição de chipset, GPU, firmware e resolução em cada hardware — AIWA validada (mt5867, Mali-G31, firmware RTM7.230903.142, 1280 × 720, 60 Hz, Wi-Fi e IP); TCL não validada nesta change (aceito pelo autor).
- [x] 3.3 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume) e atualizá-la se a change trouxe algo novo: componente de terceiros (crate, fonte, ícones) e sua licença, endereço do repositório (https://github.com/douglaspands/lume-tv-launcher) ou versão; sem novidade, a tarefa registra "Sobre revisado, sem mudança". Verificar com `cargo run -p lume-desktop -- --screenshot about` — Sobre revisado, sem mudança (nenhum componente de terceiros, licença, endereço ou versão novo).
