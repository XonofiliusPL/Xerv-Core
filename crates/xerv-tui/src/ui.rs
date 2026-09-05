use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{
    App, Area, CorePanel, Panel, Step, UiAreas, COMMAND_ACTIONS, COMMAND_COUNT, INSTALL_STEPS,
    ONBOARDING_STEPS, SIDEBAR_COUNT, SIDEBAR_ITEMS,
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

    // Główny obszar = boczny panel (sidebar) + treść (dashboard) + pasek komend.
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(COMMAND_BAR_HEIGHT)])
        .split(chunks[1]);
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
/// wartości „odpowiedzi”). Nigdy ramki interaktywne.
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

/// Akcent dla gotowych kroków onboardingu (green ✓) vs placeholderów (muted ○).
fn step_style(done: bool) -> Style {
    if done {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        muted_style()
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

/// Ramka karty — aktywna (fokus) cyan, hover magenta, spoczynek dark gray.
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

// ---- header (bez zmian względem poprzedniej wersji) ------------------------------

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

const ITEM_MARKER_ACTIVE: &str = "\u{25CF} "; // ● : aktywny (cyan)
const ITEM_MARKER_CURSOR: &str = "\u{258E} "; // ▎ : kursor nawigacji (white) — gdy active_panel == Side
const ITEM_MARKER_HOVER: &str = "\u{25CB} "; // ○ : hover myszy (magenta, niezmienia aktywnego)

/// Pozycja sidebaru: imię + styl + marker — renderowana z jednego źródła.
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
/// Aktywna pozycja: cyan (●). Kursor (↑↓/klik): white (▎).
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

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // " NAVIGATION"
            Constraint::Length(SIDEBAR_COUNT as u16),
            Constraint::Min(1), // wolna przestrzeń
        ])
        .split(inner);

    let head = Line::from(Span::styled(" NAVIGATION", muted_style()));
    f.render_widget(Paragraph::new(head), rows[0]);

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

// ---- dashboard (Core: Install / Onboarding / Main) -----------------------------

const STEP_ICON_DONE: char = '✓';
const STEP_ICON_PENDING: char = '○';

/// Jeden wiersz kroku (done = green ✓ / muted ○ + label).
fn step_line(step: &Step, is_cursor: bool) -> Line<'static> {
    let marker = if step.done {
        Span::styled(format!(" {STEP_ICON_DONE} "), step_style(true))
    } else {
        Span::styled(format!(" {STEP_ICON_PENDING} "), step_style(false))
    };
    let label = Span::styled(
        step.label,
        if is_cursor {
            value_style().add_modifier(Modifier::BOLD)
        } else {
            if step.done {
                step_style(true)
            } else {
                muted_style()
            }
        },
    );
    Line::from(vec![marker, label])
}

/// Status strip — semantyczny status Core (READY/SHUTDOWN) + kluczowe info.
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

/// Karta progresu — lista kroków z znacznikami ✓/○, responsywnie tekst w środku.
fn draw_progress_card(
    f: &mut Frame,
    area: Rect,
    title: &'static str,
    steps: &[Step],
    cursor: Option<usize>,
    app: &App,
    areas: &mut Vec<(&'static str, Area)>,
) {
    let is_active = app.active_panel == Panel::Main;
    let is_hovered = app.dashboard_hover == Some(title);
    let block = card_block(title, is_active, is_hovered);

    let mut lines: Vec<Line<'static>> = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let is_cursor = cursor == Some(i);
        lines.push(step_line(step, is_cursor));
    }
    // Wyrównaj do wysokości (puste spacery na doł).
    let inner_h = area.height.saturating_sub(2) as usize;
    while lines.len() < inner_h {
        lines.push(Line::from(Span::raw("")));
    }

    f.render_widget(
        Paragraph::new(lines).block(block),
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        },
    );
    areas.push((title, to_area(area)));
}

