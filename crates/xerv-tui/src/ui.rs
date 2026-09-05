use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{
    AgentStatus, App, Area, Panel, UiAreas, WorkspaceModel, WorktreeKind, COMMAND_ACTIONS,
    COMMAND_COUNT, SIDEBAR_COUNT, SIDEBAR_ITEMS,
};

const HEADER_HEIGHT: u16 = 4;
const FOOTER_HEIGHT: u16 = 1;
const SIDE_WIDTH_PERCENT: u16 = 25;
const MIN_USABLE_WIDTH: u16 = 30;
const COMMAND_BAR_HEIGHT: u16 = 3;

/// Rysuje cały ekran i zwraca obszary interaktywne dla `App::on_render`.
pub fn ui(f: &mut Frame, app: &mut App) -> UiAreas {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(1),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(area);

    draw_header(f, chunks[0], app);
    draw_footer(f, chunks[2]);

    // Najpierw dzielimy main na: obszar kart (Min) + command bar (Length).
    // Wcześniej dashboard rysował się na CAŁYM main_area, a command bar
    // nadpisywał jego dolne linie — stąd clipping ramek.
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(COMMAND_BAR_HEIGHT)])
        .split(chunks[1]);
    // Command bar is 3 rows, fixed.
    // Command bar is 3 rows, fixed below the main content.
    let cmd_area = main_chunks[1];
    let (side_rect, cards) = draw_main(f, main_chunks[0], app);
    draw_command_bar(f, cmd_area, app);

    UiAreas {
        side: Some(to_area(side_rect)),
        main: Some(to_area(main_chunks[0])),
        command_bar: Some(to_area(cmd_area)),
        cards,
    }
}

// ---- helpers ----------------------------------------------------------------

fn to_area(r: Rect) -> Area {
    Area {
        x: r.x,
        y: r.y,
        w: r.width,
        h: r.height,
    }
}

/// Skraca ścieżkę do ostatnich N segmentów.
fn short_path(path: &std::path::Path, max_segments: usize) -> String {
    let parts: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.len() <= max_segments {
        return parts.join("/");
    }
    let kept: Vec<_> = parts[parts.len() - max_segments..].to_vec();
    format!("…/{}", kept.join("/"))
}

/// Człowiekowo-czytelny uptime.
fn format_uptime(started_at_unix: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(started_at_unix);
    let secs = now.saturating_sub(started_at_unix);
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86_400 {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d{}h", secs / 86_400, (secs % 86_400) / 3600)
    }
}

// ---- design tokens (DESIGN.md) -------------------------------------------------
//
// Każdy kolor z DESIGN.md ma odpowiadającą funkcję stylu. Jedno miejsce prawdy:
// zmiana identyfikacji = zmiana tutaj (i w DESIGN.md), nie rozproszone literały.

/// primary #00D7D7 (terminal Cyan) — wyłącznie akcent interaktywny.
fn accent_primary() -> Style {
    Style::default().fg(Color::Cyan)
}

/// primary + waga (BOLD) — brand, aktywny element, status napisu interakcji.
fn accent_primary_bold() -> Style {
    accent_primary().add_modifier(Modifier::BOLD)
}

/// secondary #D787D7 (terminal Magenta) — wybrane informacje (wersja API,
/// wartości „odpowiedzi"). Nigdy ramki interaktywne.
fn accent_secondary() -> Style {
    Style::default().fg(Color::Magenta)
}

/// value #FFFFFF (terminal White) — wartości danych; jaśniejsze niż etykieta,
/// wyraźnie odróżnione od pustego tła.
fn value_style() -> Style {
    Style::default().fg(Color::White)
}

/// muted #585858 (terminal DarkGray) — etykiety, separatory, idle ramki.
fn muted_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// success/danger — wyłącznie semantyka statusu READY/SHUTDOWN.
fn status_style(ok: bool) -> Style {
    if ok {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    }
}

fn card_border_style(active: bool, hovered: bool) -> Style {
    if active {
        accent_primary()
    } else if hovered {
        accent_secondary()
    } else {
        muted_style()
    }
}

