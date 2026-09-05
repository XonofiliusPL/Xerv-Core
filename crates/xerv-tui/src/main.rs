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
use xerv_tui::cli;
use xerv_tui::event::{read_event, Event};
use xerv_tui::ui::ui;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);
    if let Some(first) = iter.next() {
        let first = first.as_str();
        match first {
            "-h" | "--help" | "help" => {
                print_usage();
                return Ok(());
            }
            "-V" | "--version" | "version" => {
                cli::version();
                return Ok(());
            }
            "install" => match cli::run_install() {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("xerv install: {e}");
                    std::process::exit(1);
                }
            },
            "uninstall" => match cli::run_uninstall() {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("xerv uninstall: {e}");
                    std::process::exit(1);
                }
            },
            "update" => match cli::run_update() {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("xerv update: {e}");
                    std::process::exit(1);
                }
            },
            _ => {
                eprintln!("xerv: unknown command '{first}'");
                print_usage();
                std::process::exit(1);
            }
        }
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

    // Auto-check update in background — does not block TUI.
    // Checks GitHub Releases on startup; result sets `app.update_available`.
    let update_thread = std::thread::spawn(move || -> Option<String> {
        use xerv_core::api::api_version;
        let mut res = None;
        if let Ok(info) = xerv_tui::update::check_for_update(&api_version()) {
            res = info.map(|r| r.version.to_string());
        }
        res
    });

    // Raw mode + alternate screen + mouse capture.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app, Some(update_thread));
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
    eprintln!("  update     Update xerv to latest release");
    eprintln!("  version    Print xerv version");
    eprintln!("  help, -h   Print this help");
}

fn run_app<B>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mut update_thread: Option<std::thread::JoinHandle<Option<String>>>,
) -> io::Result<()>
where
    B: ratatui::backend::Backend,
    std::io::Error: From<B::Error>,
{
    let mut update_checked = false;
    loop {
        // Check once in background whether the update thread finished.
        if !update_checked
            && update_thread
                .as_ref()
                .map(|h| h.is_finished())
                .unwrap_or(false)
        {
            if let Some(handle) = update_thread.take() {
                if let Ok(Some(v)) = handle.join() {
                    app.update_available = Some(v);
                }
            }
            update_checked = true;
        }

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
