//! Testy geometrii Dashboardu i interakcji myszy.
//!
//! Warstwa: ratatui `TestBackend` parsuje bufor renderu i weryfikuje
//! — clipping/overflow ramek,
//! — responsywność layoutu,
//! — hover/click hit-test (Event::Hover / Event::Click),
//! — separację keyboard focus / mouse hover / active.
//!
//! Core UI ma panele: Install → Onboarding → Main oraz sidebar
//! z pozycjami: Addons, Settings, Help, Exit.

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, CorePanel, Panel, COMMANDS, COMMAND_COUNT, SIDEBAR_COUNT, SIDEBAR_ITEMS};
use xerv_tui::event::Event;
use xerv_tui::ui::ui;

fn make_app(dir: &std::path::Path, started_at_unix: u64) -> App {
    let state_path = dir.join("state.json");
    std::fs::write(
        &state_path,
        format!(r#"{{"schema_version":7,"started_at_unix":{started_at_unix},"boot_count":42}}"#),
    )
    .unwrap();
    let cfg = CoreConfig {
        data_dir: dir.to_path_buf(),
        log_level: "debug".into(),
        state_filename: "state.json".into(),
    };
    App::try_new(cfg, state_path).unwrap()
}

/// Renderuje i zwraca (bufor jako linie, szerokość, wysokość).
fn render_lines(app: &mut App, width: u16, height: u16) -> Vec<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut areas_holder: Option<xerv_tui::app::UiAreas> = None;
    terminal
        .draw(|f| {
            areas_holder = Some(ui(f, app));
        })
        .unwrap();
    app.on_render(areas_holder.expect("ui did not return areas"));
    let s = terminal.backend().to_string();
    s.lines()
        .map(|l| l.strip_prefix('"').unwrap_or(l))
        .map(|l| l.strip_suffix('"').unwrap_or(l))
        .map(|l| l.to_string())
        .collect()
}

#[track_caller]
fn assert_command_bar_within_viewport(lines: &[String], width: u16, ctx: &str) {
    let bar_row = lines
        .iter()
        .position(|l| l.matches('┌').count() >= COMMAND_COUNT)
        .unwrap_or_else(|| panic!("{ctx}: command bar not found"));
    let line = &lines[bar_row];
    assert_eq!(
        line.chars().count(),
        width as usize,
        "{ctx}: command bar row width != terminal width"
    );
    let open_slots = line.matches('┌').count();
    assert_eq!(
        open_slots, COMMAND_COUNT,
        "{ctx}: expected {COMMAND_COUNT} command slots, found {open_slots}"
    );
    let close_row = bar_row + 2;
    let close_line = lines
        .get(close_row)
        .unwrap_or_else(|| panic!("{ctx}: command bar closing row missing"));
    let trimmed = close_line.trim_end();
    assert!(
        trimmed.ends_with('┘'),
        "{ctx}: last command slot appears clipped (no closing border at right edge): '{close_line}'"
    );
}

#[track_caller]
fn assert_no_line_overflows(lines: &[String], width: u16, ctx: &str) {
    for (i, l) in lines.iter().enumerate() {
        assert!(
            l.chars().count() <= width as usize,
            "{ctx}: line {i} overflows viewport ({} > {}): '{l}'",
            l.chars().count(),
            width
        );
    }
}

#[track_caller]
fn assert_core_card_present(lines: &[String], ctx: &str, label: &str) {
    let present = lines.iter().any(|l| l.contains(&format!("┌ {label} ")));
    assert!(present, "{ctx}: core card '{label}' not found");
}

// ---- panel presence -------------------------------------------------------------

#[test]
fn install_panel_shows_progress_card() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.core_panel = CorePanel::Install;
    let lines = render_lines(&mut app, 80, 24);
    assert_core_card_present(&lines, "80", "install");
}

#[test]
fn onboarding_panel_shows_progress_card() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.core_panel = CorePanel::Onboarding;
    let lines = render_lines(&mut app, 80, 24);
    assert_core_card_present(&lines, "80", "onboarding");
}

#[test]
fn main_panel_shows_core_card() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.core_panel = CorePanel::Main;
    let lines = render_lines(&mut app, 80, 24);
    assert_core_card_present(&lines, "80", "core");
}

