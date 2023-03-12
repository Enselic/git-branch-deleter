use std::{
    error::Error,
    io::{stdin, stdout},
    process::Command,
};

use std::io::Write;

use termion::raw::IntoRawMode;
use termion::{clear, input::TermRead};
use termion::{cursor, event::Key};

/// Info about a git branch
#[derive(Debug)]
struct Branch {
    /// Name of the git branch
    name: String,

    /// The latest stdout or stderr message from `git branch -d/-D {name}`
    status: String,
}

/// Keyboard input is mapped to one of these actions
#[derive(Debug)]
enum Action {
    Delete,
    ForceDelete,
    Quit,
    MoveDown,
    MoveUp,
    None,
}

#[derive(Debug)]
struct Selection {
    index: usize,
    max: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut keys = stdin().lock().keys();
    let mut stdout = stdout().lock().into_raw_mode()?;
    let (mut branches, max_branch_name_len) = local_git_branches();

    let mut selection = Selection::new(branches.len() - 1);
    loop {
        // Clear the screen only once to avoid flicker
        write!(&mut stdout, "{}", clear::All)?;
        write!(&mut stdout, "{}", cursor::Goto::default())?;

        print_branches(&mut stdout, &branches, selection.index, max_branch_name_len)?;

        let selected_branch = branches.get_mut(selection.index).unwrap();
        print_help(&mut stdout, max_branch_name_len, selected_branch)?;

        stdout.flush().unwrap();

        match key_to_action(keys.next().unwrap()?) {
            Action::MoveUp => selection.move_up(),
            Action::MoveDown => selection.move_down(),
            Action::Delete => selected_branch.delete("-d"),
            Action::ForceDelete => selected_branch.delete("-D"),
            Action::Quit => break,
            _ => {}
        }
    }

    Ok(())
}

fn print_branches<'a>(
    stdout: &mut dyn std::io::Write,
    branches: &[Branch],
    selected: usize,
    max_branch_name_len: usize,
) -> std::io::Result<()> {
    writeln!(stdout, "BRANCHES\r")?;
    writeln!(stdout, "\r")?;
    for (index, branch) in branches.into_iter().enumerate() {
        writeln!(
            stdout,
            "{}{}{}{}{}\r",
            if selected == index { "-> " } else { "   " },
            branch.name,
            " ".repeat(max_branch_name_len - branch.name.len()),
            branch.status,
            clear::AfterCursor
        )?;
    }

    Ok(())
}

fn print_help(
    stdout: &mut dyn std::io::Write,
    indentation: usize,
    branch: &mut Branch,
) -> std::io::Result<()> {
    let indentation = " ".repeat(indentation as usize);
    let branch_name = branch.name.as_str();

    writeln!(stdout, "\r")?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "COMMANDS\r")?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "    d{indentation}   git branch -d {branch_name}\r",)?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "    D{indentation}   git branch -D {branch_name}\r",)?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "    q{indentation}   quit\r")?;
    writeln!(stdout, "\r")?;

    Ok(())
}

fn local_git_branches() -> (Vec<Branch>, usize) {
    let stdout = Command::new("git")
        .args(["branch", "--list", "--color=never"])
        .env("HOME", "/no-config")
        .env("XDG_CONFIG_HOME", "/no-config")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap()
        .stdout;

    let stdout: String = String::from_utf8_lossy(&stdout).into_owned();

    let branches: Vec<Branch> = stdout.lines().map(Branch::from_line).collect();

    let max_branch_name_len = branches
        .iter()
        .map(|branch| branch.name.len())
        .max()
        .unwrap_or(0);

    (branches, max_branch_name_len)
}

fn key_to_action(key: Key) -> Action {
    match key {
        Key::Down | Key::Ctrl('n') | Key::Char('j') => Action::MoveDown,
        Key::Up | Key::Ctrl('p') | Key::Char('k') => Action::MoveUp,
        Key::Esc | Key::Char('q') | Key::Ctrl('c') => Action::Quit,
        Key::Delete | Key::Char('d') => Action::Delete,
        Key::Char('D') => Action::ForceDelete,
        _ => Action::None,
    }
}

impl Branch {
    fn from_line(line: impl AsRef<str>) -> Self {
        let status = line
            .as_ref()
            .starts_with("*")
            .then(|| "(current branch)".to_owned())
            .unwrap_or_default();

        Self {
            name: line.as_ref().split_at(2).1.to_owned(),
            status,
        }
    }

    fn delete(&mut self, delete_arg: &str) {
        let output = Command::new("git")
            .arg("branch")
            .arg(delete_arg)
            .arg(self.name.as_str())
            .output()
            .unwrap();
        self.status = String::from_utf8_lossy(if output.status.success() {
            &output.stdout
        } else {
            &output.stderr
        })
        .into();
    }
}

impl Selection {
    fn new(max: usize) -> Self {
        Self { index: 0, max }
    }

    fn move_up(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.index < self.max {
            self.index += 1;
        }
    }
}
