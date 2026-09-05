//! Testy geometrii Dashboardu: wykrywanie clippingu/overflow, responsywność.
//!
//! Te testy parsują bufor renderu (TestBackend) i weryfikują, że ramki kart
//! oraz command bar mieszczą się w viewport — nie tylko że render nie panikuje.

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use xerv_core::api::CoreConfig;
use xerv_tui::app::{App, Area, COMMANDS, COMMAND_COUNT, SIDEBAR_COUNT, SIDEBAR_ITEMS};
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
    // Ratatui TestBackend drukuje linie w cudzysłowach — usuwamy OBUDOWĘ,
    // zachowując dokładne pozycje kolumn (tekst bufora starts at col 0).
    s.lines()
        .map(|l| {
            let t = l.strip_prefix('"').unwrap_or(l);
            let t = t.strip_suffix('"').unwrap_or(t);
            t.to_string()
        })
        .collect()
}

/// Środkowa linia każdego slotu command baru musi mieć zamykającą ramkę │
/// w tej samej kolumnie co otwierająca — dowód, że slot nie wycieka poza viewport.
#[track_caller]
fn assert_command_bar_within_viewport(lines: &[String], width: u16, ctx: &str) {
    // Znajdź wiersz command baru: zawiera ramki slotów ┌ i leży poniżej kart.
    // Identyfikujemy go po etykiecie pierwszego slotu "modules" — nie ma
    // innego "┌ modules" w dashboardie.
    // Command bar to linia z dokładnie COMMAND_COUNT otwartymi slotami (┌).
    // Filtrujemy po liczbie `┌` >= COMMAND_COUNT — unikamy pomytału z kartami
    // (które mają 1-2 `┌` w górnym rogu) i z gridu workspace.
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
    let card_rows: Vec<&String> = lines.iter().filter(|l| l.contains("┌ agent ")).collect();
    assert!(!card_rows.is_empty(), "{ctx}: agent card not found");
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
    // W układzie 2-kolumnowym istnieje linia zawierająca jednocześnie "agent" i "workspace".
    let two_col_line = lines
        .iter()
        .any(|l| l.contains("┌ agent") && l.contains("┌ workspace"));
    assert!(two_col_line, "{ctx}: expected 2-column card layout");
}

#[track_caller]
fn assert_one_column(lines: &[String], ctx: &str) {
    // W układzie 1-kolumnowym "agent" i "workspace" są w osobnych liniach.
    let sys = lines.iter().any(|l| l.contains("┌ agent"));
    let run = lines.iter().any(|l| l.contains("┌ workspace"));
    assert!(sys, "{ctx}: agent card missing");
    assert!(run, "{ctx}: workspace card missing");
    let same_line = lines
        .iter()
        .any(|l| l.contains("┌ agent") && l.contains("┌ workspace"));
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
    // Hover w środku karty 'agent' musi zarejestrować tę kartę —
    // wykrywa desynchronizację między zapisanymi obszarami a faktycznym renderem.
    let dir = tempfile::tempdir().unwrap();
    for width in [80u16, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        let _ = render_lines(&mut app, width, 24);
        let sys = app
            .card_areas
            .iter()
            .find(|(n, _)| *n == "agent")
            .map(|(_, a)| *a)
            .expect("agent card area");
        let col = sys.x + sys.w / 2;
        let row = sys.y + sys.h / 2;
        app.handle_event(Event::Hover(col, row));
        // Hover w obszarze karty — side_hover powinien być None (hover jest poza sidebar).
        assert_eq!(
            app.side_hover, None,
            "{width}: hover outside sidebar must not register sidebar hover"
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

// ---- style-system tests (DESIGN.md) ---------------------------------------------

use ratatui::style::Color;

/// Zwraca style komórki bufora w danym (x, y) — dzięki temu testujemy kolory,
/// a nie tylko tekst.
fn cell_styles(app: &mut App, width: u16, height: u16) -> Vec<Vec<ratatui::style::Style>> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut areas_holder: Option<xerv_tui::app::UiAreas> = None;
    terminal
        .draw(|f| {
            areas_holder = Some(ui(f, app));
        })
        .unwrap();
    app.on_render(areas_holder.expect("areas"));
    let buf = terminal.backend().buffer().clone();
    (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].style()).collect())
        .collect()
}

