use std::time::{SystemTime, UNIX_EPOCH};

use xerv_core::api::{CoreConfig, XervCore};

use crate::event::Event;

// ---- Agent Workspace data model (Point 3) ----------------------------------------
//
// Minimalny model stanu potrzebny do wyświetlenia Agent Workspace w TUI.
// Dane są czyste struktury (Serializable) — nie zależą od ratatui.
// `demo()` dostarcza placeholderowe dane demonstracyjne; prawdziwa integracja
// z Hermesem / agentami przyjdzie w późniejszych punktach.

/// Stan czasu trwania agenta — mapowany na human-readable uptime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent jest aktywnie pracujący.
    Running,
    /// Agent jest wstrzymany (oczekuje na interakcję).
    Idle,
    /// Agent dokończył pracę.
    Done,
    /// Agent się wyłamał.
    Error,
}

impl AgentStatus {
    /// Jednoliterowy znacznik UI (spójny z DESIGN.md — semantyka statusu).
    pub fn marker(self) -> &'static str {
        match self {
            AgentStatus::Running => "●",
            AgentStatus::Idle => "○",
            AgentStatus::Done => "✓",
            AgentStatus::Error => "✗",
        }
    }

    /// Czy status jest "pozytywny" (green) czy negatywny (red).
    pub fn is_ok(self) -> bool {
        matches!(
            self,
            AgentStatus::Running | AgentStatus::Idle | AgentStatus::Done
        )
    }
}

/// Czas działania agenta jako sekfundy od startu.
///
/// `started_at_unix` to unix timestamp startu; obliczamy uptime w locie.
#[derive(Debug, Clone, Copy)]
pub struct AgentUptime {
    started_at_unix: u64,
}

impl AgentUptime {
    pub fn new(started_at_unix: u64) -> Self {
        Self { started_at_unix }
    }

    /// Czytelny uptime: `12s`, `3m7s`, `2h15m`, `5d3h`.
    pub fn format(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(self.started_at_unix);
        let secs = now.saturating_sub(self.started_at_unix);
        if secs < 60 {
            format!("{secs}s")
        } else if secs < 3600 {
            format!("{}m{}s", secs / 60, secs % 60)
        } else if secs < 86_400 {
            format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
        } else {
            format!("{}d{}h", secs / 86_400, (secs % 86_400) / 3600)
        }
    }
}

/// Model stanu pojedynczego agenta w workspace.
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub status: AgentStatus,
    pub model: String,
    pub uptime: AgentUptime,
    pub current_task: String,
}

/// Typy worktree'ów (placeholder — brak integracji git na tym etapie).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorktreeKind {
    /// Worktree to bieżący katalog projektu.
    Current,
    /// Worktree to odlotowany (detached).
    Detached,
    /// Worktree to feature branch.
    Branch,
}

/// Pojedynczy worktree projektu.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: String,
    pub kind: WorktreeKind,
    pub active: bool,
}

/// Model projektu w ramach workspace.
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub worktrees: Vec<WorktreeInfo>,
}

/// Model sesji TUI — grupuje otwarte session-y.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub name: String,
    pub active: bool,
}

/// Pełny model workspace — hierarchia danych demodelowych dla głównego panelu.
///
/// Hierarchia: Workspace → Project → Worktree → Session → Agent
#[derive(Debug, Clone)]
pub struct WorkspaceModel {
    /// Nazwa bieżącego workspace.
    pub workspace_name: String,
    /// Lista projektów w workspace.
    pub projects: Vec<ProjectInfo>,
    /// Aktywna sesja TUI.
    pub active_session: Option<usize>,
    /// Lista otwartych sesji.
    pub sessions: Vec<SessionInfo>,
    /// Aktywny agent (index do `agents`).
    pub active_agent: usize,
    /// Lista agentów w workspace (placeholder dane).
    pub agents: Vec<AgentInfo>,
}

impl WorkspaceModel {
    /// Placeholder dane demonstracyjne — pełny workspace z jednym projektem,
    /// dwoma worktree'ami, jedną sesją i dwoma agentami (jeden aktywny).
    pub fn demo() -> Self {
        Self {
            workspace_name: "~/Work/xerv".to_string(),
            projects: vec![ProjectInfo {
                name: "xerv".to_string(),
                worktrees: vec![
                    WorktreeInfo {
                        path: "~/Work/xerv".to_string(),
                        kind: WorktreeKind::Current,
                        active: true,
                    },
                    WorktreeInfo {
                        path: "~/Work/xerv/.wt/docs".to_string(),
                        kind: WorktreeKind::Branch,
                        active: false,
                    },
                ],
            }],
            active_session: Some(0),
            sessions: vec![SessionInfo {
                name: "main".to_string(),
                active: true,
            }],
            active_agent: 0,
            agents: vec![
                AgentInfo {
                    name: "hermes-planner".to_string(),
                    status: AgentStatus::Running,
                    model: "claude-3-7-sonnet".to_string(),
                    uptime: AgentUptime::new(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0)
                            - 7200,
                    ),
                    current_task: "Implement Goal 3: Agent Workspace UI".to_string(),
                },
                AgentInfo {
                    name: "claude-coder".to_string(),
                    status: AgentStatus::Idle,
                    model: "claude-3-7-sonnet".to_string(),
                    uptime: AgentUptime::new(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0)
                            - 300,
                    ),
                    current_task: "(idle — awaiting task)".to_string(),
                },
            ],
        }
    }
}

/// Wpisy w sekcji "last activity" (dane demonstracyjne).
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    pub timestamp: String,
    pub content: String,
}