/// Karta stanu Core (Main) — status, wersja, API, schema, uptime, paths.
fn draw_core_card(f: &mut Frame, area: Rect, app: &App, areas: &mut Vec<(&'static str, Area)>) {
    let is_active = app.active_panel == Panel::Main;
    let is_hovered = app.dashboard_hover == Some("core");
    let block = card_block("core", is_active, is_hovered);

    let cfg = app.core.config();
    let st = app.core.state();
    let api = app.core.api_version();
    let shutdown = app.core.is_shutdown();

    let status_text = if shutdown { "SHUTDOWN" } else { "READY" };
    let status_color = if shutdown { Color::Red } else { Color::Green };

    let lines = vec![
        kv_pair(
            "status",
            status_text,
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        kv_pair("xerv", env!("CARGO_PKG_VERSION"), value_style()),
        kv_pair(
            "api",
            &format!("v{}.{}.{}", api.major, api.minor, api.patch),
            accent_secondary(),
        ),
        kv_pair("schema", &format!("v{}", st.schema_version), value_style()),
        kv_pair("uptime", &format_uptime(st.started_at_unix), value_style()),
        kv_pair(
            "data dir",
            &short_path(&cfg.data_dir, 3),
            accent_secondary(),
        ),
        kv_pair("state file", &cfg.state_filename, value_style()),
        kv_pair("log level", &cfg.log_level, value_style()),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
    areas.push(("core", to_area(area)));
}

/// Para etykieta→wartość z jawnym stylem (etykieta muted, wartość styl docelowy).
fn kv_pair(label: &str, value: &str, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{label:<12}"), muted_style()),
        Span::styled(value.to_string(), value_style),
    ])
}

/// Statyczny panel Help — dostępny z Sidebaru (Help). Prezentuje skróty
/// klawiaturowe i opis działania TUI. Nie jest interactive — to jedynie
/// odczyt informacji.
fn draw_help(f: &mut Frame, area: Rect, app: &App, areas: &mut Vec<(&'static str, Area)>) {
    let is_hovered = app.dashboard_hover == Some("help");
    let block = card_block("help", app.active_panel == Panel::Main, is_hovered);

    // Treść Help: skróty + opis — statyczna, niezależna od Core.
    let lines: Vec<Line<'static>> = vec![
        Line::from(Span::styled(
            " Navigation",
            value_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Tab", accent_primary()),
            Span::raw("  przełącz panel (Side ↔ Main)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("↑↓", accent_primary()),
            Span::raw("  porusz kursor przez nawigację"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Enter", accent_primary()),
            Span::raw("  aktywuj / przejdź dalej"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("←→", accent_primary()),
            Span::raw("  wybierz akcję w command barze"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Home/End", accent_primary()),
            Span::raw("  przejdź na początek/koniec listy"),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            " Panels",
            value_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::raw("install"),
            Span::raw("  — przygotowanie Core (core init, config ready)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::raw("onboarding "),
            Span::raw("— konfiguracja Core (config/state ready)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::raw("main"),
            Span::raw("  — pełny stan Core po zakończeniu przygotowania"),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            " Mouse",
            value_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::raw("hover"),
            Span::raw("  — wizualny highlight (magenta), nie zmienia aktywnego"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::raw("click"),
            Span::raw("  — aktywacja elementu (sidebar / command bar / karta)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Exit", accent_primary()),
            Span::raw("  — wyjdź z Xerv (klik lub Enter)"),
        ]),
    ];

    // Scrollujemy, jeśli treść nie mieści się — tu statyczna, więc po prostu
    // renderujemy do bloku; puste linie wyrównują do wysokości.
    let inner_h = area.height.saturating_sub(2) as usize;
    let mut body = lines;
    while body.len() < inner_h {
        body.push(Line::from(Span::raw("")));
    }

    f.render_widget(
        Paragraph::new(body).block(block),
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        },
    );
    areas.push(("help", to_area(area)));
}

fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) -> Vec<(&'static str, Area)> {
    // Układ pionowy: status strip (1) + treść (Min).
    // Treść = Install (progres), Onboarding (progres), Main (core card).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status strip
            Constraint::Min(0),    // treść
        ])
        .split(area);

    draw_status_strip(f, chunks[0], app);

    let mut areas = Vec::new();
    let content = chunks[1];
    if app.show_help {
        draw_help(f, content, app, &mut areas);
    } else {
        match app.core_panel {
            CorePanel::Install => {
                draw_progress_card(f, content, "install", &INSTALL_STEPS, None, app, &mut areas);
            }
            CorePanel::Onboarding => {
                draw_progress_card(
                    f,
                    content,
                    "onboarding",
                    &ONBOARDING_STEPS,
                    None,
                    app,
                    &mut areas,
                );
            }
            CorePanel::Main => {
                draw_core_card(f, content, app, &mut areas);
            }
        }
    }
    areas
}

// ---- command bar ----------------------------------------------------------------

/// Command Bar / Action Bar — komplementuje Sidebar (główna nawigacja).
/// Prezentuje dostępne akcje z ikoną + skrótem. Każdy slot jest klikalny
/// myszą i obsługiwany klawiaturą (←→/Enter). selected (keyboard) = cyan >
/// hovered (mouse) = magenta > spoczynek (muted).
fn draw_command_bar(f: &mut Frame, area: Rect, app: &App) {
    let slots = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Ratio(1, COMMAND_COUNT as u32);
            COMMAND_COUNT
        ])
        .split(area);

    for (i, action) in COMMAND_ACTIONS.iter().enumerate() {
        let selected = app.selected_command == i;
        let hovered = app.hovered_command == Some(i);
        // Hierarchia: selected (cyan) > hovered (magenta) > spoczynek (muted).
        // Hover nie zmienia `selected` — czysty nakładnik wizualny.
        let (border, text) = if selected {
            (accent_primary(), accent_primary_bold())
        } else if hovered {
            (accent_secondary(), accent_secondary())
        } else {
            (muted_style(), muted_style())
        };

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
        Span::styled("↑↓", accent_primary()),
        Span::raw(" nav  "),
        Span::styled("Enter", accent_primary()),
        Span::raw(" select  "),
        Span::styled("←→", accent_primary()),
        Span::raw(" command"),
    ]);
    let p = Paragraph::new(keys);
    f.render_widget(p, area);
}
