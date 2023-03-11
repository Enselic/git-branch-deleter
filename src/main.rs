use std::{
    error::Error,
    io::{stdin, stdout},
    process::Command,
};

use std::io::{Read, Write};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize 'em all.
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let branches = branches();
    let mut selected = 0;

    let mut keys = stdin.keys();
    loop {
        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto::default()
        )?;
        for (index, branch) in branches.iter().enumerate() {
            if selected == index {
                writeln!(stdout, "* {branch}\r")?;
            } else {
                writeln!(stdout, "  {branch}\r")?;
            }
        }
        match keys.next().unwrap()? {
            Key::Esc | Key::Char('q') | Key::Ctrl('c') => break,
            Key::Up => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Key::Down => {
                if selected < branches.len() - 1 {
                    selected += 1;
                }
            }
            c => {
                write!(stdout, "{:?}", c)?;
            }
        }

        stdout.flush().unwrap();
    }

    Ok(())
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
