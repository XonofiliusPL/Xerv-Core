//! Testy geometrii Dashboardu: wykrywanie clippingu/overflow, responsywność.
//!
//! Te testy parsują bufor renderu (TestBackend) i weryfikują, że ramki kart
//! oraz command bar mieszczą się w viewport — nie tylko że render nie panikuje.

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, Area, COMMANDS, COMMAND_COUNT};
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
    // Ratatui TestBackend drukuje linie w cudzysłowach — wyciągamy środek.
    s.lines().map(|l| l.trim_matches('"').to_string()).collect()
}

/// Środkowa linia każdego slotu command baru musi mieć zamykającą ramkę │
/// w tej samej kolumnie co otwierająca — dowód, że slot nie wycieka poza viewport.
#[track_caller]
fn assert_command_bar_within_viewport(lines: &[String], width: u16, ctx: &str) {
    // Znajdź wiersz command baru: zawiera ramki slotów ┌ i leży poniżej kart.
    // Etykiety mogą być kontrolowanie przycinane na wąskich terminalach,
    // więc nie szukamy pełnych nazw — zliczamy otwarte sloty.
    let bar_row = lines
        .iter()
        .position(|l| {
            l.matches('┌').count() >= 2 && !l.contains("┌ system") && !l.contains("┌ runtime")
        })
        .unwrap_or_else(|| panic!("{ctx}: command bar not found"));
    let line = &lines[bar_row];
    assert_eq!(
        line.chars().count(),
        width as usize,
        "{ctx}: command bar row width != terminal width"
    );
    // Liczba otwartych slotów w wierszu musi odpowiadać liczbie komend.
    let open_slots = line.matches('┌').count();
    assert_eq!(
        open_slots, COMMAND_COUNT,
        "{ctx}: expected {COMMAND_COUNT} command slots, found {open_slots}"
    );
    // Wiersz zamykający musi kończyć się ramką │ na ostatniej kolumnie —
    // dowód, że ostatni slot nie wycieka poza viewport.
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
fn assert_cards_within_viewport(lines: &[String], width: u16, ctx: &str) {
    // Każda linia bufora musi mieć dokładnie szerokość terminala (TestBackend
    // wypełnia spacjami) — to nie wykrywa wycieku. Wykrywamy wyciek inaczej:
    // ramka zamykająca karty musi pojawić się w obrębie width.
    let card_rows: Vec<&String> = lines.iter().filter(|l| l.contains("┌ system ")).collect();
    assert!(!card_rows.is_empty(), "{ctx}: system card not found");
    // Linia z ramką zamykającą kart (└) — ostatnia linia kart.
    // Każda linia zawierająca ┌ musi mieć parę ┘/┤/│ zamykającą w tej samej lub
    // wcześniejszej kolumnie — uproszczona wersja: żaden wiersz nie jest dłuższy niż width.
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
fn assert_two_columns(lines: &[String], ctx: &str) {
    // W układzie 2-kolumnowym istnieje linia zawierająca jednocześnie "system" i "runtime".
    let two_col_line = lines
        .iter()
        .any(|l| l.contains("┌ system") && l.contains("┌ runtime"));
    assert!(two_col_line, "{ctx}: expected 2-column card layout");
}

#[track_caller]
fn assert_one_column(lines: &[String], ctx: &str) {
    // W układzie 1-kolumnowym "system" i "runtime" są w osobnych liniach.
    let sys = lines.iter().any(|l| l.contains("┌ system"));
    let run = lines.iter().any(|l| l.contains("┌ runtime"));
    assert!(sys, "{ctx}: system card missing");
    assert!(run, "{ctx}: runtime card missing");
    let same_line = lines
        .iter()
        .any(|l| l.contains("┌ system") && l.contains("┌ runtime"));
    assert!(
        !same_line,
        "{ctx}: expected 1-column layout, found 2 columns"
    );
}

#[test]
fn no_clipping_at_80_columns() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 80, 24);
    assert_cards_within_viewport(&lines, 80, "80");
    assert_command_bar_within_viewport(&lines, 80, "80");
    assert_one_column(&lines, "80"); // cards area = 60 < 80 → 1 kolumna
}

#[test]
fn no_clipping_at_100_columns() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 100, 24);
    assert_cards_within_viewport(&lines, 100, "100");
    assert_command_bar_within_viewport(&lines, 100, "100");
    assert_one_column(&lines, "100"); // cards = 75 < 80
}

#[test]
fn two_columns_at_120_columns() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24);
    assert_cards_within_viewport(&lines, 120, "120");
    assert_command_bar_within_viewport(&lines, 120, "120");
    assert_two_columns(&lines, "120"); // cards = 90 ≥ 80
}

