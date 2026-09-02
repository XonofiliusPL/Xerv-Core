use std::io;
use std::time::Duration;

use crossterm::event::poll;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use xerv_core::api::CoreConfig;
use xerv_tui::app::App;
use xerv_tui::event::{read_event, Event};
use xerv_tui::ui::ui;

fn main() -> io::Result<()> {
    // Domyślna konfiguracja xerv-core (decyzja 3.8a: brak argumentów CLI).
    let cfg = CoreConfig::default();
    let state_path = cfg.data_dir.join(&cfg.state_filename);

    // App::try_new może zwrócić błąd inicjalizacji Rdzenia (decyzja 3.7b/3.9a).
    let mut app = match App::try_new(cfg, state_path) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("xerv-tui: core init failed: {e}");
            std::process::exit(1);
        }
    };

    // Terminal raw mode + alternate screen.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    // Restore.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()>
where
    B: ratatui::backend::Backend,
    std::io::Error: From<B::Error>,
{
    loop {
        terminal.draw(|f| ui(f, app))?;
        if poll(Duration::from_millis(200))? {
            let ev = read_event()?;
            app.handle_event(ev);
        } else {
            app.handle_event(Event::Tick);
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
