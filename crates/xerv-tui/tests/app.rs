//! Tests for the screen model (Screen) and navigation in App.
use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, NavItem, Screen};
use xerv_tui::event::Event;

fn make_app() -> App {
    make_app_with_update(None)
}

fn make_app_with_update(update: Option<String>) -> App {
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
    let mut app = App::try_new(cfg, state_path).unwrap();
    app.update_available = update;
    app
}

#[test]
fn try_new_starts_on_main_screen() {
    let app = make_app();
    assert_eq!(app.current_screen, Screen::Main);
    assert!(!app.should_quit);
    assert_eq!(app.nav_cursor, 0);
    assert!(app.nav_stack.is_empty());
    assert!(app.update_available.is_none());
}

#[test]
fn entries_without_update_has_3_items() {
    let app = make_app();
    let entries = app.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0], NavItem::Settings);
    assert_eq!(entries[1], NavItem::Help);
    assert_eq!(entries[2], NavItem::Quit);
    assert!(!entries.contains(&NavItem::Update));
}

#[test]
fn entries_with_update_has_4_items_with_update_above_quit() {
    let app = make_app_with_update(Some("0.2.0".to_string()));
    let entries = app.entries();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0], NavItem::Settings);
    assert_eq!(entries[1], NavItem::Help);
    assert_eq!(entries[2], NavItem::Update);
    assert_eq!(entries[3], NavItem::Quit);
    // Update must be directly above Quit.
    let update_pos = entries.iter().position(|&e| e == NavItem::Update).unwrap();
    let quit_pos = entries.iter().position(|&e| e == NavItem::Quit).unwrap();
    assert_eq!(quit_pos, update_pos + 1);
}

#[test]
fn nav_down_up_moves_cursor_only_on_main() {
    let mut app = make_app();
    app.handle_event(Event::NavDown);
    assert_eq!(app.nav_cursor, 1);
    app.handle_event(Event::NavDown);
    assert_eq!(app.nav_cursor, 2);
    app.handle_event(Event::NavDown);
    assert_eq!(app.nav_cursor, 0); // wrap
    app.handle_event(Event::NavUp);
    assert_eq!(app.nav_cursor, 2);
    app.handle_event(Event::NavUp);
    assert_eq!(app.nav_cursor, 1);
}

#[test]
fn nav_arrows_left_right_also_move_cursor() {
    let mut app = make_app();
    app.handle_event(Event::NavRight);
    assert_eq!(app.nav_cursor, 1);
    app.handle_event(Event::NavLeft);
    assert_eq!(app.nav_cursor, 0);
}

#[test]
fn navigation_keys_do_not_move_cursor_off_main() {
    let mut app = make_app();
    app.handle_event(Event::OpenHelp);
    assert_eq!(app.current_screen, Screen::Help);
    app.handle_event(Event::NavDown);
    assert_eq!(app.nav_cursor, 0, "NavDown must not move cursor off Main");
}

#[test]
fn enter_on_settings_opens_settings() {
    let mut app = make_app();
    app.nav_cursor = 0; // Settings
    app.handle_event(Event::Confirm);
    assert_eq!(app.current_screen, Screen::Settings);
    assert_eq!(app.nav_stack.len(), 1);
}

#[test]
fn enter_on_help_opens_help() {
    let mut app = make_app();
    app.nav_cursor = 1; // Help
    app.handle_event(Event::Confirm);
    assert_eq!(app.current_screen, Screen::Help);
    assert_eq!(app.nav_stack.len(), 1);
}

#[test]
fn enter_on_quit_quits() {
    let mut app = make_app();
    let entries = app.entries();
    let quit_pos = entries.iter().position(|&e| e == NavItem::Quit).unwrap();
    app.nav_cursor = quit_pos;
    app.handle_event(Event::Confirm);
    assert!(app.should_quit);
}

#[test]
fn backtick_from_settings_returns_to_main() {
    let mut app = make_app();
    app.nav_cursor = 0;
    app.handle_event(Event::Confirm);
    assert_eq!(app.current_screen, Screen::Settings);
    app.handle_event(Event::Return);
    assert_eq!(app.current_screen, Screen::Main);
    assert!(app.nav_stack.is_empty());
}

#[test]
fn h_opens_help_from_main() {
    let mut app = make_app();
    app.handle_event(Event::OpenHelp);
    assert_eq!(app.current_screen, Screen::Help);
    assert_eq!(app.nav_stack.len(), 1);
}

#[test]
fn h_opens_help_from_settings() {
    let mut app = make_app();
    app.nav_cursor = 0;
    app.handle_event(Event::Confirm);
    app.handle_event(Event::OpenHelp);
    assert_eq!(app.current_screen, Screen::Help);
    assert!(app.nav_stack.len() >= 2);
}

#[test]
fn backtick_from_help_returns_to_previous_screen() {
    let mut app = make_app();
    app.handle_event(Event::OpenHelp);
    assert_eq!(app.current_screen, Screen::Help);
    app.handle_event(Event::Return);
    assert_eq!(app.current_screen, Screen::Main);
}

#[test]
fn q_quit_is_universal() {
    let mut app = make_app();
    app.handle_event(Event::OpenHelp);
    app.handle_event(Event::Quit);
    assert!(app.should_quit);
}

#[test]
fn refresh_and_tick_are_no_ops() {
    let mut app = make_app();
    app.handle_event(Event::Tick);
    assert!(!app.should_quit);
    assert_eq!(app.current_screen, Screen::Main);
}

// ---- Update-specific navigation tests ----

#[test]
fn open_update_with_no_update_available_is_noop() {
    let mut app = make_app(); // update_available = None
    app.handle_event(Event::OpenUpdate);
    assert_eq!(app.current_screen, Screen::Main);
}

#[test]
fn open_update_with_update_available_opens_confirm() {
    let mut app = make_app_with_update(Some("0.2.0".to_string()));
    app.handle_event(Event::OpenUpdate);
    assert_eq!(app.current_screen, Screen::UpdateConfirm);
    assert_eq!(app.nav_stack.len(), 1);
}

#[test]
fn confirm_update_on_confirm_screen_sets_in_progress() {
    let mut app = make_app_with_update(Some("0.2.0".to_string()));
    app.handle_event(Event::OpenUpdate);
    app.handle_event(Event::ConfirmUpdate);
    assert!(app.update_in_progress);
}

#[test]
fn cancel_update_returns_to_main() {
    let mut app = make_app_with_update(Some("0.2.0".to_string()));
    app.handle_event(Event::OpenUpdate);
    assert_eq!(app.current_screen, Screen::UpdateConfirm);
    app.handle_event(Event::CancelUpdate);
    assert_eq!(app.current_screen, Screen::Main);
}

#[test]
fn enter_on_update_opens_confirm() {
    let mut app = make_app_with_update(Some("0.2.0".to_string()));
    let entries = app.entries();
    let update_pos = entries.iter().position(|&e| e == NavItem::Update).unwrap();
    app.nav_cursor = update_pos;
    app.handle_event(Event::Confirm);
    assert_eq!(app.current_screen, Screen::UpdateConfirm);
}

#[test]
fn return_from_update_confirm_returns_to_main() {
    let mut app = make_app_with_update(Some("0.2.0".to_string()));
    app.handle_event(Event::OpenUpdate);
    app.handle_event(Event::Return);
    assert_eq!(app.current_screen, Screen::Main);
}
