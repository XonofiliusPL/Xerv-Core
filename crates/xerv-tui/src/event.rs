use crossterm::event::{
    self as ct_event, Event as CtEvent, KeyCode, KeyEvent, KeyEventKind, MouseEvent,
};

/// Action events (inputs) handled by the application.
///
/// Each variant maps 1:1 to an `App::handle_event` action. Mouse events
/// (`Hover`/`Click`) are created in `From<MouseEvent>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// q / Esc — always quits the application.
    Quit,
    /// Down / Right arrows (on Screen::Main) — move cursor.
    NavDown,
    /// Up / Left arrows (on Screen::Main) — move cursor.
    NavUp,
    /// Left/Right aliases.
    NavLeft,
    NavRight,
    /// Enter / Space — confirm current position (OpenScreen/Activate).
    Confirm,
    /// Y — confirm update on UpdateConfirm screen.
    ConfirmUpdate,
    /// N — cancel update on UpdateConfirm screen.
    CancelUpdate,
    /// Backspace — return to previous view (Return).
    Return,
    /// h — open Help (from any view).
    OpenHelp,
    /// u — open Update screen (when available, from Main).
    OpenUpdate,
    /// Mouse movement at (col, row) — pure highlight (magenta),
    /// does not change cursor.
    Hover(u16, u16),
    /// Mouse click at (col, row) — activate position.
    Click(u16, u16),
    Tick,
}

/// Conversion from `crossterm::event::MouseEvent` → `Event`.
///
/// MouseMove → `Hover`, Left Click → `Click`, other buttons/moves → `Tick`
/// (ignored — no right-click/context menu).
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

/// Read the next input event, ignoring `KeyEventKind::Release`.
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
                    // h — Help (global shortcut).
                    KeyCode::Char('h') => Event::OpenHelp,
                    // U — Update Xerv (when available, from Main).
                    KeyCode::Char('U') => Event::OpenUpdate,
                    KeyCode::Tab => Event::NavRight,
                    KeyCode::BackTab => Event::NavLeft,
                    KeyCode::Enter | KeyCode::Char(' ') => Event::Confirm,
                    // y/n on UpdateConfirm screen.
                    KeyCode::Char('y') | KeyCode::Char('Y') => Event::ConfirmUpdate,
                    KeyCode::Char('n') | KeyCode::Char('N') => Event::CancelUpdate,
                    KeyCode::Backspace => Event::Return,
                    // ↑/↓ as well as ←/→ — navigation on Screen::Main.
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
