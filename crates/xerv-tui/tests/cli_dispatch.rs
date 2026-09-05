//! Testy dispatchu CLI (main.rs) — help/-h/--help oraz unknown command.
//!
//! Builduje bin `xerv` i uruchamia go z różnymi arg w podprocesie.
//! Nie testuje TUI (brak args), ani install/uninstall/update (wymagają network/filesystem).

use std::process::Command;

fn xerv_binary() -> String {
    // Kompilacja release nie jest wymagana — debug jest dostateczny.
    // `CARGO_BIN_EXE_xerv` jest dostępny w integration tests.
    env!("CARGO_BIN_EXE_xerv").to_string()
}

#[test]
fn xerv_help_shows_usage() {
    let bin = xerv_binary();
    let out = Command::new(&bin)
        .arg("help")
        .output()
        .expect("failed to run xerv help");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "xerv help should exit 0, got stdout={stdout:?} stderr={stderr:?}"
    );
    assert!(
        stdout.contains("Usage:") || stderr.contains("Usage:"),
        "xerv help should print usage"
    );
}

#[test]
fn xerv_h_shows_usage() {
    let bin = xerv_binary();
    let out = Command::new(&bin)
        .arg("-h")
        .output()
        .expect("failed to run xerv -h");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("Usage:") || stderr.contains("Usage:"),
        "xerv -h should print usage"
    );
}

#[test]
fn xerv_long_help_shows_usage() {
    let bin = xerv_binary();
    let out = Command::new(&bin)
        .arg("--help")
        .output()
        .expect("failed to run xerv --help");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("Usage:") || stderr.contains("Usage:"),
        "xerv --help should print usage"
    );
}

#[test]
fn xerv_help_h_and_long_help_produce_same_output() {
    let bin = xerv_binary();
    let run = |arg: &str| -> String {
        let out = Command::new(&bin).arg(arg).output().expect("failed to run");
        // print_usage używa eprintln — output w stderr.
        String::from_utf8_lossy(&out.stderr).to_string()
    };
    let h = run("-h");
    let help = run("help");
    assert_eq!(
        h, help,
        "xerv -h and xerv help must produce identical output"
    );
}

#[test]
fn xerv_unknown_command_exits_nonzero() {
    let bin = xerv_binary();
    let out = Command::new(&bin)
        .arg("bogus-command")
        .output()
        .expect("failed to run xerv bogus");
    assert!(
        !out.status.success(),
        "xerv bogus-command should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown command 'bogus-command'"),
        "stderr should mention unknown command: {stderr}"
    );
}

#[test]
fn xerv_version_exits_zero() {
    let bin = xerv_binary();
    let out = Command::new(&bin)
        .arg("--version")
        .output()
        .expect("failed to run xerv --version");
    assert!(out.status.success(), "--version should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("xerv "), "version output: {stdout}");
}
