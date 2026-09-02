//! Testy Dashboardu: karty renderują dane z Core, command bar jest interaktywny,
//! layout nie overflowuje na wąskich i szerokich terminalach.

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, COMMANDS, COMMAND_COUNT};
use xerv_tui::event::Event;
use xerv_tui::ui::ui;

fn make_app(dir: &std::path::Path, started_at_unix: u64) -> App {
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

fn render(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut areas_holder: Option<xerv_tui::app::UiAreas> = None;
    terminal
        .draw(|f| {
            areas_holder = Some(ui(f, app));
        })
        .unwrap();
    app.on_render(areas_holder.expect("ui did not return areas"));
    terminal.backend().to_string()
}

#[test]
fn dashboard_shows_section_titles() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let s = render(&mut app, 120, 24);
    assert!(s.contains("system"), "missing system card: {s}");
    assert!(s.contains("runtime"), "missing runtime card: {s}");
}

#[test]
fn dashboard_shows_data_from_core() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let s = render(&mut app, 120, 24);
    assert!(s.contains("api version"), "missing api version row: {s}");
    assert!(s.contains("0.1.0"), "missing api version value: {s}");
    assert!(s.contains("boot count"), "missing boot count row: {s}");
    assert!(s.contains("42"), "missing boot count value from state: {s}");
    assert!(s.contains("schema"), "missing schema row: {s}");
    assert!(s.contains("v7"), "missing schema value from state: {s}");
    assert!(s.contains("READY"), "missing READY status: {s}");
    assert!(s.contains("debug"), "missing log level from config: {s}");
}

#[test]
fn dashboard_has_command_bar_with_slots() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let s = render(&mut app, 120, 24);
    for name in COMMANDS.iter() {
        assert!(s.contains(name), "command bar missing slot '{name}': {s}");
    }
}

#[test]
fn command_bar_keyboard_navigation_updates_selection() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    assert_eq!(app.selected_command, 0);
    app.handle_event(Event::NextCommand);
    assert_eq!(app.selected_command, 1);
    app.handle_event(Event::PrevCommand);
    assert_eq!(app.selected_command, 0);
    // Zawijanie w lewo.
    app.handle_event(Event::PrevCommand);
    assert_eq!(app.selected_command, COMMAND_COUNT - 1);
}

#[test]
fn command_bar_mouse_click_selects_slot() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    // Render 120x24 — command bar wypełniany w trakcie renderu.
    let _ = render(&mut app, 120, 24);
    let bar = app
        .command_bar_area
        .expect("command bar area not registered");
    // Klik w 3. slot (środek slotu, środek wysokości).
    let slot_w = bar.w / COMMAND_COUNT as u16;
    let click_col = bar.x + slot_w * 2 + slot_w / 2;
    let click_row = bar.y + bar.h / 2;
    app.handle_event(Event::ClickCommand(click_col, click_row));
    assert_eq!(app.selected_command, 2, "click should select 3rd command");
}

#[test]
fn mouse_click_on_side_panel_activates_side() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render(&mut app, 120, 24);
    let side = app.side_area.expect("side area not registered");
    app.handle_event(Event::ClickPanel(side.x + 1, side.y + 1));
    assert_eq!(app.active_panel, xerv_tui::app::Panel::Side);
}

#[test]
fn mouse_click_on_main_panel_activates_main() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render(&mut app, 120, 24);
    let main = app.main_area.expect("main area not registered");
    // Klik poza command barem (górna część main).
    app.handle_event(Event::ClickPanel(main.x + 1, main.y + 1));
    assert_eq!(app.active_panel, xerv_tui::app::Panel::Main);
}

#[test]
fn dashboard_renders_on_narrow_terminal_without_panic() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    // Wąski terminal: jedna kolumna kart.
    let s = render(&mut app, 45, 24);
    assert!(
        s.contains("system"),
        "narrow layout missing system card: {s}"
    );
}

#[test]
fn dashboard_renders_on_wide_terminal() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let s = render(&mut app, 200, 30);
    assert!(s.contains("system"), "wide layout missing system card: {s}");
    assert!(
        s.contains("runtime"),
        "wide layout missing runtime card: {s}"
    );
}

#[test]
fn shutdown_status_reflected_in_dashboard() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.core.shutdown().unwrap();
    let s = render(&mut app, 120, 24);
    assert!(
        s.contains("SHUTDOWN"),
        "dashboard should show SHUTDOWN: {s}"
    );
}
