//! Testy ekranów Core UI (Screen model) i interakcji myszy.
//!
//! Warstwa: ratatui `TestBackend` parsuje bufor renderu i weryfikuje
//! — renderowanie Main/Help/Settings,
//! — listę nawigacji (Settings, Help, Quit),
//! — hit-test mysą (Event::Hover / Event::Click),
//! — sekcje Help (skróty, info, GitHub link),
//! — stopkę z hintem,
//! — brak starych elementów Core UI (Sidebar, Command Bar, Install/Onboarding
//!   jako osobne ekrany, Agent Workspace, itp.).

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, Screen, NAV_COUNT};
use xerv_tui::event::Event;
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

/// Renderuje i zwraca (bufor jako linie, on_render z geometrią).
fn render(app: &mut App, width: u16, height: u16) -> Vec<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, app)).unwrap();
    terminal
        .backend()
        .to_string()
        .lines()
        .map(|l| l.strip_prefix('"').unwrap_or(l))
        .map(|l| l.strip_suffix('"').unwrap_or(l))
        .map(|l| l.to_string())
        .collect()
}

// ---- navigation list -------------------------------------------------------------

#[test]
fn main_screen_lists_exactly_required_items() {
    let mut app = make_app();
    let lines = render(&mut app, 80, 24);
    for name in ["Settings", "Help", "Quit"] {
        let found = lines.iter().any(|l| l.contains(name));
        assert!(found, "main screen should show '{name}'");
    }
    assert_eq!(NAV_COUNT, 3);
}

#[test]
fn sidebar_forbidden_items_not_present() {
    // Nie ma już sidebar — sprawdzamy, że nagłówek listy nie istnieje
    // i brak starych pozycji nawigacji.
    let mut app = make_app();
    let lines = render(&mut app, 100, 24);
    let joined = lines.join("\n");
    for forbidden in [
        "NAVIGATION",
        "Addons",
        "Agents",
        "Projects",
        "Services",
        "Packages",
        "Plugins",
        "Registry",
        "Marketplace",
        "Logs",
        "Workspaces",
        "Worktrees",
        "Sessions",
        "Terminal",
    ] {
        assert!(
            !joined.contains(forbidden),
            "UI must not contain '{forbidden}'"
        );
    }
}

// ---- settings / help / quit ------------------------------------------------------

#[test]
fn settings_screen_shows_placeholder() {
    let mut app = make_app();
    app.nav_cursor = 0;
    app.handle_event(Event::Confirm);
    let lines = render(&mut app, 80, 24);
    let joined = lines.join("\n");
    assert!(
        joined.contains("settings"),
        "settings screen should show title"
    );
}

#[test]
fn help_screen_shows_shortcuts_section() {
    let mut app = make_app();
    app.handle_event(Event::OpenHelp);
    let lines = render(&mut app, 80, 24);
    let joined = lines.join("\n");
    assert!(
        joined.contains("Keyboard"),
        "help should have Keyboard section"
    );
    for key in ["q", "h", "Enter", "BackSpace"] {
        let found = lines.iter().any(|l| l.contains(key));
        assert!(found, "help keyboard section should show '{key}'");
    }
}

#[test]
fn help_screen_shows_github_link() {
    let mut app = make_app();
    app.handle_event(Event::OpenHelp);
    let lines = render(&mut app, 100, 24);
    let joined = lines.join("\n");
    assert!(
        joined.contains("https://github.com/XonofiliusPL/Xerv-Core"),
        "help should contain github link: {joined}"
    );
}

#[test]
fn help_screen_shows_about_section() {
    let mut app = make_app();
    app.handle_event(Event::OpenHelp);
    let lines = render(&mut app, 80, 24);
    let joined = lines.join("\n");
    assert!(joined.contains("About"), "help should have About section");
}

// ---- footer hint ----------------------------------------------------------------

