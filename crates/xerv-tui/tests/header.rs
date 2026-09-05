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

// ---- footer hint ----------------------------------------------------------------

/// Zwraca style komórki bufora w (x, y) — pozwala testować kolory, nie tylko tekst.
fn cell_styles(app: &mut App, width: u16, height: u16) -> Vec<Vec<ratatui::style::Style>> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, app)).unwrap();
    let buf = terminal.backend().buffer().clone();
    (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].style()).collect())
        .collect()
}

/// Pozycja (x,y) pierwszego znaku `needle` w `lines` (char-index).
fn find_cell(lines: &[String], needle: &str) -> Option<(usize, usize)> {
    for (y, l) in lines.iter().enumerate() {
        if let Some(xb) = l.find(needle) {
            return Some((y, l[..xb].chars().count()));
        }
    }
    None
}

#[test]
fn footer_keys_are_cyan_accents() {
    let mut app = make_app();
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let s = terminal.backend().to_string();
    let lines: Vec<String> = s.lines().map(|l| l.trim_matches('"').to_string()).collect();

    let styles = cell_styles(&mut app, 120, 24);
    for key in ["←", "↑", "→", "↓", "Enter", "BackSpace"] {
        if let Some((y, x)) = find_cell(&lines, key) {
            let st = styles[y][x];
            assert_eq!(
                st.fg,
                Some(ratatui::style::Color::Cyan),
                "footer key '{key}' must be Cyan (accent), got {:?}",
                st.fg
            );
        }
    }
}

#[test]
fn footer_update_shortcut_only_when_update_available() {
    let mut app = make_app();
    let s = render(&mut app, 120, 24);
    assert!(
        !s.contains("U  Update"),
        "no U Update in footer without update"
    );
    app.update_available = Some("0.2.0".to_string());
    let s2 = render(&mut app, 120, 24);
    assert!(
        s2.contains("U  Update"),
        "footer shows U Update when available"
    );
}

#[test]
fn footer_update_key_is_cyan_when_available() {
    let mut app = make_app();
    app.update_available = Some("0.2.0".to_string());
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let s = terminal.backend().to_string();
    let lines: Vec<String> = s.lines().map(|l| l.trim_matches('"').to_string()).collect();
    if let Some((y, x)) = find_cell(&lines, "U  Update") {
        let styles = cell_styles(&mut app, 120, 24);
        let st = styles[y][x];
        assert_eq!(
            st.fg,
            Some(ratatui::style::Color::Cyan),
            "footer Update key must be Cyan, got {:?}",
            st.fg
        );
    }
}

#[test]
fn footer_descriptions_are_white() {
    let mut app = make_app();
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let s = terminal.backend().to_string();
    let lines: Vec<String> = s.lines().map(|l| l.trim_matches('"').to_string()).collect();
    let styles = cell_styles(&mut app, 120, 24);

    for desc in ["Nav", "Confirm", "Return"] {
        if let Some((y, x)) = find_cell(&lines, desc) {
            let st = styles[y][x];
            assert_eq!(
                st.fg,
                Some(ratatui::style::Color::White),
                "footer description '{desc}' must be White (value), got {:?}",
                st.fg
            );
        }
    }
}
