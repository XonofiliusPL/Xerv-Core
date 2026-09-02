use xerv_core::api::{CoreConfig, XervCore};

use crate::event::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Side,
    Main,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Side => Panel::Main,
            Panel::Main => Panel::Side,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Panel::Side => Panel::Main,
            Panel::Main => Panel::Side,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub core: XervCore,
    pub active_panel: Panel,
    pub should_quit: bool,
}

impl App {
    /// Tworzy `App` z natychmiastową inicjalizacją `XervCore` (decyzja 3.7b).
    /// Zwraca błąd inicjalizacji Rdzenia; `main.rs` propaguje i wychodzi z kodem 1.
    pub fn try_new(config: CoreConfig, state_path: std::path::PathBuf) -> xerv_core::api::ApiResult<Self> {
        let core = XervCore::new(config, state_path)?;
        Ok(Self {
            core,
            active_panel: Panel::Side,
            should_quit: false,
        })
    }

    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Quit => self.should_quit = true,
            Event::NextPanel => self.active_panel = self.active_panel.next(),
            Event::PrevPanel => self.active_panel = self.active_panel.prev(),
            Event::Refresh | Event::Tick => {
                // Pierwsza wersja: no-op.
            }
        }
    }
}
