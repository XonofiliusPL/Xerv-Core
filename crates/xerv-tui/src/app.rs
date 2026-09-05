use xerv_core::api::XervCore;

use crate::event::Event;

/// Main Xerv Core UI screens — screen-based model.
///
/// One main screen is active at any moment; navigation switches
/// `current_screen`. `nav_stack` remembers history (BackSpace = pop).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Main screen — navigation list: Settings, Help, Update, Quit.
    Main,
    /// Help screen (full screen).
    Help,
    /// Settings screen (placeholder — full screen, empty).
    Settings,
    /// Update confirmation screen (full screen).
    UpdateConfirm,
}

/// Position in the main navigation list (Screen::Main).
/// `Update` is dynamic — only when an update is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Settings,
    Help,
    /// Update Xerv — visible only when `App::update_available == Some`.
    Update,
    Quit,
}

/// Label + shortcut for a navigation entry.
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

/// Rectangle in terminal coordinates (column/row).
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

/// Application state model for Xerv Core UI.
#[derive(Debug)]
pub struct App {
    /// Core instance (for shutdown on Quit).
    pub core: XervCore,
    /// Active screen.
    pub current_screen: Screen,
    /// Screen stack (BackSpace = Return to previous view).
    pub nav_stack: Vec<Screen>,
    /// Index of currently highlighted position in Screen::Main.
    pub nav_cursor: usize,
    /// Index of mouse-hover position in Screen::Main (magenta, does not
    /// change cursor).
    pub nav_hover: Option<usize>,
    /// Whether the application should quit.
    pub should_quit: bool,
    /// Navigation list area (for mouse hit-tests).
    pub nav_area: Option<Rect>,
    /// Latest available version as string (Some = update available).
    pub update_available: Option<String>,
    /// Whether an update is in progress.
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

    /// Dynamic navigation list (Settings, Help, Update, Quit).
    /// `Update` is visible only when `update_available == Some`.
    pub fn entries(&self) -> Vec<NavItem> {
        let mut items = vec![NavItem::Settings, NavItem::Help];
        if self.update_available.is_some() {
            items.push(NavItem::Update);
        }
        items.push(NavItem::Quit);
        items
    }

    /// Handle an application event.
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
            // Y — confirm update on UpdateConfirm screen.
            Event::ConfirmUpdate => {
                if self.current_screen == Screen::UpdateConfirm && self.update_available.is_some() {
                    self.update_in_progress = true;
                }
            }
            // N — cancel update on UpdateConfirm screen.
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

    /// Move cursor by `delta` (with wrap, on Main only).
    fn move_cursor(&mut self, delta: i32) {
        if self.current_screen != Screen::Main {
            return;
        }
        let count = self.entries().len();
        let signed = self.nav_cursor as i32 + delta;
        self.nav_cursor = ((signed % count as i32).rem_euclid(count as i32)) as usize;
    }

    /// Activate the currently highlighted item (Enter).
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

    /// Return to previous view (BackSpace). No-op on empty stack.
    fn go_back(&mut self) {
        if let Some(prev) = self.nav_stack.pop() {
            self.current_screen = prev;
        }
    }

    /// Mouse hover — pure overlay (magenta), does not change cursor/active.
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

    /// Mouse click — activates position.
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

    /// Update geometry after render.
    pub fn on_render(&mut self, nav_area: Option<Rect>) {
        self.nav_area = nav_area;
    }
}
