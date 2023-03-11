use std::{
    error::Error,
    io::{stdin, stdout},
    process::Command,
};

use std::io::{Read, Write};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct BranchInfo {
    name: String,
    current: bool,
    status: &'static str,
    error: Option<String>,
}

impl BranchInfo {
    fn parse(line: impl AsRef<str>) -> Self {
        let current = line.as_ref().starts_with("*");
        Self {
            name: line.as_ref().split_at(2).1.to_owned(),
            current,
            status: "",
            error: None,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize 'em all.
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let branches: Vec<BranchInfo> = branches();
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
            Key::Delete => {
                let branch = &mut branches[selected];
                match delete_branch(&branch.name) {
                    Ok(_) => {
                        branch.branches.remove(selected);
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    Err(e) => {
                        writeln!(stdout, "Error: {}", e)?;
                    }
                }
                let stdout = Command::new("git")
                    .args(["branch", "-D"])
                    .arg(branch.trim())
                    .spawn()
                    .unwrap()
                    .wait();
                //let stdout: String = String::from_utf8_lossy(&stdout).into_owned();
                //writeln!(stdout, "{}", stdout)?;
            }
            c => {
                write!(stdout, "{:?}", c)?;
            }
        }

        stdout.flush().unwrap();
    }

    Ok(())
}

fn branches() -> Vec<BranchInfo> {
    let stdout = Command::new("git")
        .args(["branch", "--list", "--color=never"])
        .output()
        .unwrap()
        .stdout;

    let stdout: String = String::from_utf8_lossy(&stdout).into_owned();

    stdout.lines().map(BranchInfo::parse).collect()
}

fn delete_branch(branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["branch", "-D"])
        .arg(branch.trim())
        .output()
        .unwrap();
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into())
    }
}
