//! Testy headera TUI — brand XERV + link do GitHub.
//!
//! Header jest statycznym Top Barem: nie pokazuje pól diagnostycznych Core
//! (status, API, schema, uptime, data dir, state file, log, boot).

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use xerv_core::api::CoreConfig;
use xerv_tui::app::App;
use xerv_tui::ui::ui;

fn make_app() -> App {
    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join("state.json");
    std::fs::write(
        &state_path,
        r#"{"schema_version":7,"started_at_unix":1700000000,"boot_count":42}"#,
    )
    .unwrap();
    let cfg = CoreConfig {
        data_dir: dir.path().to_path_buf(),
        log_level: "debug".into(),
        state_filename: "state.json".into(),
    };
    App::try_new(cfg, state_path).unwrap()
}

fn render(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, app)).unwrap();
    terminal.backend().to_string()
}

#[test]
fn header_contains_brand() {
    let mut app = make_app();
    let s = render(&mut app, 80, 24);
    assert!(s.contains("XERV"), "header missing brand: {s}");
}

#[test]
fn header_contains_github_link() {
    let mut app = make_app();
    let s = render(&mut app, 120, 24);
    assert!(
        s.contains("https://github.com/XonofiliusPL/Xerv-Core"),
        "header missing github link: {s}"
    );
}

#[test]
fn header_does_not_contain_core_diagnostics() {
    let mut app = make_app();
    let s = render(&mut app, 120, 24);
    // Header nie powinien pokazywać pól diagnostycznych Core UI.
    for forbidden in ["api 0.1.0", "schema", "uptime", "boot #", "log level"] {
        assert!(!s.contains(forbidden), "header must not show '{forbidden}'");
    }
}

#[test]
fn header_fits_narrow_terminal() {
    let mut app = make_app();
    // Wąski terminal — header nie panicuje.
    let _ = render(&mut app, 30, 20);
}
