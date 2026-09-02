use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, Panel};

const HEADER_HEIGHT: u16 = 3;
const FOOTER_HEIGHT: u16 = 1;
const SIDE_WIDTH_PERCENT: u16 = 25;

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

fn draw_header(f: &mut Frame, area: Rect, _app: &App) {
    let api = xerv_core::api::api_version();
    let title = Line::from(vec![
        Span::styled(
            "Xerv",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("api {}.{}.{}", api.major, api.minor, api.patch),
            Style::default().fg(Color::Magenta),
        ),
    ]);
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .title(" xerv-tui ")
        .title_style(Style::default().fg(Color::Cyan));
    let p = Paragraph::new(title).block(block);
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
        .border_style(if app.active_panel == Panel::Side {
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
        .border_style(if app.active_panel == Panel::Main {
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
