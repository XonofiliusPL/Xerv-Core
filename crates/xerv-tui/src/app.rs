use xerv_core::api::{CoreConfig, XervCore};

use crate::event::Event;

/// Liczba komend w command barze Dashboardu. Punkt 3 nie implementuje akcji —
/// to są sloty na przyszłe moduły/agenty (zgodnie z kierunkiem rozszerzania Xerv).
pub const COMMAND_COUNT: usize = 6;

pub const COMMANDS: [&str; COMMAND_COUNT] = [
    "modules", "agents", "registry", "services", "logs", "config",
];

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

#[derive(Debug)]
pub struct App {
    pub core: XervCore,
    pub active_panel: Panel,
    pub should_quit: bool,
    pub selected_command: usize,
    /// Aktualizowany przez `Event::Hover` — Area ostatnio najechanej karty.
    pub hovered_card: Option<Area>,
    /// Czy mysz weszła w command bar ostatnio (do rozróżnienia kliknięcia).
    pub hovered_command: Option<Area>,
    /// Obszary kart (wypełniane przez ui::ui, czytane przez on_mouse).
    pub card_areas: Vec<(&'static str, Area)>,
    pub command_bar_area: Option<Area>,
    pub side_area: Option<Area>,
    pub main_area: Option<Area>,
}

impl App {
    pub fn try_new(
        config: CoreConfig,
        state_path: std::path::PathBuf,
    ) -> xerv_core::api::ApiResult<Self> {
        let core = XervCore::new(config, state_path)?;
        Ok(Self {
            core,
            active_panel: Panel::Side,
            should_quit: false,
            selected_command: 0,
            hovered_card: None,
            hovered_command: None,
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

    /// Obsługa kliknięcia — rozróżnia panel/command bar po obszarach zapisanych w `on_render`.
    pub fn on_click(&mut self, col: u16, row: u16) {
        if let Some(rect) = self.command_bar_area {
            if rect.contains(col, row) {
                // Oblicz indeks komendy względem szerokości command bar.
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
                self.active_panel = Panel::Main;
                return;
            }
        }
        if let Some(rect) = self.side_area {
            if rect.contains(col, row) {
                self.active_panel = Panel::Side;
                return;
            }
        }
        if let Some(rect) = self.main_area {
            if rect.contains(col, row) {
                self.active_panel = Panel::Main;
            }
        }
    }

    /// Aktualizuje hover na podstawie pozycji myszy.
    pub fn on_hover(&mut self, col: u16, row: u16) {
        // Command bar
        let mut cmd_hover: Option<Area> = None;
        if let Some(rect) = self.command_bar_area {
            if rect.contains(col, row) {
                cmd_hover = Some(rect);
            }
        }
        self.hovered_command = cmd_hover;

        // Cards (pierwszy rect, który zawiera punkt)
        let mut card_hover: Option<Area> = None;
        for (_name, rect) in &self.card_areas {
            if rect.contains(col, row) {
                card_hover = Some(*rect);
                break;
            }
        }
        self.hovered_card = card_hover;
    }

    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Quit => self.should_quit = true,
            Event::NextPanel => self.active_panel = self.active_panel.next(),
            Event::PrevPanel => self.active_panel = self.active_panel.prev(),
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
                // Akcja jeszcze nie zaimplementowana (scope: brak nowych funkcji biznesowych).
                // Na tym etapie SelectCommand tylko potwierdza fokus na Main i przesuwa zaznaczenie.
                self.active_panel = Panel::Main;
            }
            Event::ClickPanel(col, row) => self.on_click(col, row),
            Event::ClickCommand(col, row) => self.on_click(col, row),
            Event::Hover(col, row) => self.on_hover(col, row),
            Event::Refresh | Event::Tick => {
                // No-op.
            }
        }
    }
}

/// Snapshot obszarów UI wypełniany przez `ui::ui` i konsumowany przez `App::on_render`.
#[derive(Debug, Default)]
pub struct UiAreas {
    pub side: Option<Area>,
    pub main: Option<Area>,
    pub command_bar: Option<Area>,
    pub cards: Vec<(&'static str, Area)>,
}
