use crossterm::event::{
    self as ct_event, Event as CtEvent, KeyCode, KeyEvent, KeyEventKind, MouseEvent,
};

/// Zdarzenia akcyjne (wejścia) obsługiwane przez aplikację.
///
/// Każdy wariant mapuje się 1:1 na akcję `App::handle_event`. Zdarzenia
/// myszy (`Hover`/`Click`) tworzone są w `From<MouseEvent>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// q / Esc — zawsze kończy aplikację.
    Quit,
    /// Strzałki w dół / prawo (w Screen::Main) — przesuwa cursor.
    NavDown,
    /// Strzałki w górę / lewo (w Screen::Main) — przesuwa cursor.
    NavUp,
    /// Lef/Right aliases.
    NavLeft,
    NavRight,
    /// Enter / Space — potwierdza aktualną pozycję (OpenScreen/Activate).
    Confirm,
    /// Backspace — powrót do poprzedniego widoku (Return).
    Return,
    /// h — otwiera Help (z dowolnego widoku).
    OpenHelp,
    /// Ruch myszy na (col, row) — czysty highlight (magenda), nie zmienia cursor.
    Hover(u16, u16),
    /// Kliknięcie myszą na (col, row) — aktywacja pozycji.
    Click(u16, u16),
    Tick,
}

/// Konwersja z `crossterm::event::MouseEvent` → `Event`.
///
/// MouseMove → `Hover`, Left Click → `Click`, inne przyciski/ruchy → `Tick`
/// (ignorowany — brak right-click/context menu).
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
                    // h — Help (globalny skrót).
                    KeyCode::Char('h') => Event::OpenHelp,
                    KeyCode::Tab => Event::NavRight,
                    KeyCode::BackTab => Event::NavLeft,
                    KeyCode::Enter | KeyCode::Char(' ') => Event::Confirm,
                    KeyCode::Backspace => Event::Return,
                    // ↑/↓ oraz ←/→ — nawigacja w Screen::Main.
                    KeyCode::Down | KeyCode::Right => Event::NavDown,
                    KeyCode::Up | KeyCode::Left => Event::NavUp,
                    _ => continue,
                });
            }
            CtEvent::Mouse(m) => return Ok(m.into()),
            _ => continue,
        }
    }
}
