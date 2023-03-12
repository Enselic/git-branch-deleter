use std::{
    error::Error,
    fmt::{Display, Formatter},
    io::{stdin, stdout},
    process::Command,
};

use std::io::Write;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct BranchInfo {
    name: String,
    current: bool,
    status: Option<String>,
}

impl BranchInfo {
    fn parse(line: impl AsRef<str>) -> Self {
        let current = line.as_ref().starts_with("*");
        Self {
            name: line.as_ref().split_at(2).1.to_owned(),
            current,
            status: None,
        }
    }
}

impl Display for BranchInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.current { "* " } else { "  " })?;
        write!(f, "{}", self.name)?;
        if let Some(status) = &self.status {
            write!(f, " {status}")?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize 'em all.
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let mut branches: Vec<BranchInfo> = branches();
    let mut selected = 0;

    let mut keys = stdin.keys();
    loop {
        let mut delete_request = None;

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
            Key::Delete | Key::Ctrl('d') => {
                delete_request = Some("-d");
            }
            Key::Ctrl('D') => {
                delete_request = Some("-D");
            }
            c => {
                write!(stdout, "{:?}", c)?;
            }
        }

        if let Some(delete_request) = delete_request {
            let branch = &mut branches[selected];
            branch.status = Some(delete_branch(&branch.name, delete_request));
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

fn delete_branch(branch: &str, delete_arg: &str) -> String {
    let output = Command::new("git")
        .arg("branch")
        .arg(delete_arg)
        .arg(branch.trim())
        .output()
        .unwrap();
    let result: String = if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
    } else {
        String::from_utf8_lossy(&output.stderr)
    }
    .into();
    result.replace("\n", " ")
}
