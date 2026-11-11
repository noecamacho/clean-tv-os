## Contexto

Esta change reúne duas frentes complementares em um único pacote planejado para execução direta e sem ambiguidades:

1. **Alinhamento das especificações e documentos com o código real do Lume**:
   Durante as evoluções recentes (`v0.2.0`, `v0.2.1`, `escala-tipografica`, `menu-de-contexto`, `splash`, `wallpapers-full-hd`), diversas telas em `crates/lume-core/src/app/*.rs` e seletores em `themes/lume/theme.css` evoluíram, mas trechos de `openspec/specs/` e `docs/theme-format.md` mantiveram valores antigos. O app construído é a fonte da verdade visual e comportamental; a documentação deve refletir exatamente as constantes e fluxos do código.

2. **Substituição de "CPU" (%) por "Clock da CPU" (`<atual> / <máximo>`) no painel "Status da TV"**:
   Em Android 8+ (API 26+), o SELinux bloqueia a leitura de `/proc/stat` por apps comuns (`EACCES`). Quando isso ocorre, a leitura cai para a frequência de clock em `/sys/devices/system/cpu/cpu*/cpufreq/`. Em TVs como a AIWA (Android 9, 4 núcleos Cortex-A53 até 1,50 GHz), o governador mantém o clock mínimo em `1,10 GHz` (73,3%) e o clock em repouso em `1,35 GHz` (90%), embora o uso real de CPU seja inferior a 10%. Mostrar `"90%"` sob o rótulo `"CPU"` dá a falsa impressão de processador sobrecarregado. Exibir **"Clock da CPU"** com `<atual> / <máximo>` (ex.: `1,35 GHz / 1,50 GHz`) e a barra proporcional (`current / max`) comunica com precisão e transparência o dado real do hardware em todas as TVs.

## Objetivos / Não-Objetivos

**Objetivos:**
- Atualizar `openspec/specs/design-system/spec.md`, `openspec/specs/home-editor/spec.md`, `openspec/specs/theming/spec.md`, `openspec/specs/wallpaper/spec.md`, `openspec/specs/launcher-shell/spec.md` e `openspec/specs/device-stats/spec.md` (via sync das delta specs desta change) para refletirem 100% do código atual.
- Corrigir `docs/theme-format.md` (exemplo de `.dock-label` com fundo transparente) e o comentário de `--font-primary` em `themes/lume/theme.css`.
- Ler a frequência média por núcleo (`scaling_cur_freq` e `cpuinfo_max_freq`) diretamente em Rust a partir de `/sys/devices/system/cpu/cpu*/cpufreq/`, sem tocar em código Java.
- Formatar frequências em kHz como `X,XX GHz` (ou `X.XX GHz` conforme locale) quando `>= 1_000_000 kHz` e `XXX MHz` quando `< 1_000_000 kHz`.
- Exibir `<atual> / <máximo>` na primeira linha do painel "Status da TV" quando a leitura de CPU for `CpuReading::Freq { current, max }`, mantendo a barra proporcional a `current / max`.
- Atualizar a tradução de `stats_cpu` nos 5 arquivos `strings.xml` (`pt-rBR`, `en`, `es`, `id`, `ru`) e nos dados de demonstração do `lume-desktop`.

**Não-Objetivos:**
- Alterar qualquer arquivo `.java` (`LumeBridge.java` ou `LumeActivity.java`). A regra do projeto exige construção 100% Rust sempre que possível, e o Rust tem acesso direto de leitura ao `sysfs` (`/sys/devices/system/cpu/...`) no Android sem precisar de JNI.
- Alterar o layout visual, medidas ou cores de qualquer tela além do texto e rótulo da linha de clock da CPU no painel "Status da TV".

## Decisões Técnicas

### Decisão 1: Leitura de `cpufreq` 100% em Rust (`crates/lume-core/src/stats.rs` e `crates/lume-app/src/android/bridge.rs`)

