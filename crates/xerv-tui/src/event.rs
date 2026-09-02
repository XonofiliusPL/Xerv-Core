use crossterm::event::{self as ct_event, Event as CtEvent, KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Quit,
    NextPanel,
    PrevPanel,
    Refresh,
    Tick,
}

pub fn read_event() -> std::io::Result<Event> {
    loop {
        let ev = ct_event::read()?;
        if let CtEvent::Key(KeyEvent { code, kind, .. }) = ev {
            if kind != KeyEventKind::Press {
                continue;
            }
            return Ok(match code {
                KeyCode::Char('q') | KeyCode::Esc => Event::Quit,
                KeyCode::Char('?') => Event::Quit, // tymczasowo — Etap 4 doda ekran help
                KeyCode::Tab => Event::NextPanel,
                KeyCode::BackTab => Event::PrevPanel,
                KeyCode::Char('r') => Event::Refresh,
                _ => continue,
            });
        }
    }
}