/// Ramka karty — aktywna (fokus klawiatury) cyan, hover magenta, spoczynek dark gray.
fn card_block<'a>(title: &'a str, active: bool, hovered: bool) -> Block<'a> {
    let mut b = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(card_border_style(active, hovered));
    if active || hovered {
        b = b.title_style(card_border_style(active, hovered).add_modifier(Modifier::BOLD));
    } else {
        b = b.title_style(muted_style());
    }
    b
}

// ---- header (bez zmian względem poprzedniej wersji) -----------------------------

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    if area.width < MIN_USABLE_WIDTH {
        let title = Line::from(Span::styled(" Xerv", accent_primary_bold()));
        let block = Block::default().borders(Borders::BOTTOM);
        f.render_widget(Paragraph::new(title).block(block), area);
        return;
    }

    let api = app.core.api_version();
    let cfg = app.core.config();
    let st = app.core.state();
    let shutdown = app.core.is_shutdown();

    let brand = Line::from(vec![
        Span::raw(" "),
        Span::styled("XERV", accent_primary_bold()),
        Span::styled("  •  ", muted_style()),
        Span::styled(
            format!("api {}.{}.{}", api.major, api.minor, api.patch),
            accent_secondary(),
        ),
        Span::styled("  •  core ", muted_style()),
        if shutdown {
            Span::styled("SHUTDOWN", status_style(false).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("READY", status_style(true).add_modifier(Modifier::BOLD))
        },
        Span::styled(format!("  boot #{}", st.boot_count), muted_style()),
    ]);

    let separator = Line::from(Span::styled("─".repeat(area.width as usize), muted_style()));

    let meta = Line::from(vec![
        Span::raw(" "),
        Span::styled("data ", muted_style()),
        Span::styled(short_path(&cfg.data_dir, 3), accent_secondary()),
        Span::styled("  schema v", muted_style()),
        Span::styled(st.schema_version.to_string(), accent_secondary()),
        Span::styled("  uptime ", muted_style()),
        Span::styled(format_uptime(st.started_at_unix), accent_secondary()),
        Span::styled("  log ", muted_style()),
        Span::styled(cfg.log_level.clone(), accent_secondary()),
    ]);

    let lines = vec![brand, separator, meta];
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(muted_style());
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

// ---- main split ----------------------------------------------------------------

fn draw_main(f: &mut Frame, area: Rect, app: &App) -> (Rect, Vec<(&'static str, Area)>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(SIDE_WIDTH_PERCENT),
            Constraint::Min(1),
        ])
        .split(area);

    draw_side(f, chunks[0], app);
    let cards = draw_dashboard(f, chunks[1], app);
    (chunks[0], cards)
}

const ITEM_MARKER_ACTIVE: &str = "\u{25CF} "; // ● : aktywny ekran (cyan)
const ITEM_MARKER_CURSOR: &str = "\u{258E} "; // ▎ : kursor nawigacji (white) — gdy active_panel == Side
const ITEM_MARKER_HOVER: &str = "\u{25CB} "; // ○ : hover myszy (magenta, niezmienia aktywnego)

/// Pozycja sidebaru: imię + styl + marker — renderowana z pojedynczego źródła.
struct SidebarItem {
    name: &'static str,
    style: Style,
    marker: &'static str,
}

/// Buduje listę pozycji na podstawie stanu aplikacji (active + cursor + hover).
fn sidebar_items(app: &App) -> Vec<SidebarItem> {
    SIDEBAR_ITEMS
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_active = app.side_active == i;
            let is_cursor = app.side_cursor == i && app.active_panel == Panel::Side;
            let is_hover = app.side_hover == Some(i);
            let (marker, style) = if is_active {
                (ITEM_MARKER_ACTIVE, accent_primary_bold())
            } else if is_cursor {
                (ITEM_MARKER_CURSOR, value_style())
            } else if is_hover {
                (ITEM_MARKER_HOVER, accent_secondary())
            } else {
                ("  ", muted_style())
            };
            SidebarItem {
                name,
                style,
                marker,
            }
        })
        .collect()
}

