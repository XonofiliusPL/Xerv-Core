use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

const HEADER_HEIGHT: u16 = 1;
const FOOTER_HEIGHT: u16 = 1;

/// Minimal Top Bar — exclusively brand + GitHub link. No diagnostic
/// fields (Core status / API / schema / uptime / data / state / log / boot).
pub fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(1),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(area);

    draw_header(f, chunks[0]);
    draw_content(f, chunks[1], app);
    draw_footer(f, chunks[2], app);
}

// ---- design tokens (DESIGN.md) -------------------------------------------------
//
// Single source of truth for color identification — changing here = changing
// in DESIGN.md.

/// primary #00D7D7 (terminal Cyan) — interactive accent / active element.
fn accent_primary() -> Style {
    Style::default().fg(Color::Cyan)
}

/// primary + BOLD — brand, active text.
fn accent_primary_bold() -> Style {
    accent_primary().add_modifier(Modifier::BOLD)
}

/// secondary #D787D7 (terminal Magenta) — hover / accent values.
fn accent_secondary() -> Style {
    Style::default().fg(Color::Magenta)
}

/// value #FFFFFF (terminal White) — data values.
fn value_style() -> Style {
    Style::default().fg(Color::White)
}

/// muted #585858 (terminal DarkGray) — labels, separators, idle.
fn muted_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

const ITEM_MARKER_CURSOR: &str = "\u{258E} "; // ▎ : cursor (white) — selected option
const ITEM_MARKER_HOVER: &str = "\u{25CB} "; // ○ : hover (magenta, does not change cursor)

/// Header — exclusively brand and GitHub link.
fn draw_header(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("XERV", accent_primary_bold()),
        Span::styled("  ", muted_style()),
        Span::styled(
            "https://github.com/XonofiliusPL/Xerv-Core",
            accent_secondary(),
        ),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

/// Footer — exclusively shortcut hints, nothing else.
/// Keys = accent_primary (cyan), descriptions (Nav/Confirm/Return) = value
/// (white) — spacing and layout preserved.
/// `U Update` appears only when update is available.
fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let mut spans: Vec<Span<'static>> = vec![
        Span::raw(" "),
        Span::styled("← → ↑ ↓", accent_primary()),
        Span::styled("  Nav", value_style()),
        Span::raw("    "),
        Span::styled("Enter", accent_primary()),
        Span::styled("  Confirm", value_style()),
        Span::raw("    "),
        Span::styled("BackSpace", accent_primary()),
        Span::styled("  Return", value_style()),
    ];
    if app.update_available.is_some() {
        spans.push(Span::raw("    "));
        spans.push(Span::styled("U", accent_primary()));
        spans.push(Span::styled("  Update", value_style()));
    }
    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}

/// Render the main area content based on the current screen.
/// For `Screen::Main` saves `app.nav_area` (navigation list area) —
/// needed for mouse hit-tests in `App::on_hover`/`on_click`.
fn draw_content(f: &mut Frame, area: Rect, app: &mut App) {
    match app.current_screen {
        crate::app::Screen::Main => draw_main(f, area, app),
        crate::app::Screen::Help => draw_help(f, area),
        crate::app::Screen::Settings => draw_settings(f, area),
        crate::app::Screen::UpdateConfirm => draw_update_confirm(f, area, app),
    }
}

/// Main screen — navigation list. Each item is one row;
/// active = cyan+bold (▎), hover = magenta (○).
fn draw_main(f: &mut Frame, area: Rect, app: &mut App) {
    // List area = inside frame (border 1 on each side).
    // List starts at row 0 in inner (first item = idx 0).
    app.nav_area = Some(crate::app::Rect {
        x: area.x + 1,
        y: area.y + 1,
        w: area.width.saturating_sub(2),
        h: area.height.saturating_sub(2),
    });

    let items = app.entries();
    let lines: Vec<Line<'static>> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let label = item.label();
            let is_cursor = i == app.nav_cursor;
            let is_hover = app.nav_hover == Some(i);
            let (marker, style) = if is_cursor {
                (
                    ITEM_MARKER_CURSOR,
                    value_style().add_modifier(Modifier::BOLD),
                )
            } else if is_hover {
                (ITEM_MARKER_HOVER, accent_secondary())
            } else {
                ("  ", muted_style())
            };
            Line::from(vec![
                Span::raw(marker).style(style),
                Span::styled(label.to_string(), style),
            ])
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" xerv ")
        .border_style(accent_primary());
    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// Settings screen — placeholder (empty), does not implement functionality.
fn draw_settings(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" settings ")
        .border_style(muted_style());
    let line = Line::from(Span::styled("(not yet implemented)", muted_style()));
    f.render_widget(
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

/// Full-screen Help screen — shortcuts, info about Xerv, GitHub link.
fn draw_help(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" help ")
        .border_style(accent_secondary());

    let inner_h = area.height.saturating_sub(2) as usize;
    let mut body: Vec<Line<'static>> = vec![
        Line::from(Span::styled(
            " Keyboard",
            value_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("q", accent_primary()),
            Span::raw("   Quit"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("h", accent_primary()),
            Span::raw("   Open Help"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("← → ↑ ↓", accent_primary()),
            Span::raw("  Navigation"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Enter", accent_primary()),
            Span::raw("  Confirm"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("BackSpace", accent_primary()),
            Span::raw("  Return"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("U", accent_primary()),
            Span::raw("   Open Update (when available)"),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            " About",
            value_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::raw("Xerv Core — minimal TUI foundation."),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Source", accent_secondary()),
            Span::raw(": "),
            Span::raw("https://github.com/XonofiliusPL/Xerv-Core"),
        ]),
    ];

    while body.len() < inner_h {
        body.push(Line::from(Span::raw("")));
    }

    f.render_widget(
        Paragraph::new(body)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

/// Update confirmation screen — full screen.
/// Shows: current version, latest version, Y/N prompt.
fn draw_update_confirm(f: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" update xerv ")
        .border_style(accent_primary_bold());

    let current = xerv_core::api::api_version().to_string();
    let latest = app
        .update_available
        .clone()
        .unwrap_or_else(|| "?".to_string());

    let body: Vec<Line<'static>> = vec![
        Line::from(Span::styled(" New update available", value_style())),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::raw("  Current: "),
            Span::styled(current.clone(), accent_secondary()),
        ]),
        Line::from(vec![
            Span::raw("  Latest:  "),
            Span::styled(latest.clone(), accent_primary_bold()),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            " Press Y to confirm, N or BackSpace to cancel.",
            muted_style(),
        )),
    ];

    f.render_widget(
        Paragraph::new(body)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}
