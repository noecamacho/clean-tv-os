# Tasks

## 1. Leitura e formatação do Clock da CPU em Rust (`lume-core` e `lume-app`)

- [x] 1.1 Em `crates/lume-core/src/stats.rs`, adicionar a função `pub fn format_freq(khz: u64, comma: bool) -> String` que formata valores `>= 1_000_000` kHz em GHz com duas casas decimais (`"{ghz:.2} GHz"`, trocando `.` por `,` quando `comma` for `true`) e valores `< 1_000_000` kHz em MHz arredondados sem casas decimais (`"{mhz} MHz"`). Adicionar testes unitários em `crates/lume-core/src/stats.rs` cobrindo `format_freq(1_350_000, true) == "1,35 GHz"`, `format_freq(1_500_000, false) == "1.50 GHz"`, `format_freq(1_000_000, true) == "1,00 GHz"` e `format_freq(750_000, true) == "750 MHz"`. Verificar com `cargo test -p lume-core stats -- --nocapture`.
- [x] 1.2 Em `crates/lume-core/src/stats.rs`, adicionar as funções `pub fn read_cpu_freq_from_dir(base: &std::path::Path) -> Option<CpuReading>` e `pub fn read_sysfs_cpu_freq() -> Option<CpuReading>`. `read_cpu_freq_from_dir` deve iterar `cpu0`..`cpu15` dentro de `base`, ler `cpufreq/scaling_cur_freq` (ou fallback `cpufreq/cpuinfo_cur_freq`) e `cpufreq/cpuinfo_max_freq` (ou fallback `cpufreq/scaling_max_freq`), ignorar núcleos com valores `0` ou ausentes, e retornar `Some(CpuReading::Freq { current: sum_cur / count, max: sum_max / count })` quando `count > 0`. Adicionar teste unitário em `crates/lume-core/src/stats.rs` criando um diretório temporário com `cpu0/cpufreq` e `cpu1/cpufreq` e validando a média retornada por `read_cpu_freq_from_dir`. Verificar com `cargo test -p lume-core stats -- --nocapture`.
- [x] 1.3 Em `crates/lume-app/src/android/bridge.rs`, atualizar `Bridge::stats(&self) -> Option<Stats>` para `let mut stats = Stats::parse(&self.call("stats", "")?)?; if let Some(freq) = lume_core::stats::read_sysfs_cpu_freq() { stats.cpu = freq; } Some(stats)`. Garantir que nenhum arquivo `.java` foi modificado (`git status --porcelain android/app/src/main/java`). Verificar compilação com `cargo check --workspace`.

## 2. Exibição de "Clock da CPU" no painel "Status da TV", traduções e modo demo

- [x] 2.1 Em `crates/lume-core/src/app/settings.rs`, na função `draw_stats`, atualizar a construção de `cpu_val`: quando `stats.map(|s| s.cpu)` for `Some(stats::CpuReading::Freq { current, max })` com `max > 0`, formatar como `format!("{} / {}", stats::format_freq(current, comma), stats::format_freq(max, comma))`; caso contrário, manter o fallback `self.cpu_use.map(|p| format!("{:.0}%", p.clamp(0.0, 100.0))).unwrap_or_else(|| "—".to_string())`. Manter a barra horizontal proporcional usando `self.cpu_use.unwrap_or(0.0) / 100.0`. Verificar com `cargo test -p lume-core`.
- [x] 2.2 Atualizar `<string name="stats_cpu">` nos 5 arquivos de tradução do Android e sincronizar o espelho do `lume-desktop`:
  - `android/app/src/main/res/values-pt-rBR/strings.xml`: `<string name="stats_cpu">Clock da CPU</string>`
  - `android/app/src/main/res/values/strings.xml`: `<string name="stats_cpu">CPU clock</string>`
  - `android/app/src/main/res/values-es/strings.xml`: `<string name="stats_cpu">Reloj de CPU</string>`
  - `android/app/src/main/res/values-id/strings.xml`: `<string name="stats_cpu">Clock CPU</string>`
  - `android/app/src/main/res/values-ru/strings.xml`: `<string name="stats_cpu">Частота ЦП</string>`
  Verificar com `cargo test -p lume-desktop` que todas as chaves de tradução continuam válidas e completas nos 5 idiomas.
- [x] 2.3 Em `tools/lume-desktop/src/demo.rs`, na função `feed_stats`, atualizar `CpuReading::Freq { current: 3_600_000, max: 8_000_000 }` para `CpuReading::Freq { current: 1_350_000, max: 1_500_000 }`. Gerar captura da tela de configurações com `cargo run -p lume-desktop -- --size 1920x1080 --locale pt-BR --screenshot settings /tmp/lume-settings-check.png` e verificar que o comando executa sem erros.

## 3. Alinhamento da documentação de tema e comentários do CSS

- [x] 3.1 Em `docs/theme-format.md`, no bloco de exemplo de `.dock-label` (próximo à linha 101), atualizar `background: rgba(15, 23, 30, 0.9);` para `background: rgba(0, 0, 0, 0);` para refletir `themes/lume/theme.css`. Verificar com `grep -n -A 6 "\.dock-label" docs/theme-format.md`.
- [x] 3.2 Em `themes/lume/theme.css`, na linha 16 (`--font-primary: "Plus Jakarta Sans"; /* UI, body, titles, clock */`), remover `clock` do comentário (deixando `/* UI, body, titles, labels */`), pois o `.clock` usa `--font-brand` ("Outfit"). Verificar com `cargo test -p lume-core theme` que os testes de tema continuam passando.

## 4. Sincronização das especificações principais (`openspec/specs/`) com as delta specs

- [x] 4.1 Sincronizar as 6 delta specs de `openspec/changes/alinhar-design-system-e-clock-cpu/specs/` (`design-system`, `device-stats`, `home-editor`, `theming`, `wallpaper`, `launcher-shell`) para `openspec/specs/` e validar com `openspec validate --specs --strict`.

## 5. Verificação de integração, APK e revisão da tela Sobre

- [x] 5.1 Rodar `cargo test --workspace && cargo clippy --workspace --all-targets && cargo fmt --all -- --check` e verificar que todos os testes passam sem warnings ou diferenças de formatação.
- [x] 5.2 Se a AIWA ou a TCL estiver conectada via `adb`, gerar o APK com `scripts/container-build.sh release`, validar com `uv run scripts/verify-apk.py dist/lume-*-release-debugsigned.apk`, instalar na TV e verificar que a tela de Configurações exibe "Clock da CPU" e `<atual> / <máximo>` (ex.: `1,35 GHz / 1,50 GHz`). Caso nenhuma TV esteja conectada na sessão, verificar no emulador (`scripts/emulator.sh`) ou via captura do `lume-desktop`.
- [x] 5.3 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume, em `crates/lume-core/src/app/settings.rs` e `android/app/src/main/res/values*/strings.xml`) para confirmar se a change trouxe algum novo componente de terceiros, licença ou mudança de repositório; como não há dependências novas, registrar "Sobre revisado, sem mudança".

## Workflow follow-up

- Arquivar a change com `/opsx:archive` após revisão e validação do autor.