/// Sidebar — nawigacja Xerv. Single source: SIDEBAR_ITEMS.
/// Aktywna pozycja: cyan (●). Kursor nawigacji (↑/↓/klik): white (▎).
/// Hover myszy: magenta (○) — nie zmienia aktywnej pozycji.
fn draw_side(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" xerv ")
        .border_style(if app.active_panel == Panel::Side {
            accent_primary()
        } else {
            muted_style()
        });
    f.render_widget(block, area);

    // Wewnętrzna przestrzeń (za borderami):
    // x+1, y+1, width-2, height-2.
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // nagłówek NAVIGATION
            Constraint::Length(SIDEBAR_COUNT as u16),
            Constraint::Min(1), // wolna przestrzeń
        ])
        .split(inner);

    // Nagłówek sekcji.
    let head = Line::from(Span::styled(" NAVIGATION", muted_style()));
    f.render_widget(Paragraph::new(head), rows[0]);

    // Każda pozycja renderowana z single source (sidebar_items).
    let items = sidebar_items(app);
    for (i, item) in items.into_iter().enumerate() {
        let y = rows[1].y + i as u16;
        let line = Line::from(vec![
            Span::raw(item.marker).style(item.style),
            Span::styled(item.name.to_string(), item.style),
        ]);
        let rect = Rect {
            x: area.x + 1,
            y,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        f.render_widget(Paragraph::new(line), rect);
    }
}

// ---- dashboard -----------------------------------------------------------------
//
// Główny panel Dashboardu = Agent Workspace (Punkt 3).
//
// Layout pionowy:
//   1. Status strip (1 wiersz) — zachowany z poprzedniej wersji.
//   2. Workspace grid: dwie kolumny — Agent info (aktywny) | Workspace tree.
//   3. Terminal placeholder — wyraźne miejsce na przyszły terminal.
//   4. Last activity — lista demonstracyjnych wpisów.
//
// Karty w gridzie oraz terminal/activity mają nagłówek z ikoną.
// Karty aktywne (agent) mają ramkę cyan; inne muted — zgodnie z DESIGN.md.

const TERMINAL_PLACEHOLDER_HEIGHT: u16 = 4;
const ACTIVITY_MAX_ROWS_SMALL: usize = 3; // demo: 3 wpisy

/// Agent Workspace dashboard: status strip + workspace grid + terminal + activity.
/// Zwraca obszary kart (dla hit-testów w przyszłości).
fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) -> Vec<(&'static str, Area)> {
    // Układ pionowy:
    //   1. Status strip (1 wiersz)
    //   2. Agent card (9 wierszy max — name/model/status/uptime/sep/task)
    //   3. Workspace tree (Min — dostaje resztę)
    //   4. Terminal placeholder (8 wierszy)
    //   5. Last activity (Min, max 3 wpisy)
    //
    // Priorytet: agent card > terminal > workspace tree > last activity.
    // Gdy wysokość jest mała, workspace tree i last activity mogą ustąpić —
    // agent card i terminal dostają priorytet (stałe LENGTH).
    // Agent card: 6 pól treści (agent, model, status, uptime, task + separator)
    // + 2 border = 8. Workspace tree i last activity mogą ustąpić priorytetem.
    const AGENT_CARD_H: u16 = 8;
    const TERM_ROWS: u16 = TERMINAL_PLACEHOLDER_HEIGHT + 2; // = 5
    let min_needed = 1 + AGENT_CARD_H + 1 + TERM_ROWS + 1;
    let has_activity = area.height >= min_needed;
    let has_tree = area.height >= 1 + AGENT_CARD_H + TERM_ROWS;

    let constraints: Vec<Constraint> = {
        let mut c = vec![
            Constraint::Length(1),            // status strip
            Constraint::Length(AGENT_CARD_H), // agent card
        ];
        if has_tree {
            c.push(Constraint::Min(0)); // workspace tree (0 = may collapse if no space)
        }
        c.push(Constraint::Length(TERM_ROWS));
        if has_activity {
            c.push(Constraint::Min(0)); // last activity (0 = may collapse)
        }
        c
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_slice())
        .split(area);

    draw_status_strip(f, chunks[0], app);

    // Agent card zawsze na chunks[1]. Workspace tree na chunks[2] (jeśli exists).
    let mut areas = Vec::new();
    let ws = &app.workspace;
    if has_tree {
        draw_agent_card(f, chunks[1], ws, app, &mut areas);
        draw_workspace_tree_card(f, chunks[2], ws, &mut areas);
        draw_terminal_placeholder(f, chunks[3]);
        if has_activity {
            draw_last_activity(f, chunks[4], app);
        }
    } else {
        draw_agent_card(f, chunks[1], ws, app, &mut areas);
        draw_terminal_placeholder(f, chunks[2]);
    }
    areas
}

