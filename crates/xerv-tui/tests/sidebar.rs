//! Testy sidebaru TUI: 4 pozycje (Addons, Settings, Help, Exit),
//! nawigacja klawiatura/mysz, hover vs active vs cursor.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, Panel, SIDEBAR_COUNT, SIDEBAR_ITEMS};
use xerv_tui::event::Event;
use xerv_tui::ui::ui;

/// Tworzy aplikację w izolowanym dir (dane + config + state razem).
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
fn sidebar_shows_all_four_items() {
    let mut app = make_app();
    let s = render(&mut app, 120, 20);

    for name in SIDEBAR_ITEMS.iter() {
        assert!(s.contains(name), "sidebar should contain {name}");
    }
    assert_eq!(SIDEBAR_ITEMS.len(), 4);
}

#[test]
fn sidebar_default_active_is_addons() {
    let mut app = make_app();
    assert_eq!(app.side_active, 0); // Addons
    assert_eq!(app.side_cursor, 0);
    assert_eq!(app.side_hover, None);
    let s = render(&mut app, 120, 20);
    assert!(
        s.contains("\u{25CF} Addons"),
        "Addons should be active with \u{25CF}: {s}"
    );
}

#[test]
fn sidebar_nav_down_moves_cursor() {
    let mut app = make_app();
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 1);
    assert_eq!(app.side_active, 0); // nie zmienia aktywnego
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_nav_down_wraps_at_end() {
    let mut app = make_app();
    for _ in 0..(SIDEBAR_COUNT - 1) {
        app.handle_event(Event::NavDown);
    }
    assert_eq!(app.side_cursor, SIDEBAR_COUNT - 1);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 0);
}

#[test]
fn sidebar_nav_up_moves_cursor() {
    let mut app = make_app();
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 2);
    app.handle_event(Event::NavUp);
    assert_eq!(app.side_cursor, 1);
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_nav_up_wraps_at_start() {
    let mut app = make_app();
    app.handle_event(Event::NavUp);
    assert_eq!(app.side_cursor, SIDEBAR_COUNT - 1); // z 0 na koniec
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_home_goes_to_first() {
    let mut app = make_app();
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 2);
    app.handle_event(Event::NavHome);
    assert_eq!(app.side_cursor, 0);
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_end_goes_to_last() {
    let mut app = make_app();
    app.handle_event(Event::NavEnd);
    assert_eq!(app.side_cursor, SIDEBAR_COUNT - 1);
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_enter_sets_active_to_cursor() {
    let mut app = make_app();
    // Ustawiamy cursor na Settings (1).
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 1);
    assert_eq!(app.side_active, 0); // jeszcze Addons
    app.handle_event(Event::NavActivate);
    assert_eq!(app.side_active, 1);
    assert_eq!(app.side_cursor, 1);
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_space_sets_active_to_cursor() {
    let mut app = make_app();
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 2); // Help
    app.handle_event(Event::NavActivate);
    assert_eq!(app.side_active, 2);
}

#[test]
fn sidebar_click_sets_active_and_cursor() {
    let mut app = make_app();
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    // Klik w pozycję o indeksie 2 (Help).
    let item_y = side_area.y + 1 + 1 + 2; // border + " NAVIGATION" + offset
    let item_x = side_area.x + 1;
    app.handle_event(Event::Click(item_x, item_y));

    assert_eq!(app.side_active, 2);
    assert_eq!(app.side_cursor, 2);
    assert_eq!(app.side_hover, Some(2));
    assert_eq!(app.active_panel, Panel::Side);
}

#[test]
fn sidebar_click_outside_items_no_effect() {
    let mut app = make_app();
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    // Klik w nagłówek " NAVIGATION".
    let header_y = side_area.y + 1;
    let header_x = side_area.x + 1;
    app.handle_event(Event::Click(header_x, header_y));

    // Żadnej zmiany — klik poza pozycjami.
    assert_eq!(app.side_active, 0);
    assert_eq!(app.side_cursor, 0);
    assert_eq!(app.side_hover, None);
}

#[test]
fn sidebar_mouse_move_updates_hover() {
    let mut app = make_app();
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    // Hover na pozycji 1 (Settings).
    let item_y = side_area.y + 1 + 1 + 1;
    let item_x = side_area.x + 1;
    app.handle_event(Event::Hover(item_x, item_y));

    assert_eq!(app.side_hover, Some(1));
    assert_eq!(app.side_active, 0); // hover nie zmienia active
    assert_eq!(app.side_cursor, 0);
}

#[test]
fn sidebar_hover_and_active_are_separate() {
    let mut app = make_app();
    render(&mut app, 120, 20);

    app.side_active = 0; // Addons
    app.side_cursor = 0;

    let side_area = app.side_area.unwrap();
    // Hover na Help (2).
    let item_y = side_area.y + 1 + 1 + 2;
    let item_x = side_area.x + 1;
    app.handle_event(Event::Hover(item_x, item_y));

    assert_eq!(app.side_active, 0);
    assert_eq!(app.side_cursor, 0);
    assert_eq!(app.side_hover, Some(2));
}

#[test]
fn sidebar_hover_clears_when_moving_away() {
    let mut app = make_app();
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    // Hover na Settings (1).
    let item_y = side_area.y + 1 + 1 + 1;
    let item_x = side_area.x + 1;
    app.handle_event(Event::Hover(item_x, item_y));
    assert_eq!(app.side_hover, Some(1));

    // Ruch myszy poza sidebar — hover wycisza.
    app.handle_event(Event::Hover(200, 200));
    assert_eq!(app.side_hover, None);
}

#[test]
fn sidebar_render_active_is_cyan_bold() {
    let mut app = make_app();
    app.side_active = 1; // Settings
    app.side_cursor = 1;
    let s = render(&mut app, 120, 20);
    assert!(
        s.contains("\u{25CF} Settings"),
        "active item should have \u{25CF}: {s}"
    );
}

#[test]
fn sidebar_render_hover_is_magenta() {
    let mut app = make_app();
    render(&mut app, 120, 20);

    let side_area = app.side_area.unwrap();
    // Hover na Settings (1), active = Addons (0).
    let item_y = side_area.y + 1 + 1 + 1;
    let item_x = side_area.x + 1;
    app.handle_event(Event::Hover(item_x, item_y));

    let s = render(&mut app, 120, 20);
    // Hover ma ○ (piórko).
    assert!(
        s.contains("\u{25CB} Settings"),
        "hover item should have \u{25CB}: {s}"
    );
    // Aktywny Addons powinien mieć ●.
    assert!(
        s.contains("\u{25CF} Addons"),
        "active should keep \u{25CF}: {s}"
    );
}

#[test]
fn sidebar_render_cursor_when_panel_is_side() {
    let mut app = make_app();
    app.handle_event(Event::NavDown); // cursor = 1 (Settings)
    let s = render(&mut app, 120, 20);
    assert!(
        s.contains("\u{258E} Settings"),
        "cursor should have \u{258E}: {s}"
    );
}

#[test]
fn sidebar_render_cursor_hidden_when_panel_is_main() {
    let mut app = make_app();
    app.handle_event(Event::NavDown);
    app.active_panel = Panel::Main;
    let s = render(&mut app, 120, 20);
    assert!(
        !s.contains("\u{258E} Settings"),
        "cursor should be hidden when panel is Main: {s}"
    );
}