- **Por quê**: `LumeBridge.java` hoje soma `scaling_cur_freq` e `cpuinfo_max_freq` de todos os núcleos (ex.: 4 núcleos a `1_350_000 kHz` resultam em `current = 5_400_000` e `max = 6_000_000`). Para exibir a frequência por núcleo (`1,35 GHz / 1,50 GHz`) sem modificar `.java`, fazemos a leitura diretamente em Rust no `sysfs` e sobrescrevemos `stats.cpu` em `Bridge::stats`.
- **Implementação exata**:
  1. Em `crates/lume-core/src/stats.rs`, adicionar duas funções públicas:
     - `pub fn read_cpu_freq_from_dir(base: &std::path::Path) -> Option<CpuReading>`:
       Itera `i` de `0..16`, verificando o diretório `base.join(format!("cpu{i}/cpufreq"))`.
       Para cada núcleo:
       - Lê `cur` tentando primeiro `scaling_cur_freq` e, se ausente ou inválido, `cpuinfo_cur_freq` (fazendo `.trim().parse::<u64>()`).
       - Lê `max` tentando primeiro `cpuinfo_max_freq` e, se ausente ou inválido, `scaling_max_freq` (fazendo `.trim().parse::<u64>()`).
       - Se `cur > 0 && max > 0`, acumula `sum_cur += cur`, `sum_max += max` e `count += 1`.
       - Ao final do laço, se `count > 0`, retorna `Some(CpuReading::Freq { current: sum_cur / count, max: sum_max / count })`, caso contrário `None`.
     - `pub fn read_sysfs_cpu_freq() -> Option<CpuReading>`:
       Chama `read_cpu_freq_from_dir(std::path::Path::new("/sys/devices/system/cpu"))`.
  2. Em `crates/lume-app/src/android/bridge.rs`, no método `pub fn stats(&self) -> Option<Stats>`:
     ```rust
     pub fn stats(&self) -> Option<Stats> {
         let mut stats = Stats::parse(&self.call("stats", "")?)?;
         if let Some(freq) = lume_core::stats::read_sysfs_cpu_freq() {
             stats.cpu = freq;
         }
         Some(stats)
     }
     ```
     Se por algum motivo o `sysfs` não puder ser lido diretamente pelo Rust, o valor de fallback retornado pelo `LumeBridge.java` continua funcionando sem quebrar.

### Decisão 2: Formatação de frequência (`format_freq` em `crates/lume-core/src/stats.rs`)

- **Implementação exata**:
  Adicionar em `crates/lume-core/src/stats.rs`:
  ```rust
  /// Formats a CPU frequency in kHz as GHz (2 decimal places when >= 1_000_000 kHz)
  /// or MHz (rounded integer when < 1_000_000 kHz).
  pub fn format_freq(khz: u64, comma: bool) -> String {
      if khz >= 1_000_000 {
          let ghz = khz as f64 / 1_000_000.0;
          let s = format!("{ghz:.2} GHz");
          if comma {
              s.replace('.', ",")
          } else {
              s
          }
      } else {
          let mhz = (khz as f64 / 1_000.0).round() as u64;
          format!("{mhz} MHz")
      }
  }
  ```
- **Exemplos verificados em testes unitários**:
  - `format_freq(1_350_000, true)` -> `"1,35 GHz"`
  - `format_freq(1_500_000, false)` -> `"1.50 GHz"`
  - `format_freq(750_000, true)` -> `"750 MHz"`
  - `format_freq(1_000_000, true)` -> `"1,00 GHz"`

### Decisão 3: Exibição no painel "Status da TV" (`crates/lume-core/src/app/settings.rs`)

