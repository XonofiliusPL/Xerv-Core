use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, Area, Panel, UiAreas, COMMANDS, COMMAND_COUNT};

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

fn label_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn card_border_style(active: bool, hovered: bool) -> Style {
    if active {
        Style::default().fg(Color::Cyan)
    } else if hovered {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
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
        b = b.title_style(label_style());
    }
    b
}

// ---- header (bez zmian względem poprzedniej wersji) -----------------------------

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    if area.width < MIN_USABLE_WIDTH {
        let title = Line::from(Span::styled(
            " Xerv",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
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
        Span::styled(
            "XERV",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  •  ", label_style()),
        Span::styled(
            format!("api {}.{}.{}", api.major, api.minor, api.patch),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("  •  core ", label_style()),
        if shutdown {
            Span::styled(
                "SHUTDOWN",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "READY",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        },
        Span::styled(format!("  boot #{}", st.boot_count), label_style()),
    ]);

    let separator = Line::from(Span::styled("─".repeat(area.width as usize), label_style()));

    let meta = Line::from(vec![
        Span::raw(" "),
        Span::styled("data ", label_style()),
        Span::styled(
            short_path(&cfg.data_dir, 3),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("  schema v", label_style()),
        Span::styled(
            st.schema_version.to_string(),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("  uptime ", label_style()),
        Span::styled(
            format_uptime(st.started_at_unix),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("  log ", label_style()),
        Span::styled(cfg.log_level.clone(), Style::default().fg(Color::Cyan)),
    ]);

    let lines = vec![brand, separator, meta];
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(label_style());
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

fn draw_side(f: &mut Frame, area: Rect, app: &App) {
    let items = vec![
        ListItem::new("Dashboard"),
        ListItem::new("(more screens soon)"),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" panels ")
        .border_style(if app.active_panel == Panel::Side {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

// ---- dashboard -----------------------------------------------------------------

/// Gęsty dashboard: pasek statusu + grid 3 kart. Zwraca obszary kart.
fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) -> Vec<(&'static str, Area)> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    draw_status_strip(f, chunks[0], app);
    draw_cards(f, chunks[1], app)
}

/// Jednolinijkowy pasek statusu wewnątrz dashboardu (gęsta informacja).
fn draw_status_strip(f: &mut Frame, area: Rect, app: &App) {
    let shutdown = app.core.is_shutdown();
    let st = app.core.state();
    let api = app.core.api_version();

    let status_span = if shutdown {
        Span::styled(
            "● SHUTDOWN",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            "● READY",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    };

    let line = Line::from(vec![
        Span::raw(" "),
        status_span,
        Span::styled("  api ", label_style()),
        Span::styled(
            format!("{}.{}.{}", api.major, api.minor, api.patch),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("  boot #", label_style()),
        Span::styled(
            st.boot_count.to_string(),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("  uptime ", label_style()),
        Span::styled(
            format_uptime(st.started_at_unix),
            Style::default().fg(Color::Magenta),
        ),
    ]);

    f.render_widget(Paragraph::new(line), area);
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

    // Karta 1: System.
    let sys_lines = vec![
        kv(
            "api version",
            &format!("{}.{}.{}", api.major, api.minor, api.patch),
            Color::Magenta,
        ),
        kv("data dir", &short_path(&cfg.data_dir, 2), Color::Cyan),
        kv("state file", &cfg.state_filename, Color::Cyan),
        kv("log level", &cfg.log_level, Color::Cyan),
    ];
    render_card(
        f,
        card_chunks[0],
        "system",
        sys_lines,
        false,
        false,
        &mut areas,
    );

    // Karta 2: Runtime/State.
    let run_lines = vec![
        kv("schema", &format!("v{}", st.schema_version), Color::Magenta),
        kv("boot count", &st.boot_count.to_string(), Color::Magenta),
        kv("uptime", &format_uptime(st.started_at_unix), Color::Magenta),
        kv(
            "status",
            if app.core.is_shutdown() {
                "SHUTDOWN"
            } else {
                "READY"
            },
            if app.core.is_shutdown() {
                Color::Red
            } else {
                Color::Green
            },
        ),
    ];
    render_card(
        f,
        card_chunks[1],
        "runtime",
        run_lines,
        false,
        false,
        &mut areas,
    );

    areas
}

/// Jedna para etykieta→wartość.
fn kv<'a>(label: &str, value: &str, value_color: Color) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{label:<12}"), label_style()),
        Span::styled(value.to_string(), Style::default().fg(value_color)),
    ])
}

/// Renderuje kartę i zapisuje jej obszar.
fn render_card(
    f: &mut Frame,
    area: Rect,
    title: &'static str,
    lines: Vec<Line<'static>>,
    _active: bool,
    _hovered: bool,
    areas: &mut Vec<(&'static str, Area)>,
) {
    let block = card_block(title, _active, _hovered);
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
        let style = if selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let label = format!(" {} ", name);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let p = Paragraph::new(Line::from(Span::styled(label, style))).block(block);
        f.render_widget(p, slots[i]);
    }
}

// ---- footer (bez zmian) -----------------------------------------------------------

fn draw_footer(f: &mut Frame, area: Rect) {
    let keys = Line::from(vec![
        Span::raw(" "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(" quit  "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" panel  "),
        Span::styled("r", Style::default().fg(Color::Cyan)),
        Span::raw(" refresh  "),
        Span::styled("←→", Style::default().fg(Color::Cyan)),
        Span::raw(" command  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" select"),
    ]);
    let p = Paragraph::new(keys);
    f.render_widget(p, area);
}
