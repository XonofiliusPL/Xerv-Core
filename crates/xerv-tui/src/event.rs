use crossterm::event::{
    self as ct_event, Event as CtEvent, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent,
    MouseEventKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Quit,
    NextPanel,
    PrevPanel,
    Refresh,
    NextCommand,
    PrevCommand,
    SelectCommand,
    /// Kursor nawigacji sidebaru w dół (↑/↓ gdy fokus na Side).
    NavDown,
    /// Kursor nawigacji sidebaru w górę.
    NavUp,
    /// Potwierdzenie pozycji nawigacji (Enter/Space gdy fokus na Side).
    NavActivate,
    /// Pierwsza pozycja (Home).
    NavHome,
    /// Ostatnia pozycja (End).
    NavEnd,
    /// Kliknięcie myszą na współrzędnej (col, row) — Panel.
    ClickPanel(u16, u16),
    /// Kliknięcie myszą na współrzędnej (col, row) — Command bar.
    ClickCommand(u16, u16),
    /// Ruch myszy na (col, row) — aktualizuje highlight.
    Hover(u16, u16),
    Tick,
}

pub fn read_event() -> std::io::Result<Event> {
    loop {
        let ev = ct_event::read()?;
        match ev {
            CtEvent::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) => {
                return Ok(match code {
                    KeyCode::Char('q') | KeyCode::Esc => Event::Quit,
                    KeyCode::Char('?') => Event::Quit,
                    KeyCode::Tab => Event::NextPanel,
                    KeyCode::BackTab => Event::PrevPanel,
                    KeyCode::Char('r') => Event::Refresh,
                    KeyCode::Right => Event::NextCommand,
                    KeyCode::Left => Event::PrevCommand,
                    KeyCode::Enter => Event::NavActivate,
                    KeyCode::Char(' ') => Event::NavActivate,
                    // ↑/↓: nawigacja sidebaru; Enter/Space potwierdza.
                    KeyCode::Down => Event::NavDown,
                    KeyCode::Up => Event::NavUp,
                    KeyCode::Home => Event::NavHome,
                    KeyCode::End => Event::NavEnd,
                    _ => continue,
                });
            }
            CtEvent::Mouse(m) => return Ok(mouse_to_event(m)),
            _ => continue,
        }
    }
}

fn mouse_to_event(m: MouseEvent) -> Event {
    let (col, row) = (m.column, m.row);
    match m.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Heurystyka: kto żyje wyżej? Decyzja w main.rs (App::on_mouse)
            // wie gdzie są panele; tu przekazujemy surowe współrzędne.
            // Rozróżnienie panel-vs-command robi wywołujący.
            Event::ClickPanel(col, row)
        }
        MouseEventKind::Moved => Event::Hover(col, row),
        _ => Event::Tick,
    }
}
