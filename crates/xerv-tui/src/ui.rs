use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::App;

const HEADER_HEIGHT: u16 = 4;
const FOOTER_HEIGHT: u16 = 1;
const SIDE_WIDTH_PERCENT: u16 = 25;
const MIN_USABLE_WIDTH: u16 = 30;

pub fn ui(f: &mut Frame, app: &App) {
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
    draw_main(f, chunks[1], app);
    draw_footer(f, chunks[2]);
}

// ---- helpers ----------------------------------------------------------------

/// Skraca ścieżkę do ostatnich N segmentów, żeby header się nie rozrastał.
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

/// Człowiekowo-czytelny uptime z `started_at_unix` (sekundy).
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

// ---- header -----------------------------------------------------------------

/// Rysuje header w trzech liniach:
///   1) XERV • api 0.1.0                        CORE READY  boot #1
///   2) ─────────────────────────────────────────────────────────────
///   3) data ~/.local/share/xerv  schema v1  uptime 2m
fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    // Za mała szerokość — header uproszczony (tylko nazwa), żeby nie overflowować.
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

    // Linia 1: brand po lewej, status po prawej.
    let brand = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "XERV",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  •  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("api {}.{}.{}", api.major, api.minor, api.patch),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("  •  core ", Style::default().fg(Color::DarkGray)),
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
        Span::styled(
            format!("  boot #{}", st.boot_count),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    // Linia 2: separator.
    let separator = Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::DarkGray),
    ));

    // Linia 3: meta — data_dir (skrócone), schema, uptime.
    let meta = Line::from(vec![
        Span::raw(" "),
        Span::styled("data ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            short_path(&cfg.data_dir, 3),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("  schema v", Style::default().fg(Color::DarkGray)),
        Span::styled(
            st.schema_version.to_string(),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("  uptime ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format_uptime(st.started_at_unix),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("  log ", Style::default().fg(Color::DarkGray)),
        Span::styled(cfg.log_level.clone(), Style::default().fg(Color::Cyan)),
    ]);

    let lines = vec![brand, separator, meta];
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn draw_main(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(SIDE_WIDTH_PERCENT),
            Constraint::Min(1),
        ])
        .split(area);

    draw_side(f, chunks[0], app);
    draw_dashboard(f, chunks[1], app);
}

fn draw_side(f: &mut Frame, area: Rect, app: &App) {
    let items = vec![
        ListItem::new("Dashboard"),
        ListItem::new("(more screens soon)"),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" panels ")
        .border_style(if app.active_panel == crate::app::Panel::Side {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) {
    let cfg = app.core.config();
    let st = app.core.state();
    let api = app.core.api_version();
    let body = vec![
        Line::from(Span::styled(
            "Core initialized",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(format!(
            "  api version  : {}.{}.{}",
            api.major, api.minor, api.patch
        )),
        Line::from(format!("  data_dir     : {}", cfg.data_dir.display())),
        Line::from(format!("  state_file   : {}", cfg.state_filename)),
        Line::from(format!("  schema v     : {}", st.schema_version)),
        Line::from(format!("  boots        : {}", st.boot_count)),
        Line::from(""),
        Line::from(Span::styled(
            "TUI działa. Naciśnij ? aby zobaczyć pomoc (TODO).",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" dashboard ")
        .border_style(if app.active_panel == crate::app::Panel::Main {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default()
        });
    let p = Paragraph::new(body).block(block);
    f.render_widget(p, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let keys = Line::from(vec![
        Span::raw(" "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(" quit  "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" panel  "),
        Span::styled("r", Style::default().fg(Color::Cyan)),
        Span::raw(" refresh"),
    ]);
    let p = Paragraph::new(keys);
    f.render_widget(p, area);
}