#[test]
fn main_panel_renders_core_fields() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.core_panel = CorePanel::Main;
    let lines = render_lines(&mut app, 100, 24);
    for needle in ["CORE READY", "api", "xerv", "schema", "uptime", "data dir"] {
        let found = lines.iter().any(|l| l.contains(needle));
        assert!(found, "core card should show '{needle}'");
    }
}

// ---- sidebar navigation list -----------------------------------------------------

#[test]
fn sidebar_lists_exactly_required_items() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 80, 24);
    for name in SIDEBAR_ITEMS.iter() {
        let found = lines.iter().any(|l| l.contains(name));
        assert!(found, "sidebar should show '{name}'");
    }
    assert_eq!(SIDEBAR_ITEMS.len(), 4);
}

#[test]
fn sidebar_does_not_list_forbidden_items() {
    let items = SIDEBAR_ITEMS.join(",");
    for forbidden in [
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
            !items.contains(forbidden),
            "sidebar must NOT contain '{forbidden}'"
        );
    }
}

// ---- panel navigation ------------------------------------------------------------

#[test]
fn nav_activate_advances_install_to_onboarding_to_main() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.core_panel = CorePanel::Install;
    // Enter w panelu treści (Main) przechodzi do kolejnego CorePanela.
    app.active_panel = Panel::Main;
    assert_eq!(app.core_panel, CorePanel::Install);
    app.handle_event(Event::NavActivate);
    assert_eq!(app.core_panel, CorePanel::Onboarding);
    app.handle_event(Event::NavActivate);
    assert_eq!(app.core_panel, CorePanel::Main);
    // Main → Main (brak dalszego zaawansowania).
    app.handle_event(Event::NavActivate);
    assert_eq!(app.core_panel, CorePanel::Main);
}

#[test]
fn nav_down_up_cycles_sidebar() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.active_panel = Panel::Side;
    assert_eq!(app.side_cursor, 0);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 1);
    app.handle_event(Event::NavEnd);
    assert_eq!(app.side_cursor, SIDEBAR_COUNT - 1);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 0);
}

// ---- exit -----------------------------------------------------------------------

#[test]
fn exit_quit_from_sidebar_activate() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.active_panel = Panel::Side;
    app.side_cursor = SIDEBAR_COUNT - 1; // Exit
    app.handle_event(Event::NavActivate);
    assert!(app.should_quit);
}

#[test]
fn exit_quit_from_mouse_click() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 80, 24);
    let side = app.side_area.expect("side area");
    let exit_idx = SIDEBAR_COUNT - 1;
    // y: border(1) + "NAVIGATION" nagłówek(1) + offset do Exit.
    let row = side.y + 1 + 1 + exit_idx as u16;
    app.handle_event(Event::Click(side.x + 2, row));
    assert!(app.should_quit, "clicking Exit should quit");
}

// ---- command bar ----------------------------------------------------------------

#[test]
fn command_bar_slot_count_matches_commands() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24);
    for name in COMMANDS.iter() {
        let found = lines.iter().any(|l| l.contains(name));
        assert!(found, "command '{name}' not visible in bar");
    }
}

#[test]
fn command_bar_click_selects_slot_and_syncs_hover() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let bar = app.command_bar_area.expect("bar");
    let slot_w = bar.w / COMMAND_COUNT as u16;
    assert!(slot_w >= 5, "slot too narrow ({slot_w})");
    let col = bar.x + slot_w * 2 + slot_w / 2; // 3rd slot
    let row = bar.y + 1;
    app.handle_event(Event::Click(col, row));
    assert_eq!(
        app.selected_command, 2,
        "selected should match clicked slot"
    );
    assert_eq!(
        app.hovered_command,
        Some(2),
        "hover should sync to clicked slot"
    );
    assert_eq!(app.active_panel, Panel::Main);
}

#[test]
fn click_in_command_bar_then_move_mouse_keeps_hover() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let bar = app.command_bar_area.expect("bar");
    let col = bar.x + bar.w / 2;
    let row = bar.y + 1;
    app.handle_event(Event::Click(col, row));
    let _ = app.hovered_command; // snapshot
                                 // Teraz ruch myszy poza command bar — hover powinien się wyczyścić.
    app.handle_event(Event::Hover(col, 1));
    assert_eq!(
        app.hovered_command, None,
        "hover must clear when mouse leaves bar"
    );
}

// ---- mouse hover ----------------------------------------------------------------

