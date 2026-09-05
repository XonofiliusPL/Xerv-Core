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

mod cli;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);
    if let Some(first) = iter.next() {
        let first = first.as_str();
        if first == "-h" || first == "--help" {
            print_usage();
            return Ok(());
        }
        if first == "-V" || first == "--version" || first == "version" {
            cli::version();
            return Ok(());
        }
        if first == "install" {
            match cli::run_install() {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("xerv install: {e}");
                    std::process::exit(1);
                }
            }
        }
        if first == "uninstall" {
            match cli::run_uninstall() {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("xerv uninstall: {e}");
                    std::process::exit(1);
                }
            }
        }
        eprintln!("xerv: unknown command '{first}'");
        print_usage();
        std::process::exit(1);
    }

    // No args → Core TUI
    let cfg = CoreConfig::default();
    let state_path = cfg.data_dir.join(&cfg.state_filename);

    let mut app = match App::try_new(cfg, state_path) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("xerv: core init failed: {e}");
            std::process::exit(1);
        }
    };

    // Raw mode + alternate screen + mouse capture.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);
    drop(terminal);
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    result
}

fn print_usage() {
    eprintln!("Usage: xerv [COMMAND]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  (none)     Launch Xerv Core TUI");
    eprintln!("  install    Interactively install xerv (user-space)");
    eprintln!("  uninstall  Remove xerv installation");
    eprintln!("  version    Print xerv version");
    eprintln!("  help, -h   Print this help");
}

fn run_app<B>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()>
where
    B: ratatui::backend::Backend,
    std::io::Error: From<B::Error>,
{
    loop {
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
