use std::{
    io::{stdin, stdout},
    process::Command,
};

use std::io::Write;

use termion::raw::IntoRawMode;

fn main() -> std::io::Result<()> {
    // Initialize 'em all.
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let branches = branches();
    let selected = 0;

    loop {
        write!(stdout, "{}", termion::clear::All)?;

        for branch in branches.iter().enumerate() {
            if selected == branch.0 {
                writeln!(stdout, "* {}\r", branch.1)?;
            } else {
                writeln!(stdout, "  {}\r", branch.1)?;
            }
        }

        stdout.flush().unwrap();
        let mut buf = String::new();
        let input = std::io::stdin().read_line(&mut buf).unwrap();
        eprintln!("input: {}", input);
    }
}

fn branches() -> Vec<String> {
    let stdout = Command::new("git")
        .args(["branch", "--list", "--color=never"])
        .output()
        .unwrap()
        .stdout;

    let stdout: String = String::from_utf8_lossy(&stdout)
        .replace("*", " ")
        .trim()
        .to_owned();

    stdout.lines().map(String::from).collect()
}
