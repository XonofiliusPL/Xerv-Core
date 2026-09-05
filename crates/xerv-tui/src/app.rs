use xerv_core::api::{CoreConfig, XervCore};

use crate::event::Event;

// ---- Core UI model ----------------------------------------------------------------
//
// Xerv Core UI ma trzy główne panele (CorePanel), któremi użytkownik
// przechodzi sekwencyjnie: Install → Onboarding → Main. Po zakończeniu
// podstawowego przygotowania Core trafia do Main — samodzielnego panelu
// ze stanem Rdzenia. Sidebar nawiguje pomiędzy CorePanel a pozycjami
// nawigacji (Addons, Settings, Help, Exit).

/// Główne panele Xerv Core — sekwencja startowa Install → Onboarding → Main.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorePanel {
    /// Instalacja/podstawowa konfiguracja — progres przygotowania Core.
    Install,
    /// Onboarding — progres konfiguracji Core (config/state ready).
    Onboarding,
    /// Główny panel — pełny stan Core po zakończeniu przygotowania.
    Main,
}

impl CorePanel {
    /// Następny panel w sekwencji startowej (z uwzględnieniem, że po Main
    /// nie ma dalej — zwraca sam siebie).
    pub fn next(self) -> Self {
        match self {
            CorePanel::Install => CorePanel::Onboarding,
            CorePanel::Onboarding => CorePanel::Main,
            CorePanel::Main => CorePanel::Main,
        }
    }
}

/// Krok w progresie Install/Onboarding — label + `done` (tylko faktycznie
/// gotowe kroki mają `done = true`; reszta to placeholder dla przyszłych
/// implementacji, nie udawana gotowość).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub label: &'static str,
    pub done: bool,
}

/// Kroki instalacji Core (placeholdery dla przyszłych integracji).
pub const INSTALL_STEPS: [Step; 2] = [
    Step {
        label: "core init",
        done: true,
    },
    Step {
        label: "config ready",
        done: false,
    },
];

/// Kroki onboardingu Core. `core init` i `config ready` są gotowe
/// (Core faktycznie je posiada); `workspace ready` i `system ready` są
/// placeholderami na przyszłe implementacje — `done = false`.
pub const ONBOARDING_STEPS: [Step; 4] = [
    Step {
        label: "core init",
        done: true,
    },
    Step {
        label: "config ready",
        done: true,
    },
    Step {
        label: "workspace ready",
        done: true,
    },
    Step {
        label: "system ready",
        done: false,
    },
];

/// Akcja w Command Barze — prezentuje dostępne akcje dla bieżącego kontekstu.
///
/// Command Bar nie jest drugą listą nawigacji — to pasek akcji, który
/// komplementuje Sidebar i przygotowuje UI pod przyszłe akcje agenta/workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandAction {
    /// Krótka nazwa akcji (wyświetlana jako label).
    pub label: &'static str,
    /// Unicode symbol ikony (Nerd Font glyphy, szerokość = 1 col).
    pub icon: char,
    /// Skrót klawiaturowy (wyświetlany po prawej stronie labelu).
    pub shortcut: &'static str,
    /// Czy akcja jest już implementowana? `false` = placeholder.
    pub ready: bool,
}

/// Lista akcji w Command Barze — uporządkowana od najważniejszej.
pub const COMMAND_ACTIONS: [CommandAction; 4] = [
    CommandAction {
        label: "install",
        icon: '\u{f019}',
        shortcut: "i",
        ready: false,
    },
    CommandAction {
        label: "onboarding",
        icon: '\u{f1ae}',
        shortcut: "o",
        ready: false,
    },
    CommandAction {
        label: "main",
        icon: '\u{f0db}',
        shortcut: "m",
        ready: false,
    },
    CommandAction {
        label: "config",
        icon: '\u{f009}',
        shortcut: "c",
        ready: false,
    },
];

/// Liczba akcji w command barze (wynikająca z COMMAND_ACTIONS).
pub const COMMAND_COUNT: usize = COMMAND_ACTIONS.len();

/// Nazwy slotów — zachowane dla kompatybilności z istniejącymi testami/kodem.
pub const COMMANDS: [&str; COMMAND_COUNT] = ["install", "onboarding", "main", "config"];

/// Pozycje nawigacji Sidebar — wyłącznie: Addons, Settings, Help, Exit.
pub const SIDEBAR_ITEMS: [&str; 4] = ["Addons", "Settings", "Help", "Exit"];

/// Liczba pozycji sidebaru.
pub const SIDEBAR_COUNT: usize = SIDEBAR_ITEMS.len();

/// Indeks domyślnej aktywnej pozycji w sidebarze (Addons).
pub const DEFAULT_SIDE_ACTIVE: usize = 0;

/// Główny podział paneli TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    /// Sidebar / nawigacja (lewy panel).
    Side,
    /// Main / dashboard (prawy panel treści).
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

/// Prostokąt w układzie współrzędnych terminala (kolumna/wiersz). Własny typ —
/// nie koliduje z `ratatui::layout::Rect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Area {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Area {
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.x
            && col < self.x.saturating_add(self.w)
            && row >= self.y
            && row < self.y.saturating_add(self.h)
    }
}

