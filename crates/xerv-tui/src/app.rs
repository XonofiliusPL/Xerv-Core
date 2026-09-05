use xerv_core::api::XervCore;

use crate::event::Event;

/// Główne ekrany Xerv Core UI — model ekranowy (screen-based).
///
/// Jeden główny ekran jest aktywny w danej chwili; nawigacja przełącza
/// `current_screen`. `nav_stack` zapamiętuje historię (BackSpace = pop).
/// Brak stałego Sidebar/Main — każdy widok zajmuje cały obszar treści.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Główny ekran — lista nawigacji: Settings, Help, Quit.
    Main,
    /// Ekran pomocy (pełny ekran).
    Help,
    /// Ekran ustawień (placeholder — pełny ekran, pusty).
    Settings,
}

/// Pozycja w głównej liście nawigacji (Screen::Main).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Settings,
    Help,
    Quit,
}

/// Lista nawigacji (kolejność = kolejność strzałek).
pub const NAV_ITEMS: [NavItem; 3] = [NavItem::Settings, NavItem::Help, NavItem::Quit];
pub const NAV_COUNT: usize = NAV_ITEMS.len();

/// Etykieta + skrót dla pozycji nawigacji.
#[derive(Debug, Clone, Copy)]
pub struct NavEntry {
    pub label: &'static str,
    pub shortcut: &'static str,
}

/// Stałe etykiety (dla testów renderu).
pub const NAV_LABELS: [&str; 3] = ["Settings", "Help", "Quit"];

impl NavItem {
    pub fn label(self) -> &'static str {
        match self {
            NavItem::Settings => "Settings",
            NavItem::Help => "Help",
            NavItem::Quit => "Quit",
        }
    }
    pub fn shortcut(self) -> &'static str {
        match self {
            NavItem::Settings => "s",
            NavItem::Help => "h",
            NavItem::Quit => "q",
        }
    }
}

/// Oblicza indeks pozycji w NAV_ITEMS.
pub fn nav_index(item: NavItem) -> usize {
    match item {
        NavItem::Settings => 0,
        NavItem::Help => 1,
        NavItem::Quit => 2,
    }
}

/// Prostokąt w układzie współrzędnych terminala (kolumna/wiersz).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.x
            && col < self.x.saturating_add(self.w)
            && row >= self.y
            && row < self.y.saturating_add(self.h)
    }
}

/// Model stanu aplikacji Xerv Core UI.
#[derive(Debug)]
pub struct App {
    /// Instancja Rdzenia (do shutdown przy Quit).
    pub core: XervCore,
    /// Aktywny ekran.
    pub current_screen: Screen,
    /// Stos ekranów (do BackSpace — Return do poprzedniego widoku).
    pub nav_stack: Vec<Screen>,
    /// Indeks aktualnie podświetlonej pozycji w Screen::Main.
    pub nav_cursor: usize,
    /// Indeks hoverowanej myszą pozycji w Screen::Main (magenta, nie zmienia cursor).
    pub nav_hover: Option<usize>,
    /// Czy aplikacja powinna wyjść.
    pub should_quit: bool,
    /// Obszar listy nawigacji (dla hit-testów mysą).
    pub nav_area: Option<Rect>,
}

impl App {
    pub fn try_new(
        config: xerv_core::api::CoreConfig,
        state_path: std::path::PathBuf,
    ) -> xerv_core::api::ApiResult<Self> {
        let core = XervCore::new(config, state_path)?;
        Ok(Self {
            core,
            current_screen: Screen::Main,
            nav_stack: Vec::new(),
            nav_cursor: 0,
            nav_hover: None,
            should_quit: false,
            nav_area: None,
        })
    }

    /// Pobiera listę nawigacyjnych indeksów w aktualnym widoku.
    pub fn entries(&self) -> &[NavItem] {
        &NAV_ITEMS
    }

    /// Obsługa zdarzenia głównego loopu.
    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Quit => self.should_quit = true,
            Event::OpenHelp => {
                if self.current_screen != Screen::Help {
                    self.nav_stack.push(self.current_screen);
                    self.current_screen = Screen::Help;
                }
            }
            Event::NavDown => {
                if self.current_screen == Screen::Main {
                    self.nav_cursor = (self.nav_cursor + 1) % NAV_COUNT;
                }
            }
            Event::NavUp => {
                if self.current_screen == Screen::Main {
                    self.nav_cursor = if self.nav_cursor == 0 {
                        NAV_COUNT - 1
                    } else {
                        self.nav_cursor - 1
                    };
                }
            }
            Event::NavLeft => {
                if self.current_screen == Screen::Main {
                    self.nav_cursor = if self.nav_cursor == 0 {
                        NAV_COUNT - 1
                    } else {
                        self.nav_cursor - 1
                    };
                }
            }
            Event::NavRight => {
                if self.current_screen == Screen::Main {
                    self.nav_cursor = (self.nav_cursor + 1) % NAV_COUNT;
                }
            }
            Event::Confirm => {
                self.activate_current();
            }
            Event::Return => {
                self.go_back();
            }
            Event::Tick => {}
            Event::Hover(col, row) => self.on_hover(col, row),
            Event::Click(col, row) => self.on_click(col, row),
        }
    }

    /// Aktywacja aktualnie podświetlonej pozycji (Enter).
    fn activate_current(&mut self) {
        if self.current_screen != Screen::Main {
            return;
        }
        match self.entries()[self.nav_cursor] {
            NavItem::Settings => {
                self.nav_stack.push(self.current_screen);
                self.current_screen = Screen::Settings;
            }
            NavItem::Help => {
                self.nav_stack.push(self.current_screen);
                self.current_screen = Screen::Help;
            }
            NavItem::Quit => {
                self.should_quit = true;
            }
        }
    }

    /// Powrót do poprzedniego widoku (BackSpace). Na główce stosu nic nie robi.
    fn go_back(&mut self) {
        if let Some(prev) = self.nav_stack.pop() {
            self.current_screen = prev;
        }
    }

    /// Mouse hover — czysty nakładnik (magenta), nie zmienia cursor/active.
    pub fn on_hover(&mut self, col: u16, row: u16) {
        self.nav_hover = self.nav_area.and_then(|rect| {
            if !rect.contains(col, row) || rect.h == 0 {
                return None;
            }
            // Każda pozycja zajmuje jeden wiersz w obszarze listy.
            let idx = (row - rect.y) as usize;
            if idx < NAV_COUNT {
                Some(idx)
            } else {
                None
            }
        });
    }

    /// Mouse click — aktywuje pozycję lub zamyka aplikację.
    pub fn on_click(&mut self, col: u16, row: u16) {
        if let Some(rect) = self.nav_area {
            if rect.contains(col, row) {
                if rect.h == 0 {
                    return;
                }
                let idx = (row - rect.y) as usize;
                if idx >= NAV_COUNT {
                    return;
                }
                // Klik = aktywacja tej samej pozycji co cursor.
                self.nav_cursor = idx;
                self.activate_current();
            }
        }
    }

    /// Aktualizacja geometrii po renderze.
    pub fn on_render(&mut self, nav_area: Option<Rect>) {
        self.nav_area = nav_area;
    }
}
