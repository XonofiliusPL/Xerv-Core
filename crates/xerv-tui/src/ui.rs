use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

const HEADER_HEIGHT: u16 = 1;
const FOOTER_HEIGHT: u16 = 1;

/// Minimalny Top Bar — wyłącznie brand + link do GitHub. Brak pól diagnostycznych
/// (Core status / API / schema / uptime / data / state / log / boot).
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
    draw_footer(f, chunks[2]);
}

// ---- design tokens (DESIGN.md) -------------------------------------------------
//
// Jedno miejsce prawdy dla identyfikacji kolorów — zmiana tutaj = zmiana w DESIGN.md.

/// primary #00D7D7 (terminal Cyan) — akcent interaktywny / aktywny element.
fn accent_primary() -> Style {
    Style::default().fg(Color::Cyan)
}

/// primary + BOLD — brand, aktywny napis.
fn accent_primary_bold() -> Style {
    accent_primary().add_modifier(Modifier::BOLD)
}

/// secondary #D787D7 (terminal Magenta) — hover / wartości akcentu.
fn accent_secondary() -> Style {
    Style::default().fg(Color::Magenta)
}

/// value #FFFFFF (terminal White) — wartości danych.
fn value_style() -> Style {
    Style::default().fg(Color::White)
}

/// muted #585858 (terminal DarkGray) — etykiety, separatory, idle.
fn muted_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

const ITEM_MARKER_CURSOR: &str = "\u{258E} "; // ▎ : kursor (white) — wybrana opcja
const ITEM_MARKER_HOVER: &str = "\u{25CB} "; // ○ : hover (magenta, nie zmienia cursor)

/// Header — wyłącznie brand i link do GitHub.
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

/// Footer — wyłącznie hint skrótów, nic więcej.
fn draw_footer(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("← → ↑ ↓  Nav", accent_primary()),
        Span::raw("    "),
        Span::styled("Enter  Confirm", accent_primary()),
        Span::raw("    "),
        Span::styled("BackSpace  Return", accent_primary()),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

/// Renderuje treść głównego obszaru w zależności od aktualnego ekranu.
/// Dla `Screen::Main` zapisuje `app.nav_area` (obszar listy nawigacji)
/// — potrzebny do hit-testów mysą w `App::on_hover`/`on_click`.
fn draw_content(f: &mut Frame, area: Rect, app: &mut App) {
    match app.current_screen {
        crate::app::Screen::Main => draw_main(f, area, app),
        crate::app::Screen::Help => draw_help(f, area),
        crate::app::Screen::Settings => draw_settings(f, area),
    }
}

/// Główny ekran — lista nawigacji. Każda pozycja to jeden wiersz;
/// aktywna = cyan+bold (▎), hover = magenta (○).
fn draw_main(f: &mut Frame, area: Rect, app: &mut App) {
    // Obszar listy = wewnątrz ramki (border 1 z każdej strony).
    // Lista zaczyna się od wiersza 0 w inner (pierwszy element = idx 0).
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

/// Ekran ustawień — placeholder (pusty), nie implementuje funkcjonalności.
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

/// Pełny ekran Help — skróty, info o Xerv, link do GitHub.
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
