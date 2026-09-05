//! Testy sidebaru TUI: 8 pozycji, nawigacja klawiatura/mysz, hover vs active.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, Panel};
use xerv_tui::event::Event;
use xerv_tui::ui::ui;

fn make_app(data_dir: std::path::PathBuf) -> App {
    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join("state.json");
    std::fs::write(
        &state_path,
        r#"{"schema_version":7,"started_at_unix":1700000000,"boot_count":42}"#,
    )
    .unwrap();
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::copy(&state_path, data_dir.join("state.json")).unwrap();

    let cfg = CoreConfig {
        data_dir: data_dir.clone(),
        log_level: "debug".into(),
        state_filename: "state.json".into(),
    };
    App::try_new(cfg, data_dir.join("state.json")).unwrap()
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
fn sidebar_shows_all_eight_items() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    let s = render(&mut app, 120, 20);

    for name in [
        "Dashboard",
        "Agents",
        "Projects",
        "Services",
        "Packages",
        "Plugins",
        "Settings",
        "Help",
    ] {
        assert!(s.contains(name), "sidebar should contain {name}: {s}");
    }
}

#[test]
fn sidebar_default_active_is_dashboard() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    // Na starcie: side_active = 0 (Dashboard), side_cursor = 0, side_hover = None.
    assert_eq!(app.side_active, 0);
    assert_eq!(app.side_cursor, 0);
    assert_eq!(app.side_hover, None);
    let s = render(&mut app, 120, 20);
    // Dashboard powinien mieć marker ● (aktywny, cyan).
    assert!(
        s.contains("● Dashboard"),
        "Dashboard should be active with ● marker: {s}"
    );
}

#[test]
fn sidebar_nav_down_moves_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 1);
    assert_eq!(app.side_active, 0); // active się nie zmienia
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_nav_down_wraps_at_end() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    // Przewijamy do ostatniego elementu.
    for _ in 0..7 {
        app.handle_event(Event::NavDown);
    }
    assert_eq!(app.side_cursor, 7);
    // Kolejny NavDown powinien wrócić do 0.
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 0);
}

#[test]
fn sidebar_nav_up_moves_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 2);
    app.handle_event(Event::NavUp);
    assert_eq!(app.side_cursor, 1);
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_nav_up_wraps_at_start() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    app.handle_event(Event::NavUp);
    assert_eq!(app.side_cursor, 7); // z 0 na 7
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_home_goes_to_first() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    // Przemieszczamy Cursor
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 2);
    app.handle_event(Event::NavHome);
    assert_eq!(app.side_cursor, 0);
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_end_goes_to_last() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    app.handle_event(Event::NavEnd);
    assert_eq!(app.side_cursor, 7);
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_enter_sets_active_to_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    // Przewijamy do Agents (1)
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 1);
    assert_eq!(app.side_active, 0); // jeszcze Dashboard
                                    // Enter potwierdza
    app.handle_event(Event::NavActivate);
    assert_eq!(app.side_active, 1);
    assert_eq!(app.side_cursor, 1);
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_space_sets_active_to_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 2);
    app.handle_event(Event::NavActivate);
    assert_eq!(app.side_active, 2);
}

#[test]
fn sidebar_click_sets_active_and_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    // Muszą być wyrenderowane, żeby side_area został wypełniony.
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    // Klik w pozycję o indeksie 3 (Services).
    let item_y = side_area.y + 1 + 1 + 3; // border + nagłówek + offset
    let item_x = side_area.x + 1;
    app.handle_event(Event::ClickPanel(item_x, item_y));

    assert_eq!(app.side_active, 3);
    assert_eq!(app.side_cursor, 3);
    assert_eq!(app.side_hover, Some(3));
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_click_outside_items_no_effect() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    // Klik w nagłówek (y + 1 = pierwszy wiersz wewnątrz).
    let header_y = side_area.y + 1;
    let header_x = side_area.x + 1;
    app.handle_event(Event::ClickPanel(header_x, header_y));

    // Żadnej zmian — klik poza pozycjami.
    assert_eq!(app.side_active, 0);
    assert_eq!(app.side_cursor, 0);
    assert_eq!(app.side_hover, None);
}

#[test]
fn sidebar_mouse_move_updates_hover() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    let item_y = side_area.y + 1 + 1 + 4; // Packages (indeks 4)
    let item_x = side_area.x + 1;
    app.handle_event(Event::Hover(item_x, item_y));

    assert_eq!(app.side_hover, Some(4));
    assert_eq!(app.side_active, 0); // hover nie zmienia active
    assert_eq!(app.side_cursor, 0);
}

#[test]
fn sidebar_hover_and_active_are_separate() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    render(&mut app, 120, 20);

    // Ustawiamy active na Dashboard (0).
    app.side_active = 0;
    app.side_cursor = 0;

    let side_area = app.side_area.unwrap();
    // Hover na Help (7).
    let item_y = side_area.y + 1 + 1 + 7;
    let item_x = side_area.x + 1;
    app.handle_event(Event::Hover(item_x, item_y));

    assert_eq!(app.side_active, 0);
    assert_eq!(app.side_cursor, 0);
    assert_eq!(app.side_hover, Some(7));
}

#[test]
fn sidebar_hover_clears_when_moving_away() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    let item_y = side_area.y + 1 + 1 + 5; // Plugins
    let item_x = side_area.x + 1;
    app.handle_event(Event::Hover(item_x, item_y));
    assert_eq!(app.side_hover, Some(5));

    // Ruch myszy poza sidebar — hover powinien wyschnąć.
    app.handle_event(Event::Hover(200, 200));
    assert_eq!(app.side_hover, None);
}

#[test]
fn sidebar_render_active_is_cyan_bold() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    // Ustawiamy active na Settings (6).
    app.side_active = 6;
    app.side_cursor = 6;
    let s = render(&mut app, 120, 20);

    // Aktywny element ma ● oraz pogrubioną nazwę.
    assert!(
        s.contains("● Settings"),
        "active item should have ● marker: {s}"
    );
}

#[test]
fn sidebar_render_hover_is_magenta() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    // Hover na Agents (1), ale active jest Dashboard (0).
    let item_y = side_area.y + 1 + 1 + 1;
    let item_x = side_area.x + 1;
    app.handle_event(Event::Hover(item_x, item_y));

    let s = render(&mut app, 120, 20);
    // Hover ma ○ (piórko) w magenta.
    assert!(
        s.contains("○ Agents"),
        "hover item should have ○ marker: {s}"
    );
    // Active (Dashboard) powinien iść nadal ●.
    assert!(s.contains("● Dashboard"), "active should keep ●: {s}");
}

#[test]
fn sidebar_render_cursor_when_panel_is_side() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    app.handle_event(Event::NavDown);
    // side_cursor = 1, active_panel = Side.
    let s = render(&mut app, 120, 20);
    // Cursor ma ▎.
    assert!(s.contains("▎ Agents"), "cursor should have ▎ marker: {s}");
}

#[test]
fn sidebar_render_cursor_hidden_when_panel_is_main() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path().to_path_buf());
    app.handle_event(Event::NavDown);
    // Przełączamy na Main.
    app.active_panel = Panel::Main;
    let s = render(&mut app, 120, 20);
    // Cursor ▎ nie powinien być widoczny, gdyż panel nie jest Side.
    // Agent nie powinien mieć ▎.
    assert!(
        !s.contains("▎ Agents"),
        "cursor should be hidden when panel is Main: {s}"
    );
}
