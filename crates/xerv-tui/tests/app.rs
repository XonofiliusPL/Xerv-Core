use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, Panel};
use xerv_tui::event::Event;

fn test_config(dir: &std::path::Path) -> (CoreConfig, std::path::PathBuf) {
    let cfg = CoreConfig {
        data_dir: dir.to_path_buf(),
        log_level: "warn".into(),
        state_filename: "state.json".into(),
    };
    let path = dir.join("state.json");
    (cfg, path)
}

#[test]
fn try_new_succeeds_with_valid_dir() {
    let dir = tempfile::tempdir().unwrap();
    let (cfg, path) = test_config(dir.path());
    let app = App::try_new(cfg, path).unwrap();
    assert_eq!(app.active_panel, Panel::Side);
    assert!(!app.should_quit);
}

#[test]
fn next_prev_cycles_panels() {
    let dir = tempfile::tempdir().unwrap();
    let (cfg, path) = test_config(dir.path());
    let mut app = App::try_new(cfg, path).unwrap();
    app.handle_event(Event::NextPanel);
    assert_eq!(app.active_panel, Panel::Main);
    app.handle_event(Event::NextPanel);
    assert_eq!(app.active_panel, Panel::Side);
    app.handle_event(Event::PrevPanel);
    assert_eq!(app.active_panel, Panel::Main);
}

#[test]
fn quit_event_sets_should_quit() {
    let dir = tempfile::tempdir().unwrap();
    let (cfg, path) = test_config(dir.path());
    let mut app = App::try_new(cfg, path).unwrap();
    app.handle_event(Event::Quit);
    assert!(app.should_quit);
}

#[test]
fn refresh_and_tick_are_no_ops() {
    let dir = tempfile::tempdir().unwrap();
    let (cfg, path) = test_config(dir.path());
    let mut app = App::try_new(cfg, path).unwrap();
    app.handle_event(Event::Refresh);
    app.handle_event(Event::Tick);
    assert!(!app.should_quit);
    assert_eq!(app.active_panel, Panel::Side);
}