/// Karta aktywnego agenta — pełne informacje: nazwa, status, model, uptime, task.
fn draw_agent_card(
    f: &mut Frame,
    area: Rect,
    ws: &WorkspaceModel,
    app: &App,
    areas: &mut Vec<(&'static str, Area)>,
) {
    let agent = &ws.agents[ws.active_agent.min(ws.agents.len().saturating_sub(1))];
    let is_active = app.active_panel == Panel::Main;

    let lines = vec![
        kv_accent("agent", &agent.name),
        kv("model", &agent.model),
        status_kv_agent(agent.status),
        kv("uptime", &agent.uptime.format()),
        kv("task", &agent.current_task),
    ];
    render_card_named(f, area, "agent", lines, is_active, false, areas);
}

/// Karta drzewa workspace — hierarchia: Project → Worktree → Session → Agent.
fn draw_workspace_tree_card(
    f: &mut Frame,
    area: Rect,
    ws: &WorkspaceModel,
    areas: &mut Vec<(&'static str, Area)>,
) {
    let mut lines: Vec<Line<'static>> = Vec::new();

    kv_root(&mut lines, "workspace", &ws.workspace_name);

    for proj in &ws.projects {
        kv_indent(&mut lines, 1, "project", &proj.name);
        for wt in &proj.worktrees {
            let kind_str = match wt.kind {
                WorktreeKind::Current => "current",
                WorktreeKind::Branch => "branch",
                WorktreeKind::Detached => "detached",
            };
            let marker = if wt.active { "● " } else { "  " };
            kv_indent_marker(&mut lines, 2, kind_str, &wt.path, marker);
        }
    }

    separator_line();
    for sess in &ws.sessions {
        let marker = if sess.active { "● " } else { "  " };
        kv_indent_marker(&mut lines, 1, "session", &sess.name, marker);
    }

    separator_line();
    for (i, agent) in ws.agents.iter().enumerate() {
        let marker = if i == ws.active_agent { "● " } else { "  " };
        kv_indent_marker(&mut lines, 1, "agent", &agent.name, marker);
    }

    render_card_named(f, area, "workspace", lines, false, false, areas);
}

/// Placeholder terminalu — miejsce na przyszły terminal.
/// Nie uruchamia shell ani procesu. Wyraźny obszar z nagłówkiem i ikoną.
fn draw_terminal_placeholder(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" terminal ")
        .title_style(muted_style().add_modifier(Modifier::BOLD))
        .border_style(muted_style());

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let lines = vec![
        Line::from(Span::styled(
            "  $ shell — not initialized (Point 3)",
            muted_style(),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "  [ press `e` to enter (future) | output will appear here ]",
            muted_style(),
        )),
    ];

    f.render_widget(
        Paragraph::new(lines).block(block),
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(0),
        },
    );

    // Upewnijmy się, że cały placeholder mieści się w viewport.
    let _ = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height,
    };
}

