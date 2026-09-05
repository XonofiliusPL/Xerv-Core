use xerv_core::api::XervCore;

use crate::event::Event;

/// Główne ekrany Xerv Core UI — model ekranowy (screen-based).
///
/// Jeden główny ekran jest aktywny w danej chwili; nawigacja przełącza
/// `current_screen`. `nav_stack` zapamiętuje historię (BackSpace = pop).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Główny ekran — lista nawigacji: Settings, Help, Update, Quit.
    Main,
    /// Ekran pomocy (pełny ekran).
    Help,
    /// Ekran ustawień (placeholder — pełny ekran, pusty).
    Settings,
    /// Ekran potwierdzenia aktualizacji (pełny ekran).
    UpdateConfirm,
}

/// Pozycja w głównej liście nawigacji (Screen::Main).
/// `Update` jest dynamiczny — tylko gdy dostępna aktualizacja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Settings,
    Help,
    /// Update Xerv — widoczny tylko gdy `App::update_available == Some`.
    Update,
    Quit,
}

/// Etykieta + skrót dla pozycji nawigacji.
#[derive(Debug, Clone, Copy)]
pub struct NavEntry {
    pub label: &'static str,
    pub shortcut: &'static str,
}

impl NavItem {
    pub fn label(self) -> &'static str {
        match self {
            NavItem::Settings => "Settings",
            NavItem::Help => "Help",
            NavItem::Update => "Update Xerv",
            NavItem::Quit => "Quit",
        }
    }
    pub fn shortcut(self) -> &'static str {
        match self {
            NavItem::Settings => "s",
            NavItem::Help => "h",
            NavItem::Update => "U",
            NavItem::Quit => "q",
        }
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
    /// Najnowsza dostępna wersja jako string (Some = update dostępny).
    pub update_available: Option<String>,
    /// Czy aktualizacja w toku.
    pub update_in_progress: bool,
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
            update_available: None,
            update_in_progress: false,
        })
    }

    /// Dynamiczna lista nawigacji (Settings, Help, Update, Quit).
    /// `Update` jest widoczny tylko gdy `update_available == Some`.
    pub fn entries(&self) -> Vec<NavItem> {
        let mut items = vec![NavItem::Settings, NavItem::Help];
        if self.update_available.is_some() {
            items.push(NavItem::Update);
        }
        items.push(NavItem::Quit);
        items
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
            Event::OpenUpdate => {
                if self.current_screen == Screen::Main && self.update_available.is_some() {
                    self.nav_stack.push(self.current_screen);
                    self.current_screen = Screen::UpdateConfirm;
                }
            }
            Event::NavDown => self.move_cursor(1),
            Event::NavUp => self.move_cursor(-1i32),
            Event::NavLeft => self.move_cursor(-1i32),
            Event::NavRight => self.move_cursor(1),
            Event::Confirm => self.activate_current(),
            Event::Return => self.go_back(),
            // Y — potwierdź update na UpdateConfirm screen.
            Event::ConfirmUpdate => {
                if self.current_screen == Screen::UpdateConfirm && self.update_available.is_some() {
                    self.update_in_progress = true;
                }
            }
            // N — anuluj update na UpdateConfirm screen.
            Event::CancelUpdate => {
                if self.current_screen == Screen::UpdateConfirm {
                    self.go_back();
                }
            }
            Event::Tick => {}
            Event::Hover(col, row) => self.on_hover(col, row),
            Event::Click(col, row) => self.on_click(col, row),
        }
    }

    /// Przesunięcie kursora o `delta` (z wrap, tylko na Main).
    fn move_cursor(&mut self, delta: i32) {
        if self.current_screen != Screen::Main {
            return;
        }
        let count = self.entries().len();
        let signed = self.nav_cursor as i32 + delta;
        self.nav_cursor = ((signed % count as i32).rem_euclid(count as i32)) as usize;
    }

    /// Aktywacja aktualnie podświetlonej pozycji (Enter).
    fn activate_current(&mut self) {
        if self.current_screen != Screen::Main {
            return;
        }
        let entries = self.entries();
        let idx = self.nav_cursor % entries.len();
        match entries[idx] {
            NavItem::Settings => {
                self.nav_stack.push(self.current_screen);
                self.current_screen = Screen::Settings;
            }
            NavItem::Help => {
                self.nav_stack.push(self.current_screen);
                self.current_screen = Screen::Help;
            }
            NavItem::Update => {
                self.nav_stack.push(self.current_screen);
                self.current_screen = Screen::UpdateConfirm;
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
            let count = self.entries().len();
            let idx = (row - rect.y) as usize;
            if idx < count {
                Some(idx)
            } else {
                None
            }
        });
    }

    /// Mouse click — aktywuje pozycję.
    pub fn on_click(&mut self, col: u16, row: u16) {
        if let Some(rect) = self.nav_area {
            if rect.contains(col, row) && rect.h > 0 {
                let count = self.entries().len();
                let idx = (row - rect.y) as usize;
                if idx < count {
                    self.nav_cursor = idx;
                    self.activate_current();
                }
            }
        }
    }

    /// Aktualizacja geometrii po renderze.
    pub fn on_render(&mut self, nav_area: Option<Rect>) {
        self.nav_area = nav_area;
    }
}