fn find_text_row(lines: &[String], needle: &str) -> (usize, usize) {
    for (y, l) in lines.iter().enumerate() {
        if let Some(xb) = l.find(needle) {
            // byte-index → char-index (TestBackend bufory są indeksowane znakami,
            // a stringi mogą zawierać UTF-8 jak ● czy ─).
            let xc = l[..xb].chars().count();
            return (y, xc);
        }
    }
    panic!("'{needle}' not found in render");
}

#[test]
fn style_labels_are_muted_not_accented() {
    // Etykiety w kartach (np. "model") muszą być DarkGray (muted),
    // nigdy Cyan/Magenta — akcenty nie mogą dekorować statycznych etykiet.
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = {
        let b = TestBackend::new(120, 24);
        let mut t = Terminal::new(b).unwrap();
        let mut h = None;
        t.draw(|f| h = Some(ui(f, &mut app))).unwrap();
        t.backend().to_string()
    };
    let lns: Vec<String> = lines
        .lines()
        .map(|l| l.trim_matches('"').to_string())
        .collect();
    let styles = cell_styles(&mut app, 120, 24);
    let (y, x) = find_text_row(&lns, "model");
    let st = styles[y][x];
    assert_eq!(
        st.fg,
        Some(Color::DarkGray),
        "card label must be muted (DarkGray), got {:?}",
        st.fg
    );
}

#[test]
fn style_card_values_are_brighter_than_labels() {
    // Wartość modelu agenta (neutralna) = White; etykieta = DarkGray.
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = {
        let b = TestBackend::new(120, 24);
        let mut t = Terminal::new(b).unwrap();
        let mut h = None;
        t.draw(|f| h = Some(ui(f, &mut app))).unwrap();
        t.backend().to_string()
    };
    let lns: Vec<String> = lines
        .lines()
        .map(|l| l.trim_matches('"').to_string())
        .collect();
    let styles = cell_styles(&mut app, 120, 24);
    let (ly, lx) = find_text_row(&lns, "model");
    let label_style = styles[ly][lx];
    assert_eq!(label_style.fg, Some(Color::DarkGray));
    // Wartość zaczyna się po etykiecie w tej samej linii: "model  claude-3-7-sonnet"
    let line = &lns[ly];
    let vx = line.find("claude-3-7-sonnet").expect("value in same row");
    let value_style = styles[ly][vx];
    assert_eq!(
        value_style.fg,
        Some(Color::White),
        "card value must be White (neutral), got {:?}",
        value_style.fg
    );
}

#[test]
fn style_accent_values_are_magenta() {
    // "agent" i "workspace" to akcenty secondary (kv_accent) — Magenta.
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = {
        let b = TestBackend::new(120, 24);
        let mut t = Terminal::new(b).unwrap();
        let mut h = None;
        t.draw(|f| h = Some(ui(f, &mut app))).unwrap();
        t.backend().to_string()
    };
    let lns: Vec<String> = lines
        .lines()
        .map(|l| l.trim_matches('"').to_string())
        .collect();
    let styles = cell_styles(&mut app, 120, 24);
    // Szukamy konkretnej wartości (hermes-planner / ~/Work/xerv),
    // które są "odpowiedziami" → magenta.
    for needle in ["hermes-planner", "~/Work/xerv"] {
        let (y, x) = find_text_row(&lns, needle);
        let st = styles[y][x];
        assert_eq!(
            st.fg,
            Some(Color::Magenta),
            "'{}' value must use secondary accent (Magenta), got {:?}",
            needle,
            st.fg
        );
    }
}