/// Sekcja "last activity" — demonstracyjne wpisy z timestampem.
fn draw_last_activity(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" last activity ")
        .title_style(accent_secondary().add_modifier(Modifier::BOLD))
        .border_style(muted_style());

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if app.activity.is_empty() {
        let lines = vec![Line::from(Span::styled(
            "  (no activity yet)",
            muted_style(),
        ))];
        f.render_widget(Paragraph::new(lines).block(block), inner);
        return;
    }

    // Ograniczamy do wystarczającej liczby wierszy na małym terminalu.
    let max_rows = (inner.height as usize)
        .saturating_sub(2)
        .min(ACTIVITY_MAX_ROWS_SMALL);
    let entries: Vec<_> = app.activity.iter().take(max_rows).collect();

    let lines: Vec<Line<'static>> = entries
        .iter()
        .map(|e| {
            Line::from(vec![
                Span::styled("  ", muted_style()),
                Span::styled(e.timestamp.clone(), muted_style()),
                Span::styled("  │  ", muted_style()),
                Span::styled(e.content.clone(), value_style()),
            ])
        })
        .collect();

    let mut lines = lines;
    // Wyrównaj do dostępnej wysokości (puste spacjery na dole).
    while lines.len() < inner.height as usize {
        lines.push(Line::from(Span::raw("")));
    }

    f.render_widget(Paragraph::new(lines).block(block), inner);
}

/// Status agenta z semantycznym kolorem (green=działa, yellow=idle, red=error).
fn status_kv_agent(status: AgentStatus) -> Line<'static> {
    let (color, text) = match status {
        AgentStatus::Running => (Color::Green, "running"),
        AgentStatus::Idle => (Color::Yellow, "idle"),
        AgentStatus::Done => (Color::Green, "done"),
        AgentStatus::Error => (Color::Red, "error"),
    };
    let marker = status.marker();
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{:<12}", "status"), muted_style()),
        Span::styled(
            format!("{marker} {text}"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

/// kv dla poziomu 0 (root workspace) — wartość w accent_secondary.
fn kv_root(lines: &mut Vec<Line<'static>>, label: &str, value: &str) {
    lines.push(kv_accent(label, value));
}

/// kv z wcięciem (dla projektów/worktreeów/sessions/agents w drzewie).
fn kv_indent(lines: &mut Vec<Line<'static>>, depth: usize, label: &str, value: &str) {
    lines.push(Line::from(vec![
        Span::raw("  ".repeat(depth + 1)),
        Span::styled(format!("{label:<10}"), muted_style()),
        Span::styled(value.to_string(), value_style()),
    ]));
}

/// kv z wcięciem + markerem (● aktywny / spacja nieaktywny).
fn kv_indent_marker(
    lines: &mut Vec<Line<'static>>,
    depth: usize,
    label: &str,
    value: &str,
    marker: &str,
) {
    lines.push(Line::from(vec![
        Span::raw("  ".repeat(depth + 1)),
        Span::styled(marker.to_string(), value_style()),
        Span::styled(format!("{label:<10}"), muted_style()),
        Span::styled(value.to_string(), value_style()),
    ]));
}

/// Renderuje kartę z jawnym aktywnym/hover stanem i zapisuje obszar.
fn render_card_named(
    f: &mut Frame,
    area: Rect,
    title: &'static str,
    lines: Vec<Line<'static>>,
    active: bool,
    hovered: bool,
    areas: &mut Vec<(&'static str, Area)>,
) {
    let block = card_block(title, active, hovered);
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
    areas.push((title, to_area(area)));
}

/// Jednolinijsowy pasek statusu: status semantyczny, wartości neutralne,
/// separatory muted. Spójny z headerem (label→value, te same style).
fn draw_status_strip(f: &mut Frame, area: Rect, app: &App) {
    let shutdown = app.core.is_shutdown();
    let st = app.core.state();
    let api = app.core.api_version();

    let status = if shutdown {
        Line::from(vec![
            Span::styled("● ", status_style(false)),
            Span::styled(
                "CORE SHUTDOWN",
                status_style(false).add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("● ", status_style(true)),
            Span::styled(
                "CORE READY",
                status_style(true).add_modifier(Modifier::BOLD),
            ),
        ])
    };

    let label = |t: &str| Span::styled(format!("{t} "), muted_style());
    let value = |t: String| Span::styled(t, value_style());

    let mut line_spans = vec![Span::raw(" ")];
    line_spans.extend(status.spans);
    line_spans.push(Span::styled("  │  ", muted_style()));
    line_spans.push(label("api"));
    line_spans.push(value(format!("{}.{}.{}", api.major, api.minor, api.patch)));
    line_spans.push(Span::styled("  │  ", muted_style()));
    line_spans.push(label("boot #"));
    line_spans.push(value(st.boot_count.to_string()));
    line_spans.push(Span::styled("  │  ", muted_style()));
    line_spans.push(label("uptime"));
    line_spans.push(value(format_uptime(st.started_at_unix)));

    f.render_widget(Paragraph::new(Line::from(line_spans)), area);
}

/// Cienki separator wewnątrz karty (dim).
fn separator_line() -> Line<'static> {
    Line::from(Span::styled("  ························", muted_style()))
}

/// Para etykieta→wartość. Etykieta muted, wartość neutralna (White).
fn kv<'a>(label: &str, value: &str) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{label:<12}"), muted_style()),
        Span::styled(value.to_string(), value_style()),
    ])
}

