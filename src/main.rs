use std::{
    error::Error,
    io::{stdin, stdout},
    process::Command,
};

use std::io::{Read, Write};

use termion::raw::IntoRawMode;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize 'em all.
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let branches = branches();
    let selected = 0;

    loop {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).unwrap();
        write!(stdout, "{}", termion::clear::All)?;
        for branch in branches.iter().enumerate() {
            if selected == branch.0 {
                writeln!(stdout, "* {}\r", branch.1)?;
            } else {
                writeln!(stdout, "  {}\r", branch.1)?;
            }
        }
        match buf.as_str() {
            "q" => break,
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