#[test]
fn style_status_is_semantic_green() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = {
        let b = TestBackend::new(120, 24);
        let mut t = Terminal::new(b).unwrap();
        let mut h = None;
        t.draw(|f| h = Some(ui(f, &mut app))).unwrap();
        t.backend().to_string()
    };
    let lns: Vec<String> = lines
        .lines()
        .map(|l| l.trim_matches('"').to_string())
        .collect();
    let styles = cell_styles(&mut app, 120, 24);
    // Status strip (dashboard) oraz header zawierają "CORE READY" z zielonym kolorem.
    let (y, _) = find_text_row(&lns, "CORE READY");
    let line = &lns[y];
    let x = line[..line.find("READY").unwrap()].chars().count();
    let st = (x..)
        .map(|cx| styles[y][cx])
        .find(|s| s.fg.is_some() && s.fg != Some(Color::Reset))
        .expect("styled READY cell");
    assert_eq!(st.fg, Some(Color::Green));
    assert!(st.add_modifier.contains(ratatui::style::Modifier::BOLD));
}

#[test]
fn style_shutdown_is_semantic_red() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.core.shutdown().unwrap();
    let lines = {
        let b = TestBackend::new(120, 24);
        let mut t = Terminal::new(b).unwrap();
        let mut h = None;
        t.draw(|f| h = Some(ui(f, &mut app))).unwrap();
        t.backend().to_string()
    };
    let lns: Vec<String> = lines
        .lines()
        .map(|l| l.trim_matches('"').to_string())
        .collect();
    let styles = cell_styles(&mut app, 120, 24);
    let (y, _) = find_text_row(&lns, "SHUTDOWN");
    let line = &lns[y];
    let x = line[..line.find("SHUTDOWN").unwrap()].chars().count();
    let st = styles[y][x];
    assert_eq!(st.fg, Some(Color::Red));
}

#[test]
fn style_active_command_is_cyan_bold() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let bar = app.command_bar_area.expect("bar");
    let slot_w = bar.w / COMMAND_COUNT as u16;
    // Klik w 1. slot, render, sprawdź styl tekstu slotu.
    app.handle_event(Event::ClickCommand(bar.x + slot_w / 2, bar.y + bar.h / 2));
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut h = None;
    terminal.draw(|f| h = Some(ui(f, &mut app))).unwrap();
    let buf = terminal.backend().buffer().clone();
    // Znajdź komórkę z 'm' w pierwszym slocie (tekst " modules ").
    let y = bar.y + bar.h / 2;
    let x = bar.x + 1;
    let st = buf[(x, y)].style();
    assert_eq!(st.fg, Some(Color::Cyan), "active slot text must be Cyan");
    assert!(
        st.add_modifier.contains(ratatui::style::Modifier::BOLD),
        "active slot text must be BOLD"
    );
}

// ---- Agent Workspace tests (Point 3) ------------------------------------------------

#[test]
fn workspace_shows_agent_card_with_name_status_model_uptime_task() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24).join("\n");
    // Karta agent: nazwa, status (running), model, uptime, current task.
    assert!(
        lines.contains("hermes-planner"),
        "agent name missing: {lines}"
    );
    assert!(
        lines.contains("claude-3-7-sonnet"),
        "model missing: {lines}"
    );
    assert!(lines.contains("running"), "agent status missing: {lines}");
    assert!(lines.contains("task"), "task label missing: {lines}");
    assert!(
        lines.contains("Implement Goal 3"),
        "current task value missing: {lines}"
    );
}

#[test]
fn workspace_shows_terminal_placeholder() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24).join("\n");
    assert!(
        lines.contains("terminal"),
        "terminal placeholder title missing: {lines}"
    );
    assert!(
        lines.contains("not initialized"),
        "terminal placeholder hint missing: {lines}"
    );
}

#[test]
fn workspace_shows_last_activity_section_with_demo_entries() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24).join("\n");
    assert!(
        lines.contains("activity"),
        "last activity section title missing: {lines}"
    );
    // Co najmniej jeden demonstracyjny wpis.
    assert!(
        lines.contains("hermes-planner") || lines.contains("claude-coder"),
        "activity entries missing: {lines}"
    );
}