/// Model stanu aplikacji TUI — Xerv Core UI.
#[derive(Debug)]
pub struct App {
    pub core: XervCore,
    /// Aktywny panel Core UI (Install → Onboarding → Main).
    pub core_panel: CorePanel,
    /// Aktywna pozycja w bocznym panelu / nawigacji.
    pub side_active: usize,
    /// Kursor nawigacji (↑↓), wyróżniony białym kolorem gdy focus = Side.
    pub side_cursor: usize,
    /// Indeks pozycji na którą wskazuje mysz (magenta hover). None = brak.
    pub side_hover: Option<usize>,
    /// Indeks hoverowanego slocie command baru (magenta, nie zmienia selected).
    pub hovered_command: Option<usize>,
    /// Indeks aktualnie wybranego slocie command baru (cyan, keyboard focus).
    pub selected_command: usize,
    /// Nazwa hoverowanej karty dashboardu (magenta border). None = brak.
    pub dashboard_hover: Option<&'static str>,
    /// Czy aplikacja powinna wyjść.
    pub should_quit: bool,
    /// Focus między sidebar (Side) a panelem treści (Main).
    pub active_panel: Panel,
    /// Czy aktywnie wyświetlany jest ekran Help (Sidebar → Help).
    /// Help jest statycznym panelem, nie CorePanelem.
    pub show_help: bool,
    /// Obszary kart (wypełniane przez ui::ui, czytane przez hit-testy).
    pub card_areas: Vec<(&'static str, Area)>,
    pub command_bar_area: Option<Area>,
    pub side_area: Option<Area>,
    pub main_area: Option<Area>,
}

/// Snapshot obszarów UI wypełniany przez `ui::ui` i konsumowany przez
/// `App::on_render`.
#[derive(Debug, Default)]
pub struct UiAreas {
    pub side: Option<Area>,
    pub main: Option<Area>,
    pub command_bar: Option<Area>,
    pub cards: Vec<(&'static str, Area)>,
}

impl App {
    pub fn try_new(
        config: CoreConfig,
        state_path: std::path::PathBuf,
    ) -> xerv_core::api::ApiResult<Self> {
        let core = XervCore::new(config, state_path)?;
        Ok(Self {
            core,
            core_panel: CorePanel::Install,
            side_active: DEFAULT_SIDE_ACTIVE,
            side_cursor: DEFAULT_SIDE_ACTIVE,
            side_hover: None,
            hovered_command: None,
            selected_command: 0,
            dashboard_hover: None,
            should_quit: false,
            active_panel: Panel::Side,
            show_help: false,
            card_areas: Vec::new(),
            command_bar_area: None,
            side_area: None,
            main_area: None,
        })
    }

    /// Aktualizuje pozycje elementów UI po wyrenderowaniu.
    /// Wywoływane przez main.rs po każdym `terminal.draw(|f| ui(f, app))`.
    pub fn on_render(&mut self, areas: UiAreas) {
        self.card_areas = areas.cards;
        self.command_bar_area = areas.command_bar;
        self.side_area = areas.side;
        self.main_area = areas.main;
    }

    /// Oblicza indeks pozycji sidebaru na podstawie współrzędnych myszy.
    /// Zwraca Some(idx) jeśli kliknięto w obszar pozycji, None jeśli poza.
    pub fn side_index_at(&self, col: u16, row: u16) -> Option<usize> {
        let rect = self.side_area?;
        let inner_x = rect.x + 1;
        let inner_y = rect.y + 1;
        let inner_w = rect.w.saturating_sub(2);
        let inner_h = rect.h.saturating_sub(2);
        if col < inner_x || col >= inner_x.saturating_add(inner_w) {
            return None;
        }
        if row < inner_y || row >= inner_y.saturating_add(inner_h) {
            return None;
        }
        // Pierwszy wiersz wewnątrz = nagłówek " NAVIGATION".
        let first_item_row = inner_y + 1;
        if row < first_item_row {
            return None;
        }
        let idx = (row - first_item_row) as usize;
        if idx < SIDEBAR_COUNT {
            Some(idx)
        } else {
            None
        }
    }

    /// Obsługa kliknięcia — rozróżnia command bar/sidebar/main po
    /// obszarach zapisanych w `on_render`.
    pub fn on_click(&mut self, col: u16, row: u16) {
        // Command bar.
        if let Some(rect) = self.command_bar_area {
            if rect.contains(col, row) {
                if rect.w == 0 {
                    return;
                }
                let slot_w = rect.w / COMMAND_COUNT as u16;
                if slot_w == 0 {
                    return;
                }
                let rel = col.saturating_sub(rect.x) / slot_w;
                let idx = (rel as usize).min(COMMAND_COUNT - 1);
                self.selected_command = idx;
                self.hovered_command = Some(idx);
                self.active_panel = Panel::Main;
                return;
            }
        }
        // Sidebar.
        if let Some(_rect) = self.side_area {
            if let Some(idx) = self.side_index_at(col, row) {
                match SIDEBAR_ITEMS[idx] {
                    "Exit" => {
                        self.should_quit = true;
                        return;
                    }
                    "Help" => {
                        self.show_help = true;
                        self.active_panel = Panel::Main;
                        return;
                    }
                    _ => {
                        self.show_help = false;
                        self.side_active = idx;
                        self.side_cursor = idx;
                        self.side_hover = Some(idx);
                        self.active_panel = Panel::Side;
                        return;
                    }
                }
            }
        }
        // Main (dashboard cards).
        if let Some(rect) = self.main_area {
            if rect.contains(col, row) {
                self.active_panel = Panel::Main;
                self.dashboard_hover = self
                    .card_areas
                    .iter()
                    .find(|(_, r)| r.contains(col, row))
                    .map(|(name, _)| *name);
            }
        }
    }