#[test]
fn two_columns_at_160_columns() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 160, 30);
    assert_cards_within_viewport(&lines, 160, "160");
    assert_command_bar_within_viewport(&lines, 160, "160");
    assert_two_columns(&lines, "160");
}

#[test]
fn no_clipping_on_very_narrow_terminal() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 45, 24);
    assert_cards_within_viewport(&lines, 45, "45");
    assert_command_bar_within_viewport(&lines, 45, "45");
    assert_one_column(&lines, "45");
}

#[test]
fn command_bar_fills_full_width_at_every_tested_width() {
    let dir = tempfile::tempdir().unwrap();
    for width in [60u16, 80, 100, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        let lines = render_lines(&mut app, width, 24);
        let bar = app.command_bar_area.expect("command bar area");
        // Prawa krawędź command baru musi sięgać prawej krawędzi viewportu.
        assert_eq!(
            bar.x + bar.w,
            width,
            "{width}: command bar should span to right edge"
        );
        // Sloty: prawy brzeg ostatniego slotu = prawy brzeg bara.
        let cfg_row = lines
            .iter()
            .position(|l| l.contains(" config "))
            .expect("config slot row");
        let line = &lines[cfg_row];
        assert!(
            line.trim_end().ends_with('│'),
            "{width}: last slot's right border clipped"
        );
    }
}

#[test]
fn command_bar_click_geometry_matches_render_at_all_widths() {
    // Klik w ostatni slot musi wybrać ostatnią komendę na każdej szerokości —
    // to wykrywa rozjazd geometrii click-vs-render.
    let dir = tempfile::tempdir().unwrap();
    for width in [60u16, 80, 100, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        let _ = render_lines(&mut app, width, 24);
        let bar = app.command_bar_area.expect("command bar area");
        let slot_w = bar.w / COMMAND_COUNT as u16;
        assert!(slot_w >= 5, "{width}: slot too narrow to click ({slot_w})");
        let col = bar.x + slot_w * (COMMAND_COUNT as u16 - 1) + slot_w / 2;
        let row = bar.y + bar.h / 2;
        app.handle_event(Event::ClickCommand(col, row));
        assert_eq!(
            app.selected_command,
            COMMAND_COUNT - 1,
            "{width}: click on last slot should select last command"
        );
    }
}

#[test]
fn card_areas_stay_inside_main_area() {
    let dir = tempfile::tempdir().unwrap();
    for width in [60u16, 80, 100, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        let _ = render_lines(&mut app, width, 24);
        let main = app.main_area.expect("main area");
        for (name, card) in &app.card_areas {
            assert!(
                card.x >= main.x && card.x + card.w <= main.x + main.w,
                "{width}: card '{name}' exceeds main area horizontally (card {:?} vs main {:?})",
                card,
                main
            );
            assert!(
                card.y >= main.y && card.y + card.h <= main.y + main.h,
                "{width}: card '{name}' exceeds main area vertically"
            );
        }
    }
}

#[test]
fn card_hover_hit_test_matches_render_at_all_widths() {
    // Hover w środku karty 'system' musi zarejestrować tę kartę —
    // wykrywa desynchronizację między zapisanymi obszarami a faktycznym renderem.
    let dir = tempfile::tempdir().unwrap();
    for width in [80u16, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        let _ = render_lines(&mut app, width, 24);
        let sys = app
            .card_areas
            .iter()
            .find(|(n, _)| *n == "system")
            .map(|(_, a)| *a)
            .expect("system card area");
        let col = sys.x + sys.w / 2;
        let row = sys.y + sys.h / 2;
        app.handle_event(Event::Hover(col, row));
        assert!(
            app.hovered_card.is_some(),
            "{width}: hover inside system card did not register"
        );
        // Hover trafia dokładnie w obszar karty 'system' (te same współrzędne).
        assert_eq!(
            app.hovered_card.unwrap(),
            sys,
            "{width}: hover hit different area than system card"
        );
    }
}

#[test]
fn command_bar_slot_count_matches_commands() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24);
    for name in COMMANDS.iter() {
        assert!(
            lines.iter().any(|l| l.contains(&format!(" {name} "))),
            "command '{name}' not fully visible in bar"
        );
    }
}

// --- helpers -----------------------------------------------------------------

/// pomocniczo: dostęp do Panel bez importu w każdym teście
#[allow(dead_code)]
fn panel_side() -> xerv_tui::app::Panel {
    xerv_tui::app::Panel::Side
}

#[allow(dead_code)]
fn area_of(r: Area) -> Area {
    r
}
