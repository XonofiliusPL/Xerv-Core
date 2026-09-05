//! Testy modelu ekranów (Screen) i nawigacji w App.

use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, Screen};
use xerv_tui::event::Event;

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

#[test]
fn try_new_starts_on_main_screen() {
    let app = make_app();
    assert_eq!(app.current_screen, Screen::Main);
    assert!(!app.should_quit);
    assert_eq!(app.nav_cursor, 0);
    assert!(app.nav_stack.is_empty());
}

#[test]
fn nav_down_up_moves_cursor_only_on_main() {
    let mut app = make_app();
    app.handle_event(Event::NavDown);
    assert_eq!(app.nav_cursor, 1);
    app.handle_event(Event::NavDown);
    assert_eq!(app.nav_cursor, 2); // wrap (3 items → 2, dalej 0)
    app.handle_event(Event::NavDown);
    assert_eq!(app.nav_cursor, 0);
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
    // Wejście w Help, potem ←↓ nie powinno zmieniać nav_cursor (nie ma listy).
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
    app.nav_cursor = 2; // Quit
    app.handle_event(Event::Confirm);
    assert!(app.should_quit);
}

#[test]
fn backtick_from_settings_returns_to_main() {
    let mut app = make_app();
    app.nav_cursor = 0;
    app.handle_event(Event::Confirm); // Settings
    assert_eq!(app.current_screen, Screen::Settings);
    app.handle_event(Event::Return); // Backspace
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
    app.handle_event(Event::Confirm); // Settings
    app.handle_event(Event::OpenHelp);
    assert_eq!(app.current_screen, Screen::Help);
    assert_eq!(app.nav_stack.len(), 2);
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