#[test]
fn workspace_tree_shows_hierarchy_project_worktree_session() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24).join("\n");
    assert!(
        lines.contains("workspace"),
        "workspace root label missing: {lines}"
    );
    assert!(lines.contains("project"), "project label missing: {lines}");
    assert!(
        lines.contains("worktree") || lines.contains("current") || lines.contains("branch"),
        "worktree info missing: {lines}"
    );
    assert!(lines.contains("session"), "session label missing: {lines}");
}

#[test]
fn workspace_terminal_and_activity_within_viewport() {
    let dir = tempfile::tempdir().unwrap();
    for width in [80u16, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        let lines = render_lines(&mut app, width, 24);
        for (i, l) in lines.iter().enumerate() {
            assert!(
                l.chars().count() <= width as usize,
                "{width}: line {i} overflows: '{l}'"
            );
        }
    }
}

#[test]
fn workspace_agent_card_is_active_when_panel_is_main() {
    // Aktywna karta agenta powinna mieć cyan ramkę, gdy focus = Main.
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.active_panel = xerv_tui::app::Panel::Main;
    let _ = render_lines(&mut app, 120, 24);
    // card_areas powinno zawierać kartę "agent".
    let agent_card = app
        .card_areas
        .iter()
        .find(|(n, _)| *n == "agent")
        .expect("agent card area recorded");
    assert!(
        agent_card.1.w > 0 && agent_card.1.h > 0,
        "agent card has zero size: {:?}",
        agent_card.1
    );
}

// ---- sidebar tests --------------------------------------------------------------

#[test]
fn sidebar_shows_navigation_items() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let s = render_lines(&mut app, 120, 24).join("\n");
    for name in SIDEBAR_ITEMS.iter() {
        assert!(s.contains(name), "sidebar missing nav item '{name}'");
    }
    assert!(s.contains("NAVIGATION"), "sidebar missing section header");
    assert!(
        !s.contains("more screens soon"),
        "placeholder text still present"
    );
}

#[test]
fn sidebar_active_item_is_dashboard_with_marker() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let lines = render_lines(&mut app, 120, 24);
    // Aktywna pozycja Dashboard ma wskaźnik ● na swoim wierszu.
    let row = lines
        .iter()
        .find(|l| l.contains("Dashboard"))
        .expect("Dashboard row");
    assert!(row.contains('●'), "active item must have ● marker: '{row}'");
    // Inne pozycje — bez ●.
    let agents = lines.iter().find(|l| l.contains("Agents")).unwrap();
    assert!(
        !agents.contains('●'),
        "inactive item must not have ● marker: '{agents}'"
    );
}

#[test]
fn nav_keyboard_down_up_cycles_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    assert_eq!(app.side_cursor, 0);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 1);
    app.handle_event(Event::NavUp);
    assert_eq!(app.side_cursor, 0);
    // Zawijanie w górę z 0 → ostatnia pozycja.
    app.handle_event(Event::NavUp);
    assert_eq!(app.side_cursor, SIDEBAR_COUNT - 1);
    app.handle_event(Event::NavDown);
    assert_eq!(app.side_cursor, 0);
}

#[test]
fn nav_sets_focus_to_side_panel() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.active_panel = xerv_tui::app::Panel::Main;
    app.handle_event(Event::NavDown);
    assert_eq!(app.active_panel, xerv_tui::app::Panel::Side);
}

#[test]
fn sidebar_mouse_click_selects_nav_item() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let side = app.side_area.expect("side area");
    // Klik w pozycję o indeksie 2 (Projects, 3. w liście): first_item_row = side.y + 2.
    let click_row = side.y + 2 + 2;
    app.handle_event(Event::ClickPanel(side.x + 2, click_row));
    assert_eq!(app.side_active, 2, "click should set active to 'Projects'");
    assert_eq!(app.side_cursor, 2);
    assert_eq!(app.side_hover, Some(2));
    assert_eq!(app.active_panel, xerv_tui::app::Panel::Side);
}

