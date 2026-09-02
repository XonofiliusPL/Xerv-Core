//! Testy headera TUI: dane z Core (nie hardcoded), responsywność, brak overflow.

use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use xerv_core::api::CoreConfig;
use xerv_tui::app::App;
use xerv_tui::ui::ui;

fn make_app(data_dir: std::path::PathBuf, started_at_unix: u64) -> App {
    // Ręcznie zbuduj CoreState i wstaw przez konstruktor App nie jest możliwy
    // (App::try_new wywołuje CoreState::load, który zawsze zwraca fresh()).
    // Zamiast tego — obejście: zapisujemy plik stanu z pożądanymi polami,
    // potem App::try_new go wczyta.
    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join("state.json");
    std::fs::write(
        &state_path,
        format!(
            r#"{{"schema_version":7,"started_at_unix":{},"boot_count":42}}"#,
            started_at_unix
        ),
    )
    .unwrap();
    // Wymuś katalog danych na data_dir z plikiem stanu wewnątrz.
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::copy(&state_path, data_dir.join("state.json")).unwrap();

    let cfg = CoreConfig {
        data_dir: data_dir.clone(),
        log_level: "debug".into(),
        state_filename: "state.json".into(),
    };
    App::try_new(cfg, data_dir.join("state.json")).unwrap()
}

fn make_app_in_tempdir(dir: &std::path::Path, started_at_unix: u64) -> App {
    // Wariant pomocniczy: zapisuje plik stanu bezpośrednio w danym dir,
    // bez kopiowania — dla przypadków gdy data_dir to tymczasowy katalog.
    let state_path = dir.join("state.json");
    std::fs::write(
        &state_path,
        format!(
            r#"{{"schema_version":7,"started_at_unix":{},"boot_count":42}}"#,
            started_at_unix
        ),
    )
    .unwrap();
    let cfg = CoreConfig {
        data_dir: dir.to_path_buf(),
        log_level: "debug".into(),
        state_filename: "state.json".into(),
    };
    App::try_new(cfg, state_path).unwrap()
}

fn render_to_string(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let _ = ui(f, app);
        })
        .unwrap();
    terminal.backend().to_string()
}

#[test]
fn header_contains_brand_and_api_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf(), 1_700_000_000);
    let s = render_to_string(&mut app, 120, 20);
    assert!(s.contains("XERV"), "header missing brand: {s}");
    assert!(s.contains("api 0.1.0"), "header missing api version: {s}");
}

#[test]
fn header_reflects_state_from_core_not_hardcoded() {
    // boot_count i schema_version pochodzą z pliku stanu w data_dir.
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf(), 1_700_000_000);
    let s = render_to_string(&mut app, 120, 20);
    assert!(
        s.contains("boot #42"),
        "header should show boot #42 from state: {s}"
    );
    assert!(
        s.contains("schema v7"),
        "header should show schema v7 from state: {s}"
    );
}

#[test]
fn header_shows_log_level_from_config() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf(), 1_700_000_000);
    let s = render_to_string(&mut app, 120, 20);
    assert!(
        s.contains("debug"),
        "header should show log_level debug: {s}"
    );
}

#[test]
fn header_shows_ready_status_when_core_running() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf(), 1_700_000_000);
    let s = render_to_string(&mut app, 120, 20);
    assert!(s.contains("READY"), "header should show READY status: {s}");
    assert!(
        !s.contains("SHUTDOWN"),
        "header should not show SHUTDOWN: {s}"
    );
}

#[test]
fn header_uptime_is_human_readable() {
    // Uptime = now - started_at_unix. Wstawiamy 1h w przeszłości.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let one_hour_ago = now - 3600;
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf(), one_hour_ago);
    let s = render_to_string(&mut app, 120, 20);
    // Akceptujemy "1h0m" lub "1h" (zależnie od formattera) — ważne, żeby "h" się pojawiło.
    assert!(
        s.contains('h') && (s.contains("1h") || s.contains("59m")),
        "header should show uptime in hours: {s}"
    );
}

#[test]
fn header_does_not_overflow_wide_terminal() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf(), 1_700_000_000);
    // Szeroki terminal: nic nie powinno wyjść poza ramkę.
    let _ = render_to_string(&mut app, 200, 20);
}

#[test]
fn header_does_not_overflow_narrow_terminal() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf(), 1_700_000_000);
    // Wąski terminal: header renderuje się w wersji uproszczonej (sama nazwa).
    let s = render_to_string(&mut app, 25, 20);
    assert!(
        s.contains("Xerv"),
        "minimal header should still show brand: {s}"
    );
}

#[test]
fn header_reflects_shutdown_status() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf(), 1_700_000_000);
    app.core.shutdown().unwrap();
    let s = render_to_string(&mut app, 120, 20);
    assert!(
        s.contains("SHUTDOWN"),
        "header should show SHUTDOWN after shutdown(): {s}"
    );
    assert!(
        !s.contains("READY"),
        "header should not show READY after shutdown(): {s}"
    );
}

#[test]
fn header_shortens_long_data_path() {
    // Wystarczająco głęboka, ale tworzalna ścieżka wewnątrz tempdir.
    let dir = tempfile::tempdir().unwrap();
    let long_dir = dir
        .path()
        .join("very")
        .join("long")
        .join("path")
        .join("to")
        .join("some")
        .join("deeply")
        .join("nested")
        .join("xerv");
    std::fs::create_dir_all(long_dir.parent().unwrap()).unwrap();
    let mut app = make_app_in_tempdir(long_dir.parent().unwrap(), 1_700_000_000);
    let s = render_to_string(&mut app, 120, 20);
    assert!(
        s.contains("…/"),
        "long path should be truncated with ellipsis: {s}"
    );
}