#[test]
fn mouse_hover_command_bar_shows_hover_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let bar = app.command_bar_area.expect("bar");
    let slot_w = bar.w / COMMAND_COUNT as u16;
    let col = bar.x + slot_w + slot_w / 2; // 2nd slot
    let row = bar.y + 1;
    app.handle_event(Event::Hover(col, row));
    assert_eq!(
        app.hovered_command,
        Some(1),
        "hover should target 2nd command slot"
    );
}

#[test]
fn mouse_hover_dashboard_card_shows_hover_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.core_panel = CorePanel::Main;
    let _ = render_lines(&mut app, 120, 24);
    let card = app
        .card_areas
        .iter()
        .find(|(n, _)| *n == "core")
        .map(|(_, a)| *a)
        .expect("core card area");
    let col = card.x + card.w / 2;
    let row = card.y + card.h / 2;
    app.handle_event(Event::Hover(col, row));
    assert_eq!(
        app.dashboard_hover,
        Some("core"),
        "hover should target 'core' card"
    );
}

#[test]
fn mouse_hover_sidebar_shows_side_hover() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 80, 24);
    let side = app.side_area.expect("side area");
    let col = side.x + 2;
    // 2nd sidebar item (indeks 1): border(1) + "NAVIGATION" header(1) + offset 1.
    let row = side.y + 1 + 1 + 1;
    app.handle_event(Event::Hover(col, row));
    assert_eq!(
        app.side_hover,
        Some(1),
        "hover should target 2nd sidebar item"
    );
}

// ---- separation: hover vs active/selected ---------------------------------------

#[test]
fn hover_does_not_change_active_panel() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let bar = app.command_bar_area.expect("bar");
    app.handle_event(Event::Hover(bar.x + 1, bar.y + 1));
    assert_eq!(
        app.active_panel,
        Panel::Side,
        "hover must not change active panel"
    );
}

#[test]
fn hover_does_not_change_keyboard_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 80, 24);
    let side = app.side_area.expect("side area");
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 1);
    // Hover na inny slot niż cursor.
    let col = side.x + 2;
    let row = side.y + 1 + (SIDEBAR_COUNT as u16); // ostatni slot
    app.handle_event(Event::Hover(col, row));
    assert_eq!(app.side_hover, Some(SIDEBAR_COUNT - 1));
    assert_eq!(app.side_cursor, 1, "keyboard cursor must not follow mouse");
}

#[test]
fn hover_does_not_change_selected_command() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let bar = app.command_bar_area.expect("bar");
    let slot_w = bar.w / COMMAND_COUNT as u16;
    let col = bar.x + slot_w * 3 + slot_w / 2; // 4th slot
    let row = bar.y + 1;
    app.handle_event(Event::Hover(col, row));
    assert_eq!(app.hovered_command, Some(3));
    assert_eq!(app.selected_command, 0, "selected must not change on hover");
}

// ---- no hover outside all areas -------------------------------------------------

#[test]
fn no_hover_outside_all_areas() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    // Kliknij w rogę (1,1) — header.
    app.handle_event(Event::Hover(1, 1));
    assert_eq!(app.hovered_command, None);
    assert_eq!(app.side_hover, None);
    assert_eq!(app.dashboard_hover, None);
}

// ---- clipping/responsiveness ----------------------------------------------------

#[test]
fn no_clipping_at_80_columns() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 80, 24);
    assert_no_line_overflows(&lines, 80, "80");
    assert_command_bar_within_viewport(&lines, 80, "80");
}

#[test]
fn no_clipping_at_120_columns() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24);
    assert_no_line_overflows(&lines, 120, "120");
    assert_command_bar_within_viewport(&lines, 120, "120");
}

#[test]
fn card_areas_stay_inside_main_area() {
    let dir = tempfile::tempdir().unwrap();
    for width in [80u16, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        app.core_panel = CorePanel::Main;
        let _ = render_lines(&mut app, width, 24);
        let main = app.main_area.expect("main area");
        for (name, card) in &app.card_areas {
            assert!(
                card.x >= main.x && card.x + card.w <= main.x + main.w,
                "{width}: card '{name}' exceeds main horizontally ({card:?} vs main {main:?})"
            );
            assert!(
                card.y >= main.y && card.y + card.h <= main.y + main.h,
                "{width}: card '{name}' exceeds main vertically"
            );
        }
    }
}