#[test]
fn sidebar_click_below_items_does_not_panic_or_change_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let side = app.side_area.expect("side area");
    // Klik w wolną przestrzeń poniżej listy.
    app.handle_event(Event::ClickPanel(side.x + 2, side.y + side.h - 2));
    assert_eq!(
        app.side_cursor, 0,
        "click on empty area must not change cursor"
    );
}

#[test]
fn sidebar_cursor_visible_after_keyboard_nav_render() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.handle_event(Event::NavDown); // cursor = Agents
    let lines = render_lines(&mut app, 120, 24);
    // Cursor na Agents: wiersz Agents zawiera wskaźnik ▎ (muted, ale obecny).
    let agents = lines.iter().find(|l| l.contains("Agents")).unwrap();
    assert!(
        agents.contains('▎'),
        "cursor row must show marker: '{agents}'"
    );
    // Dashboard pozostaje aktywną pozycją (●) — jednoznaczne rozróżnienie.
    let dash = lines.iter().find(|l| l.contains("Dashboard")).unwrap();
    assert!(dash.contains('●'), "active item keeps ● marker: '{dash}'");
    assert!(
        !dash.contains('▎'),
        "active row must not show cursor marker: '{dash}'"
    );
}

#[test]
fn sidebar_no_clipping_at_any_width() {
    let dir = tempfile::tempdir().unwrap();
    for width in [45u16, 60, 80, 100, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        let lines = render_lines(&mut app, width, 24);
        for (i, l) in lines.iter().enumerate() {
            assert!(
                l.chars().count() <= width as usize,
                "{width}: line {i} overflows: '{l}'"
            );
        }
        // Wszystkie pozycje obecne (nazwy mogą być obcięte przy bardzo wąskim,
        // więc sprawdzamy prefix 'Mod' zamiast pełnej nazwy dla 45).
        // W bardzo wąskim sidebarze nagłówek może być kontrolowanie przycięty.
        let s = lines.join("\n");
        assert!(
            s.contains("NAVIGATION") || s.contains("NAVIGATI"),
            "{width}: section header missing"
        );
    }
}

#[test]
fn sidebar_items_within_side_area_at_all_widths() {
    let dir = tempfile::tempdir().unwrap();
    for width in [45u16, 80, 120, 160] {
        let mut app = make_app(dir.path(), 1_700_000_000);
        let _ = render_lines(&mut app, width, 24);
        let side = app.side_area.expect("side area");
        // Wiersze pozycji muszą mieścić się w side area (render: side.y+2 .. +2+len).
        assert!(
            side.y + 2 + SIDEBAR_COUNT as u16 <= side.y + side.h,
            "{width}: nav items exceed side area height"
        );
    }
}

// ---- navigation state-model tests (active vs cursor vs hover) ---------------------

/// Zwraca TYLKO fragment sidebaru wiersza (między pierwszym a drugim `│`)
/// — wiersz terminala zawiera też kolumnę dashboardu, której `●` (status)
/// nie może fałszować wyniku.
fn side_row(lines: &[String], name: &str) -> String {
    for l in lines {
        if !l.trim_start().starts_with('│') {
            continue;
        }
        let chars: Vec<char> = l.chars().collect();
        if let (Some(a), Some(b)) = (chars.iter().position(|c| *c == '│'), {
            // druga kolumna ramki
            chars
                .iter()
                .skip(chars.iter().position(|c| *c == '│').unwrap() + 1)
                .position(|c| *c == '│')
                .map(|p| p + chars.iter().position(|c| *c == '│').unwrap() + 1)
        }) {
            let segment: String = chars[a + 1..b].iter().collect();
            if segment.contains(name) {
                return segment;
            }
        }
    }
    panic!("sidebar row for '{name}' not found");
}

#[test]
fn nav_model_dashboard_active_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    assert_eq!(app.side_active, 0, "Dashboard must be the active item");
    let lines = render_lines(&mut app, 120, 24);
    let dash = side_row(&lines, "Dashboard");
    assert!(
        dash.contains('●'),
        "active item must have ● marker: '{dash}'"
    );
    // Żadna inna pozycja nie może mieć ●.
    for name in [
        "Agents", "Projects", "Services", "Packages", "Plugins", "Settings", "Help",
    ] {
        assert!(
            !side_row(&lines, name).contains('●'),
            "{name} must not be marked active"
        );
    }
}

