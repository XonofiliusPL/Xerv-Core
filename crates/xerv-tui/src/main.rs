use crossterm::event::{self as ct_event, DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

use xerv_core::api::CoreConfig;
use xerv_tui::app::App;
use xerv_tui::event::{read_event, Event};
use xerv_tui::ui::ui;

fn main() -> io::Result<()> {
    // Domyślna konfiguracja xerv-core (brak argumentów CLI).
    let cfg = CoreConfig::default();
    let state_path = cfg.data_dir.join(&cfg.state_filename);

    let mut app = match App::try_new(cfg, state_path) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("xerv-tui: core init failed: {e}");
            std::process::exit(1);
        }
    };

    // Raw mode + alternate screen + mouse capture.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run z pełnym restore na wyjściu (drop terminal + disable_raw_mode +
    // LeaveAlternateScreen + DisableMouseCapture). Gwarancja cleanup nawet
    // na panic — drop terminal + restore po za pętlą.
    let result = run_app(&mut terminal, &mut app);
    drop(terminal);
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    result
}

fn run_app<B>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()>
where
    B: ratatui::backend::Backend,
    std::io::Error: From<B::Error>,
{
    loop {
        // `ui` rysuje i zapisuje geometrię (nav_area) do `app` jako side effect.
        terminal.draw(|f| ui(f, app))?;
        if ct_event::poll(Duration::from_millis(200))? {
            let ev = read_event()?;
            app.handle_event(ev);
        } else {
            app.handle_event(Event::Tick);
        }
        if app.should_quit {
            let _ = app.core.shutdown();
            return Ok(());
        }
    }
}
