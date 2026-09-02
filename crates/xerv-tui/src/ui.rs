use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Area, Panel, UiAreas, ACTIVE_NAV, COMMANDS, COMMAND_COUNT, NAV_ITEMS};

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

/// Człowiekowo-czytelna data z unixa (lokalna, bez zewnętrznych crate'ów).
fn unix_to_human(unix: u64) -> String {
    // Bez chrono — liczymy dzień/godzinę względem epoki w formacie skróconym:
    // dni od epoki + HH:MM. To nie jest pełna data kalendarzowa, ale czytelne
    // "D+NNNN HH:MM" (dni od startu systemu uniksowego) — wystarcza do debug.
    let days = unix / 86_400;
    let rem = unix % 86_400;
    let hh = rem / 3600;
    let mm = (rem % 3600) / 60;
    format!("D+{days} {hh:02}:{mm:02}")
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

/// Sidebar — nawigacja Xerv. Struktura wg NAV_ITEMS; aktywny ekran (Dashboard)
/// wyróżniony cyan + wskaźnik ▎; kursor nawigacji (↑/↓) podświetla wiersz.
/// Sekcje przyszłe: muted (nieaktywne do momentu implementacji).
fn draw_side(f: &mut Frame, area: Rect, app: &App) {
    // Nagłówek sekcji + pozycje: 1 wiersz nagłówka grupy między sekcjami.
    // Layout: [ header "NAVIGATION" ] [ items... ] [ spacer ] [ footer-hint ]
    // Dzielimy WNĘTRZE ramki (bez borderów) — nagłówek nie nadpisze ramki.
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                      // nagłówek sekcji
            Constraint::Length(NAV_ITEMS.len() as u16), // pozycje
            Constraint::Min(1),                         // wolna przestrzeń
        ])
        .split(inner);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" xerv ")
        .border_style(if app.active_panel == Panel::Side {
            accent_primary()
        } else {
            muted_style()
        });
    f.render_widget(block, area);

    // Nagłówek sekcji — muted small-caps styl (małe litery, spacje).
    let head = Line::from(Span::styled(" NAVIGATION", muted_style()));
    f.render_widget(Paragraph::new(head), rows[0]);

    // Pozycje nawigacji.
    for (i, (name, implemented)) in NAV_ITEMS.iter().enumerate() {
        let is_cursor = app.nav_cursor == i && app.active_panel == Panel::Side;
        let is_active = i == ACTIVE_NAV;
        let y = rows[1].y + i as u16;

        // Wiersz: wskaźnik + nazwa. Active: cyan bold; cursor: reversed-lite
        // (wskaźnik ▎ w cyan); przyszłe: muted.
        let (marker, name_style): (Span, Span) = if is_active {
            (
                Span::styled("▎ ", accent_primary()),
                Span::styled((*name).to_string(), accent_primary_bold()),
            )
        } else if is_cursor {
            (
                Span::styled("▎ ", muted_style()),
                Span::styled((*name).to_string(), value_style()),
            )
        } else if *implemented {
            (
                Span::raw("  "),
                Span::styled((*name).to_string(), value_style()),
            )
        } else {
            (
                Span::raw("  "),
                Span::styled((*name).to_string(), muted_style()),
            )
        };

        let line = Line::from(vec![marker, name_style]);
        // Tło wiersza kursora — subtelne (bez pełnej inwersji): rysujemy
        // Paragraph z wierszem; ratatui nie ma row-bg, więc dla kursora
        // używamy tylko wskaźnika — czytelne i zgodne z minimalizmem.
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

/// Gęsty dashboard: pasek statusu + karty. Zwraca obszary kart.
fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) -> Vec<(&'static str, Area)> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    draw_status_strip(f, chunks[0], app);
    draw_cards(f, chunks[1], app)
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

/// Grid kart: lewa kolumna (system/storage), prawa (runtime + state).
fn draw_cards(f: &mut Frame, area: Rect, app: &App) -> Vec<(&'static str, Area)> {
    let mut areas = Vec::new();

    // Dwie kolumny; na wąskim terminalu jedna (vertical stack).
    let two_cols = area.width >= 80;
    let card_chunks = if two_cols {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    };

    let cfg = app.core.config();
    let st = app.core.state();
    let api = app.core.api_version();
    let shutdown = app.core.is_shutdown();

    // Karta 1: System — grupa "identity" + separator + grupa "storage/log".
    // api version i schema = "odpowiedzi" → secondary accent; reszta neutralna.
    let sys_lines = vec![
        kv_accent(
            "api version",
            &format!("{}.{}.{}", api.major, api.minor, api.patch),
        ),
        kv_accent("schema", &format!("v{}", st.schema_version)),
        separator_line(),
        kv("data dir", &short_path(&cfg.data_dir, 2)),
        kv("state file", &cfg.state_filename),
        kv("log level", &cfg.log_level),
    ];
    render_card(f, card_chunks[0], "system", sys_lines, &mut areas);

    // Karta 2: Runtime — lifecycle; szczegóły, których nie ma w stripie.
    let run_lines = vec![
        kv("boot count", &st.boot_count.to_string()),
        kv("first boot", &unix_to_human(st.started_at_unix)),
        separator_line(),
        status_kv(shutdown),
    ];
    render_card(f, card_chunks[1], "runtime", run_lines, &mut areas);

    areas
}

/// Cienki separator wewnątrz karty (dim).
fn separator_line() -> Line<'static> {
    Line::from(Span::styled("  ························", muted_style()))
}

/// Wiersz statusu z semantycznym kolorem (green/red).
fn status_kv(shutdown: bool) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{:<12}", "status"), muted_style()),
        if shutdown {
            Span::styled(
                "● SHUTDOWN",
                status_style(false).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("● READY", status_style(true).add_modifier(Modifier::BOLD))
        },
    ])
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

/// Renderuje kartę i zapisuje jej obszar.
fn render_card(
    f: &mut Frame,
    area: Rect,
    title: &'static str,
    lines: Vec<Line<'static>>,
    areas: &mut Vec<(&'static str, Area)>,
) {
    let block = card_block(title, false, false);
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
    areas.push((title, to_area(area)));
}

// ---- command bar ----------------------------------------------------------------

/// Poziomy pasek komend — sloty na przyszłe moduły (modules/agents/registry/...).
/// Interaktywny: ←/→ zmienia zaznaczenie, Enter wybiera; mysz klika slot.
fn draw_command_bar(f: &mut Frame, area: Rect, app: &App) {
    // Ratio zamiast Percentage — suma slotów zawsze wypełnia dokładnie `area.width`
    // (Percentage z zaokrągleń gubi 1-2 kolumny i ostatni slot wychodzi poza krawędź).
    let slots = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Ratio(1, COMMAND_COUNT as u32);
            COMMAND_COUNT
        ])
        .split(area);

    for (i, name) in COMMANDS.iter().enumerate() {
        let selected = app.selected_command == i;
        // Aktywny slot: cyan ramka + bold cyan tekst (bez REVERSED —
        // czytelniejszy i mniej agresywny niż pełna inwersja).
        let (border, text) = if selected {
            (accent_primary(), accent_primary_bold())
        } else {
            (muted_style(), muted_style())
        };
        let label = format!(" {} ", name);
        let block = Block::default().borders(Borders::ALL).border_style(border);
        let p = Paragraph::new(Line::from(Span::styled(label, text))).block(block);
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
