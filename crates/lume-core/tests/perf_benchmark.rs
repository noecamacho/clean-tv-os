use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

use lume_core::app::{ids, Key, Screen, Tab};
use lume_core::catalog::Discovered;
use lume_core::clock::WallClock;
use lume_core::shell::Shell;
use lume_core::text::{TextEngine, TextStyle};

fn create_test_catalog(count: usize) -> Vec<Discovered> {
    (0..count)
        .map(|i| Discovered {
            package: format!("com.app.test{i:02}"),
            label: format!("Test App {i:02}"),
            activity: format!("com.app.test{i:02}.MainActivity"),
            leanback: true,
            system: false,
            icon: None,
        })
        .collect()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lume-perf-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn create_bench_shell(dir: &std::path::Path, apps: Vec<Discovered>) -> Shell {
    let clock = WallClock {
        unix_secs: 1_700_000_000,
        utc_offset_secs: 0,
        is_24h: true,
    };
    let entries = apps;
    let mut shell = Shell::start(
        dir,
        "app.lume.tv",
        &["pt-BR".to_string()],
        clock,
        &mut || entries.clone(),
    );
    shell.wake(1920, 1080);
    let mut now = 0;
    for _ in 0..300 {
        if shell.frame_due(now) {
            shell.build_frame(now);
        } else if !shell.app().in_splash() {
            break;
        }
        now += 16;
    }
    shell
}

#[test]
#[ignore]
fn bench_before_after_tuning() {
    println!("\n=== Benchmark de Performance (Lume TV Launcher) ===");

    // 1. TextEngine::ellipsize (100 chamadas: 50 longa + 50 curta)
    let mut engine = TextEngine::new(1.0);
    let style = TextStyle::new(20.0, 500);
    let long_str = "Um Aplicativo de Streaming com Nome Muito Longo para TV 4K";
    let short_str = "YouTube";

    for _ in 0..50 {
        for _ in 0..50 {
            let _ = engine.ellipsize(long_str, &style, 140.0);
            let _ = engine.ellipsize(short_str, &style, 140.0);
        }
    }

    let start = Instant::now();
    for _ in 0..500 {
        for _ in 0..50 {
            let _ = engine.ellipsize(long_str, &style, 140.0);
            let _ = engine.ellipsize(short_str, &style, 140.0);
        }
    }
    let ellipsize_us = start.elapsed().as_secs_f64() * 1_000_000.0 / 500.0;
    println!("1. TextEngine::ellipsize (100 chamadas):");
    println!("   - Tempo médio: {ellipsize_us:.2} µs");

    // Preparar shell com 48 apps no catálogo
    let apps = create_test_catalog(48);
    let dir = temp_dir("bench-shell");
    let mut shell = create_bench_shell(&dir, apps);

    // 2. Quadro completo da Grade de Apps (Tab::Apps)
    shell.key(Key::Up, 10_000);
    shell.key(Key::Right, 10_016);
    shell.build_frame(10_032);
    assert_eq!(shell.app().tab(), Tab::Apps, "Shell deve estar na aba Apps");

    for i in 0..50 {
        shell.build_frame(20_000 + i * 16);
    }
    let mut batches_count = 0;
    let start = Instant::now();
    for i in 0..500 {
        let frame = shell.build_frame(30_000 + i * 16).unwrap();
        batches_count = frame.batches.len();
    }
    let apps_frame_us = start.elapsed().as_secs_f64() * 1_000_000.0 / 500.0;
    println!("2. Quadro da Grade de Apps (48 apps):");
    println!("   - Tempo médio: {apps_frame_us:.2} µs");
    println!("   - Batches GPU: {batches_count}");

    // 3. Quadro completo do Editor (Screen::Editor)
    shell.key(Key::Down, 40_000);
    shell.app_mut_for_test().focus_for_test(ids::GRID_SETTINGS);
    shell.key(Key::Ok, 40_016); // Abre Settings
    shell.build_frame(40_020); // Resolve layout e foco inicial de Settings
    assert_eq!(
        shell.app().screen(),
        &Screen::Settings,
        "Shell deve estar em Settings"
    );
    shell.key(Key::Ok, 40_032); // Abre Editor (ITEM_HOME_SCREEN)
    shell.build_frame(40_036); // Resolve layout do Editor
    assert_eq!(
        shell.app().screen(),
        &Screen::Editor,
        "Shell deve estar no Editor"
    );

    for i in 0..50 {
        shell.build_frame(50_000 + i * 16);
    }
    let start = Instant::now();
    for i in 0..500 {
        shell.build_frame(60_000 + i * 16);
    }
    let editor_frame_us = start.elapsed().as_secs_f64() * 1_000_000.0 / 500.0;
    println!("3. Quadro do Editor (Screen::Editor):");
    println!("   - Tempo médio: {editor_frame_us:.2} µs");

    // 4. Quadro completo da tela Sobre (Screen::About)
    shell.key(Key::Back, 70_000); // Volta para Settings
    shell.build_frame(70_010); // Resolve layout de Settings
    assert_eq!(
        shell.app().screen(),
        &Screen::Settings,
        "Shell deve voltar para Settings"
    );
    shell
        .app_mut_for_test()
        .focus_for_test(ids::SETTINGS_BASE + 3); // ITEM_ABOUT
    shell.key(Key::Ok, 70_020); // Abre About
    shell.build_frame(70_030); // Resolve layout de About
    assert_eq!(
        shell.app().screen(),
        &Screen::About,
        "Shell deve estar em About"
    );

    for i in 0..50 {
        shell.build_frame(80_000 + i * 16);
    }
    let start = Instant::now();
    for i in 0..500 {
        shell.build_frame(90_000 + i * 16);
    }
    let about_frame_us = start.elapsed().as_secs_f64() * 1_000_000.0 / 500.0;
    println!("4. Quadro da tela Sobre (Screen::About):");
    println!("   - Tempo médio: {about_frame_us:.2} µs");

    // 5. Shell::icons_wanted(1) (1.000 chamadas com 48 apps no catálogo)
    let skip = HashSet::new();
    for _ in 0..50 {
        for _ in 0..1000 {
            let _ = shell.icons_wanted(&skip, 1);
        }
    }
    let start = Instant::now();
    for _ in 0..500 {
        for _ in 0..1000 {
            let _ = shell.icons_wanted(&skip, 1);
        }
    }
    let icons_wanted_us = start.elapsed().as_secs_f64() * 1_000_000.0 / 500.0;
    println!("5. Shell::icons_wanted(1) (1.000 chamadas):");
    println!("   - Tempo médio: {icons_wanted_us:.2} µs");
    println!("===================================================\n");
}