    /// Aktualizuje hover na podstawie pozycji myszy.
    /// Hover jest *czystym nakładnikiem wizualnym* — nie zmienia active/cursor.
    pub fn on_hover(&mut self, col: u16, row: u16) {
        // Exit — hover nie aktywuje, nie trzeba przerywać.
        let _exit_hover = self
            .side_area
            .and_then(|rect| {
                if !rect.contains(col, row) {
                    return None;
                }
                self.side_index_at(col, row)
            })
            .filter(|idx| SIDEBAR_ITEMS[*idx] == "Exit");

        // Command bar.
        self.hovered_command = self.command_bar_area.and_then(|rect| {
            if !rect.contains(col, row) || rect.w == 0 {
                return None;
            }
            let slot_w = rect.w / COMMAND_COUNT as u16;
            if slot_w == 0 {
                return None;
            }
            let rel = col.saturating_sub(rect.x) / slot_w;
            let idx = (rel as usize).min(COMMAND_COUNT - 1);
            Some(idx)
        });

        // Sidebar — hover, nie zmienia aktywnej.
        self.side_hover = self.side_area.and_then(|rect| {
            if rect.contains(col, row) {
                self.side_index_at(col, row)
            } else {
                None
            }
        });
        // Upewnijmy się, że Exit hover nie zostaje przekształcony w side_hover=Some.
        // Exit jest pozycją sidebaru — hover na Exit rejestrujemy jako side_hover,
        // ale nie jako aktywacja (to robi on_click).

        // Dashboard cards.
        self.dashboard_hover = self
            .card_areas
            .iter()
            .find(|(_, rect)| rect.contains(col, row))
            .map(|(name, _)| *name);
    }

    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Quit => self.should_quit = true,
            Event::NextPanel => self.active_panel = self.active_panel.next(),
            Event::PrevPanel => self.active_panel = self.active_panel.prev(),
            Event::Refresh | Event::Tick => {
                // No-op.
            }
            Event::NextCommand => {
                self.selected_command = (self.selected_command + 1) % COMMAND_COUNT;
                self.active_panel = Panel::Main;
            }
            Event::PrevCommand => {
                self.selected_command = if self.selected_command == 0 {
                    COMMAND_COUNT - 1
                } else {
                    self.selected_command - 1
                };
                self.active_panel = Panel::Main;
            }
            Event::SelectCommand => {
                // Przejście do panelu odpowiadającego komendzie (placeholder).
                self.core_panel = match self.selected_command {
                    0 => CorePanel::Install,
                    1 => CorePanel::Onboarding,
                    _ => CorePanel::Main,
                };
                self.active_panel = Panel::Main;
            }
            Event::NavDown => {
                self.side_cursor = (self.side_cursor + 1) % SIDEBAR_COUNT;
                self.active_panel = Panel::Side;
            }
            Event::NavUp => {
                self.side_cursor = if self.side_cursor == 0 {
                    SIDEBAR_COUNT - 1
                } else {
                    self.side_cursor - 1
                };
                self.active_panel = Panel::Side;
            }
            Event::NavHome => {
                self.side_cursor = 0;
                self.active_panel = Panel::Side;
            }
            Event::NavEnd => {
                self.side_cursor = SIDEBAR_COUNT - 1;
                self.active_panel = Panel::Side;
            }
            Event::NavActivate => {
                match self.active_panel {
                    Panel::Side => {
                        let idx = self.side_cursor;
                        let name = SIDEBAR_ITEMS[idx.min(SIDEBAR_ITEMS.len() - 1)];
                        match name {
                            "Exit" => self.should_quit = true,
                            "Help" => {
                                self.show_help = true;
                                self.active_panel = Panel::Main;
                            }
                            _ => {
                                self.show_help = false;
                                self.side_active = idx;
                            }
                        }
                    }
                    Panel::Main => {
                        if self.show_help {
                            // Escape z Help — wróć do bieżącego panelu Core.
                            self.show_help = false;
                        } else {
                            // Enter w Main: przejdź do następnego CorePanela
                            // (Install → Onboarding → Main).
                            self.core_panel = self.core_panel.next();
                        }
                    }
                }
            }
            Event::Click(col, row) => self.on_click(col, row),
            Event::Hover(col, row) => self.on_hover(col, row),
        }
    }
}