/// Para etykieta→wartość z akcentem secondary (magenta) — dla „odpowiedzi":
/// kluczowych wartości, na które patrzy użytkownik w pierwszej kolejności.
fn kv_accent<'a>(label: &str, value: &str) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{label:<12}"), muted_style()),
        Span::styled(value.to_string(), accent_secondary()),
    ])
}

// ---- command bar ----------------------------------------------------------------

/// Command Bar / Action Bar — komplementuje Sidebar (główna nawigacja).
/// Prezentuje dostępne akcje dla bieżącego kontekstu z ikoną + skrótem.
/// Każdy slot jest klikalny myszą i obsługiwany klawiaturą (←→/Enter).
/// Nie jest to druga nawigacacja — to pasek akcji, przygotowany pod przyszłe
/// akcje agenta/workspace. Sloty używają ikono (Nerd Font glyphy) + labelu.
fn draw_command_bar(f: &mut Frame, area: Rect, app: &App) {
    // Ratio zapewnia, że sloty dokładnie wypełniają szerokość (bez zaokrągleń).
    let slots = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Ratio(1, COMMAND_COUNT as u32);
            COMMAND_COUNT
        ])
        .split(area);

    for (i, action) in COMMAND_ACTIONS.iter().enumerate() {
        let selected = app.selected_command == i;
        // Aktywny slot: cyan ramka + bold cyan tekst (bez REVERSED).
        // Nieaktywny: muted. Placeholder (ready=false) dodatkowo przyciemniony.
        let (border, text) = if selected {
            (accent_primary(), accent_primary_bold())
        } else {
            // Nieaktywny slot: muted. Placeholder (ready=false) także muted —
            // brak jeszcze implementowanej akcji nie różni się wizualnie,
            // dopóki nie ma interakcji. Stylistyczna hierarchia pozostaje
            // zachowana (muted = nieaktywny, cyan = aktywny).
            (muted_style(), muted_style())
        };

        // Responsywność: na wąskich slotach (szerokość < 12) pokaż tylko ikonę.
        let label = if slots[i].width >= 12 {
            format!(" {} {} ({}) ", action.icon, action.label, action.shortcut)
        } else if slots[i].width >= 6 {
            format!(" {} {}", action.icon, action.label)
        } else {
            format!(" {} ", action.icon)
        };

        let block = Block::default().borders(Borders::ALL).border_style(border);
        let p = Paragraph::new(
            Line::from(Span::styled(label, text)).alignment(ratatui::layout::Alignment::Center),
        )
        .block(block);
        f.render_widget(p, slots[i]);
    }
}

// ---- footer (bez zmian) -----------------------------------------------------------

fn draw_footer(f: &mut Frame, area: Rect) {
    let keys = Line::from(vec![
        Span::raw(" "),
        Span::styled("q", accent_primary()),
        Span::raw(" quit  "),
        Span::styled("Tab", accent_primary()),
        Span::raw(" panel  "),
        Span::styled("r", accent_primary()),
        Span::raw(" refresh  "),
        Span::styled("←→", accent_primary()),
        Span::raw(" command  "),
        Span::styled("↑↓", accent_primary()),
        Span::raw(" nav  "),
        Span::styled("Enter", accent_primary()),
        Span::raw(" select"),
    ]);
    let p = Paragraph::new(keys);
    f.render_widget(p, area);
}