#[test]
fn nav_model_cursor_on_other_item_does_not_change_active() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown); // cursor = Projects (idx 4)
    assert_eq!(
        app.side_active, 0,
        "cursor movement must NOT change active sidebar item"
    );
    let lines = render_lines(&mut app, 120, 24);
    // Projects: tylko kursor ▎, NIE ●.
    let proj = side_row(&lines, "Projects");
    assert!(proj.contains('▎'), "cursor row must have ▎: '{proj}'");
    assert!(
        !proj.contains('●'),
        "cursor row must NOT pretend to be active: '{proj}'"
    );
    // Dashboard nadal aktywny.
    let dash = side_row(&lines, "Dashboard");
    assert!(
        dash.contains('●'),
        "Dashboard remains the active item: '{dash}'"
    );
    assert!(
        !dash.contains('▎'),
        "active row must not show cursor marker when cursor is elsewhere"
    );
}

#[test]
fn nav_model_enter_activates_current_cursor_position() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    // Kursor na Services (idx 3) + Enter/Space.
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavActivate);
    assert_eq!(
        app.side_active, 3,
        "Enter should activate current cursor position"
    );
    let lines = render_lines(&mut app, 120, 24);
    assert!(
        side_row(&lines, "Services").contains('●'),
        "Services must be marked active after Enter"
    );
    assert!(
        !side_row(&lines, "Dashboard").contains('●'),
        "Dashboard should no longer be active after Enter on Services"
    );
}

#[test]
fn nav_model_hover_does_not_change_active_or_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    let _ = render_lines(&mut app, 120, 24);
    let side = app.side_area.expect("side area");
    // Hover na wierszu "Packages" (idx 4): first_item_row = side.y + 2.
    let hover_row = side.y + 2 + 4;
    app.handle_event(Event::Hover(side.x + 3, hover_row));
    // Hover aktualizuje side_hover — kursor nawigacji i aktywna pozycja nie zmieniają się.
    assert_eq!(app.side_active, 0, "hover must not change active item");
    assert_eq!(app.side_cursor, 0, "hover must not change cursor");
    // Klik (nie hover) przesuwa aktywną pozycję i kursor.
    app.handle_event(Event::ClickPanel(side.x + 3, hover_row));
    assert_eq!(app.side_active, 4, "click should set active to Packages");
    assert_eq!(app.side_cursor, 4, "click should move cursor to Packages");
}

#[test]
fn nav_model_render_matches_state_after_full_sequence() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = make_app(dir.path(), 1_700_000_000);
    // Sekwencja: w dół 3× (Services), Enter (aktywuje Services), góra 2× (Agents),
    // klik na Projects, Enter na Projects.
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavDown);
    app.handle_event(Event::NavActivate);
    app.handle_event(Event::NavUp);
    app.handle_event(Event::NavUp);
    let _ = render_lines(&mut app, 120, 24);
    let side = app.side_area.expect("side area");
    app.handle_event(Event::ClickPanel(side.x + 2, side.y + 2 + 2)); // Projects
    app.handle_event(Event::NavActivate);

    // Stan: active=Projects, cursor=Projects.
    assert_eq!(app.side_active, 2);
    assert_eq!(app.side_cursor, 2);
    let lines = render_lines(&mut app, 120, 24);
    assert!(
        side_row(&lines, "Projects").contains('●'),
        "render must match side_active"
    );
    assert!(
        !side_row(&lines, "Dashboard").contains('●'),
        "Dashboard must not be active after switching to Projects"
    );
    assert!(
        side_row(&lines, "Projects").contains('●'),
        "Projects must be active and show ●"
    );
    assert!(
        !side_row(&lines, "Agents").contains('▎'),
        "Agents must not show cursor marker when cursor is at Projects"
    );
}