impl ActivityEntry {
    pub fn demo_entries() -> Vec<Self> {
        vec![
            ActivityEntry {
                timestamp: "2026-09-04 14:02".to_string(),
                content: "hermes-planner · task complete: refactor app.rs".to_string(),
            },
            ActivityEntry {
                timestamp: "2026-09-04 13:57".to_string(),
                content: "claude-coder · spawned for workspace sync".to_string(),
            },
            ActivityEntry {
                timestamp: "2026-09-04 13:51".to_string(),
                content: "hermes-planner · status: running (idle)".to_string(),
            },
        ]
    }
}

/// Model stanu aplikacji TUI — rozszerzony o workspace model.
///
/// Pola istniejące z Punktu 3 (panel, nawigacja) zachowane.
/// Nowe pole `workspace` trzyma dane demodelowane dla Agent Workspace.
#[derive(Debug)]
pub struct App {
    pub core: XervCore,
    pub active_panel: Panel,
    pub should_quit: bool,
    pub selected_command: usize,
    /// Aktywna pozycja sidebaru (cyan). Zmienia się przez klik/myś/hom/end/enter.
    pub side_active: usize,
    /// Kursor nawigacji sidebaru (↑/↓/klik/hover) — wyróżnienie białe.
    pub side_cursor: usize,
    /// Indeks pozycji na którą mysz wskazuje (magenta hover). None = brak hoveru.
    pub side_hover: Option<usize>,
    /// Obszary cards (wypełniane przez ui::ui, czytane przez on_mouse).
    pub card_areas: Vec<(&'static str, Area)>,
    pub command_bar_area: Option<Area>,
    pub side_area: Option<Area>,
    pub main_area: Option<Area>,
    /// Model danych Agent Workspace (Punkt 3). Dane demo.
    pub workspace: WorkspaceModel,
    /// Wpisy ostatniej aktywności (dane demo).
    pub activity: Vec<ActivityEntry>,
}

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

/// Struktura nawigacji sidebaru Xerv. Jedno źródło danych dla pozycji.
pub const SIDEBAR_ITEMS: [&str; 8] = [
    "Dashboard",
    "Agents",
    "Projects",
    "Services",
    "Packages",
    "Plugins",
    "Settings",
    "Help",
];

/// Liczba pozycji sidebaru (wynikająca z SIDEBAR_ITEMS).
pub const SIDEBAR_COUNT: usize = SIDEBAR_ITEMS.len();

/// Indeks domyślnej aktywnej pozycji (Dashboard).
pub const DEFAULT_SIDE_ACTIVE: usize = 0;

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
            side_active: DEFAULT_SIDE_ACTIVE,
            side_cursor: DEFAULT_SIDE_ACTIVE,
            side_hover: None,
            card_areas: Vec::new(),
            command_bar_area: None,
            side_area: None,
            main_area: None,
            workspace: WorkspaceModel::demo(),
            activity: ActivityEntry::demo_entries(),
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

    /// Oblicz indeks pozycji sidebaru na podstawie współrzędnych myszy.
    /// Zwraca Some(idx) jeśli kliknięto w obszar pozycji, None jeśli poza.
    pub fn side_index_at(&self, col: u16, row: u16) -> Option<usize> {
        let rect = self.side_area?;
        // Wewnętrzna przestrzeń sidebaru (za borderami):
        // x+1, y+1, width-2, height-2.
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
        // Pierwszy wiersz wewnątrz to nagłówek " NAVIGATION" (1 linia).
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

    /// Obsługa kliknięcia — rozróżnia panel/command bar/sidebar po obszarach zapisanych w `on_render`.
    pub fn on_click(&mut self, col: u16, row: u16) {
        // Command bar
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
                self.active_panel = Panel::Main;
                return;
            }
        }
        // Sidebar
        if let Some(_rect) = self.side_area {
            if let Some(idx) = self.side_index_at(col, row) {
                self.side_active = idx;
                self.side_cursor = idx;
                self.side_hover = Some(idx);
                self.active_panel = Panel::Side;
                return;
            }
        }
        // Main
        if let Some(rect) = self.main_area {
            if rect.contains(col, row) {
                self.active_panel = Panel::Main;
            }
        }
    }

    /// Aktualizuje hover na podstawie pozycji myszy.
    pub fn on_hover(&mut self, col: u16, row: u16) {
        // Command bar — tylko aktualizacja stanów karty (pospolity stan,
        // bez polegania na usuniętych polach hovered_command/hovered_card).
        let _cmd_hover: Option<Area> = self.command_bar_area.filter(|rect| rect.contains(col, row));

        // Sidebar — hover nad pozycją (magenta), nie zmienia aktywnej.
        let mut side_hover: Option<usize> = None;
        if let Some(_rect) = self.side_area {
            if let Some(idx) = self.side_index_at(col, row) {
                side_hover = Some(idx);
            }
        }
        self.side_hover = side_hover;

        // Cards (pierwszy rect, który zawiera punkt)
        let mut card_hover: Option<Area> = None;
        for (_name, rect) in &self.card_areas {
            if rect.contains(col, row) {
                card_hover = Some(*rect);
                break;
            }
        }
        let _card_hover = card_hover;
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
                // Akcja jeszcze nie zaimplementowana (scope: brak nowych funkcji biznesowych).
                // Na tym etapie SelectCommand tylko potwierdza fokus na Main.
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
                // Enter/Space potwierdza aktywną pozycję jako placeholder.
                // Na tym etapie nie otwieramy żadnych ekranów — tylko akceptacja.
                self.side_active = self.side_cursor;
                self.active_panel = Panel::Side;
            }
            Event::ClickPanel(col, row) => self.on_click(col, row),
            Event::ClickCommand(col, row) => self.on_click(col, row),
            Event::Hover(col, row) => self.on_hover(col, row),
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
