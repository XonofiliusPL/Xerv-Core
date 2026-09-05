use crossterm::event::{
    self as ct_event, Event as CtEvent, KeyCode, KeyEvent, KeyEventKind, MouseEvent,
};

/// Zdarzenia akcyjne (wejścia) obsługiwane przez aplikację.
///
/// Każdy wariant mapuje się 1:1 na akcję `App::handle_event`. Zdarzenia myszy
/// (`Hover`/`Click`) są tworzone w `From<MouseEvent>` i przekazywane bezpośrednio
/// — hit-test odbywa się w `App::handle_event`.
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
    /// Ruch myszy na (col, row) — aktualizuje highlight (czysty nakładnik).
    Hover(u16, u16),
    /// Kliknięcie myszą na (col, row) — hit-test na bieżąco w `App::handle_event`.
    Click(u16, u16),
    Tick,
}

/// Konwersja z `crossterm::event::MouseEvent` → `Event`.
///
/// MouseMove → `Hover`, Left Click → `Click`, inne przyciski/ruchy → `Tick`
/// (ignorowany — nie ma right-click/context menu).
impl From<MouseEvent> for Event {
    fn from(m: MouseEvent) -> Self {
        let (col, row) = (m.column, m.row);
        match m.kind {
            crossterm::event::MouseEventKind::Moved => Event::Hover(col, row),
            crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                Event::Click(col, row)
            }
            _ => Event::Tick,
        }
    }
}

/// Czyta następny event wejściowy, ignoruje `KeyEventKind::Release`.
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
            CtEvent::Mouse(m) => return Ok(m.into()),
            _ => continue,
        }
    }
}