- **Implementação exata**:
  Em `crates/lume-core/src/app/settings.rs`, na função `draw_stats`:
  - Atualmente:
    ```rust
    let cpu_val = self
        .cpu_use
        .map(|p| format!("{:.0}%", p.clamp(0.0, 100.0)))
        .unwrap_or_else(|| "—".to_string());
    ```
  - Substituir por:
    ```rust
    let cpu_val = match stats.map(|s| s.cpu) {
        Some(stats::CpuReading::Freq { current, max }) if max > 0 => {
            format!(
                "{} / {}",
                stats::format_freq(current, comma),
                stats::format_freq(max, comma)
            )
        }
        _ => self
            .cpu_use
            .map(|p| format!("{:.0}%", p.clamp(0.0, 100.0)))
            .unwrap_or_else(|| "—".to_string()),
    };
    ```
  - O cálculo de `self.cpu_use` em `Shell::set_stats` (`crates/lume-core/src/app/mod.rs`) permanece inalterado, pois `stats::cpu_percent(None, stats.cpu)` já calcula `(current as f32 / max as f32 * 100.0).clamp(0.0, 100.0)` para alimentar a barra horizontal proporcional (`self.cpu_use.unwrap_or(0.0) / 100.0`).

### Decisão 4: Rótulos traduzidos e dados de demonstração

- Nos 5 arquivos `android/app/src/main/res/values*/strings.xml`, atualizar a chave `stats_cpu`:
  - `values/strings.xml`: `<string name="stats_cpu">CPU clock</string>`
  - `values-pt-rBR/strings.xml`: `<string name="stats_cpu">Clock da CPU</string>`
  - `values-es/strings.xml`: `<string name="stats_cpu">Reloj de CPU</string>`
  - `values-id/strings.xml`: `<string name="stats_cpu">Clock CPU</string>`
  - `values-ru/strings.xml`: `<string name="stats_cpu">Частота ЦП</string>`
- Em `tools/lume-desktop/src/demo.rs`:
  - Na função `feed_stats`, alterar `CpuReading::Freq { current: 3_600_000, max: 8_000_000 }` para frequências por núcleo realistas: `CpuReading::Freq { current: 1_350_000, max: 1_500_000 }`.

### Decisão 5: Sincronização das especificações (`openspec/specs/`) e documentação (`docs/`, `themes/`)

- Sincronizar todas as 6 delta specs desta change (`design-system`, `device-stats`, `home-editor`, `theming`, `wallpaper`, `launcher-shell`) com os arquivos principais em `openspec/specs/<capacidade>/spec.md`.
- Em `docs/theme-format.md`, atualizar o bloco `.dock-label` de `background: rgba(15, 23, 30, 0.9);` para `background: rgba(0, 0, 0, 0);`.
- Em `themes/lume/theme.css`, atualizar o comentário de `--font-primary` na linha 15 de `/* Font families (brand = Outfit for logo/clock/headings; primary = Plus Jakarta Sans for UI/body) */` para `/* Font families (brand = Outfit for logo/clock; primary = Plus Jakarta Sans for UI/body/headings) */` (ou remover `clock` da descrição de `primary` na linha 16: `--font-primary: "Plus Jakarta Sans"; /* UI, body, titles, labels */`).

## Riscos / Trade-offs

- **[Risco] Dispositivos com núcleos big.LITTLE com frequências máximas diferentes por cluster** -> **Mitigação**: A média aritmética simples entre os núcleos online (`sum_cur / count` e `sum_max / count`) produz um valor representativo e mantém a proporção exata da barra (`current / max`), além de ser determinística e sem custo perceptível de CPU.
- **[Risco] Permissão negada em algum núcleo isolado no `sysfs`** -> **Mitigação**: Cada núcleo é lido de forma independente com fallback de `scaling_cur_freq` para `cpuinfo_cur_freq` e de `cpuinfo_max_freq` para `scaling_max_freq`; núcleos inacessíveis ou offline (`0`) são ignorados, e se nenhum núcleo puder ser lido em Rust, o valor de fallback vindo de `Stats::parse` é preservado.
