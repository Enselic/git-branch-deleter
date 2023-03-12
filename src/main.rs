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
    status: Option<String>,
}

enum Action {
    Delete,
    ForceDelete,
    Quit,
    MoveDown,
    MoveUp,
    None,
}

impl BranchInfo {
    fn parse(line: impl AsRef<str>) -> Self {
        let status = line
            .as_ref()
            .starts_with("*")
            .then(|| "(current branch)".to_owned());

        Self {
            name: line.as_ref().split_at(2).1.to_owned(),
            status,
        }
    }
}

impl Display for BranchInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

    let mut branches: Vec<BranchInfo> = get_local_branches();
    // let longest_branch_name = branches
    //     .iter()
    //     .map(|branch| branch.name.len())
    //     .max()
    //     .unwrap_or(0);
    let mut selected = 0;

    let mut keys = stdin.keys();

    // Only clear the screen once to avoid flicker
    write!(stdout, "{}", termion::clear::All,)?;

    loop {
        let mut delete_request = None;

        write!(stdout, "{}", termion::cursor::Goto::default())?;

        writeln!(stdout, "                                LOCAL GIT BRANCHES\r")?;
        writeln!(stdout, "================================================================================\r")?;
        writeln!(stdout, "\r")?;

        for (index, branch) in branches.iter().enumerate() {
            let prefix = if selected == index { "-> " } else { "   " };
            writeln!(stdout, "{prefix} {branch}{}\r", termion::clear::AfterCursor)?;
        }

        let branch_name = branches[selected].name.clone();

        writeln!(stdout, "\r")?;
        writeln!(stdout, "\r")?;
        writeln!(stdout, "                                    COMMANDS\r")?;
        writeln!(stdout, "================================================================================\r")?;
        writeln!(stdout, "\r")?;
        writeln!(stdout, "  d    git branch -d {branch_name}\r")?;
        writeln!(stdout, "  D    git branch -D {branch_name}\r")?;
        writeln!(stdout, "\r")?;
        writeln!(stdout, "  q    Quit app\r")?;
        writeln!(stdout, "\r")?;

        stdout.flush().unwrap();

        match key_to_action(keys.next().unwrap()?) {
            Action::Quit => break,
            Action::MoveUp => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Action::MoveDown => {
                if selected < branches.len() - 1 {
                    selected += 1;
                }
            }
            Action::Delete => {
                delete_request = Some("-d");
            }
            Action::ForceDelete => {
                delete_request = Some("-D");
            }
            _ => {}
        }

        if let Some(delete_request) = delete_request {
            branches.get_mut(selected).unwrap().status =
                Some(delete_branch(&branch_name, delete_request));
        }
    }

    Ok(())
}

fn get_local_branches() -> Vec<BranchInfo> {
    let stdout = Command::new("git")
        .args(["branch", "--list", "--color=never"])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "HOME",
            "/this-dir-does-not-exist-to-avoid-reading-git-config",
        )
        .env(
            "XDG_CONFIG_HOME",
            "/this-dir-does-not-exist-to-avoid-reading-git-config",
        )
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

fn key_to_action(key: Key) -> Action {
    match key {
        Key::Esc | Key::Char('q') | Key::Ctrl('c') => Action::Quit,
        Key::Up | Key::Ctrl('p') | Key::Char('k') => Action::MoveUp,
        Key::Down | Key::Ctrl('n') | Key::Char('j') => Action::MoveDown,
        Key::Delete | Key::Char('d') => Action::Delete,
        Key::Char('D') => Action::ForceDelete,
        _ => Action::None,
    }
}