#[test]
fn footer_shows_exact_hint() {
    let mut app = make_app();
    let lines = render(&mut app, 100, 24);
    let joined = lines.join("\n");
    assert!(joined.contains("Nav"), "footer should contain 'Nav'");
    assert!(
        joined.contains("Confirm"),
        "footer should contain 'Confirm'"
    );
    assert!(joined.contains("Return"), "footer should contain 'Return'");
    // Footer nie ma dodatkowych akcji/statusów.
    for forbidden in ["Settings", "Help", "Quit", "Exit", "READY", "SHUTDOWN"] {
        assert!(
            !lines.last().unwrap().contains(forbidden),
            "footer must not contain '{forbidden}'"
        );
    }
}

// ---- core UI absence ------------------------------------------------------------

#[test]
fn no_command_bar_in_core_ui() {
    let mut app = make_app();
    let lines = render(&mut app, 120, 24);
    let joined = lines.join("\n");
    // Brak 4 slotów z ikonami Nerd Font (Command Bar).
    for icon in ["\u{f019}", "\u{f1ae}", "\u{f0db}", "\u{f009}"] {
        assert!(
            !joined.contains(icon),
            "core UI must not contain command-bar icon {icon}"
        );
    }
}

#[test]
fn no_core_dashboard_card_in_core_ui() {
    let mut app = make_app();
    let lines = render(&mut app, 120, 24);
    let joined = lines.join("\n");
    // Brak kart diagnostycznych.
    for forbidden in ["core", "XERV • api", "api 0.1.0", "schema v7", "uptime"] {
        assert!(
            !joined.contains(forbidden),
            "core UI must not contain '{forbidden}'"
        );
    }
}

// ---- mouse hover ----------------------------------------------------------------

#[test]
fn mouse_hover_sets_nav_hover_without_changing_cursor() {
    let mut app = make_app();
    let _ = render(&mut app, 80, 24);
    let area = app.nav_area.expect("nav area");
    // Hover na 2. pozycję (Help, idx 1).
    let col = area.x + 1;
    let row = area.y + 1;
    app.handle_event(Event::Hover(col, row));
    assert_eq!(app.nav_hover, Some(1), "hover should target 2nd item");
    assert_eq!(app.nav_cursor, 0, "cursor must not follow hover");
}

#[test]
fn mouse_hover_clears_outside_nav_area() {
    let mut app = make_app();
    let _ = render(&mut app, 80, 24);
    let area = app.nav_area.expect("nav area");
    let col = area.x + 1;
    let row = area.y + 1;
    app.handle_event(Event::Hover(col, row));
    assert_eq!(app.nav_hover, Some(1));
    // Poza obszarem listy.
    app.handle_event(Event::Hover(1, 1));
    assert_eq!(app.nav_hover, None);
}

// ---- mouse click ----------------------------------------------------------------

#[test]
fn mouse_click_activates_current_item() {
    let mut app = make_app();
    let _ = render(&mut app, 80, 24);
    let area = app.nav_area.expect("nav area");
    // Klik w Quit (idx 2).
    let col = area.x + 1;
    let row = area.y + 2;
    app.handle_event(Event::Click(col, row));
    assert_eq!(app.nav_cursor, 2);
    assert!(app.should_quit, "clicking Quit should quit");
}

#[test]
fn mouse_click_on_help_opens_help() {
    let mut app = make_app();
    let _ = render(&mut app, 80, 24);
    let area = app.nav_area.expect("nav area");
    let col = area.x + 1;
    let row = area.y + 1; // Help (idx 1)
    app.handle_event(Event::Click(col, row));
    assert_eq!(app.current_screen, Screen::Help);
}

#[test]
fn mouse_click_outside_nav_area_does_nothing() {
    let mut app = make_app();
    let _ = render(&mut app, 80, 24);
    app.handle_event(Event::Click(1, 1));
    assert_eq!(app.nav_cursor, 0);
    assert!(!app.should_quit);
    assert_eq!(app.current_screen, Screen::Main);
}

// ---- hover vs cursor separation --------------------------------------------------

#[test]
fn hover_does_not_change_nav_cursor() {
    let mut app = make_app();
    let _ = render(&mut app, 80, 24);
    app.nav_cursor = 0;
    let area = app.nav_area.expect("nav area");
    app.handle_event(Event::Hover(area.x + 1, area.y + 2)); // idx 2 (Quit)
    assert_eq!(app.nav_hover, Some(2));
    assert_eq!(app.nav_cursor, 0, "cursor must stay at 0");
}
