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
    Checkout,
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

/// To line things up nicely
const MARGIN: &str = "   ";

fn main() -> Result<(), Box<dyn Error>> {
    let mut keys = stdin().lock().keys();
    let mut stdout = stdout().lock().into_raw_mode()?;
    let (mut branches, max_branch_name_len) = local_git_branches();
    let mut selection = Selection::new(branches.len() - 1);

    // Clear the screen only once to avoid flicker
    write!(&mut stdout, "{}", clear::All)?;
    loop {
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
            Action::Checkout => {
                write!(&mut stdout, "{}", clear::All)?;
                write!(&mut stdout, "{}", cursor::Goto::default())?;
                stdout.flush()?;
                drop(stdout); // Drop stdout to release raw terminal mode
                selected_branch.checkout()?;
                std::io::stdout().lock().flush()?;
                break; // Auto-quit
            }
            Action::Quit => break,
            Action::None => {}
        }
    }

    Ok(())
}

fn print_branches(
    stdout: &mut dyn std::io::Write,
    branches: &[Branch],
    selected: usize,
    max_branch_name_len: usize,
) -> std::io::Result<()> {
    writeln!(stdout, "BRANCHES\r")?;
    writeln!(stdout, "\r")?;
    for (index, branch) in branches.iter().enumerate() {
        writeln!(
            stdout,
            "{}{}{}{MARGIN}{}{}\r",
            if selected == index { "-> " } else { "   " },
            branch.name,
            " ".repeat(max_branch_name_len - branch.name.len()),
            branch.status,
            clear::AfterCursor
        )?;
    }

    Ok(())
}

#[rustfmt::skip]
fn print_help(
    stdout: &mut dyn std::io::Write,
    indentation: usize,
    branch: &mut Branch,
) -> std::io::Result<()> {
    let command_len = 9; // "c / Enter".len()
    let pad = " ".repeat(indentation.saturating_sub(command_len));
    let branch_name = branch.name.as_str();

    writeln!(stdout, "\r")?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "COMMANDS\r")?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "   d        {pad}{MARGIN}git branch -d {branch_name}\r",)?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "   D        {pad}{MARGIN}git branch -D {branch_name}\r",)?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "   c / Enter{pad}{MARGIN}git checkout {branch_name}\r",)?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "   q / Esc  {pad}{MARGIN}quit\r")?;
    writeln!(stdout, "\r")?;

    Ok(())
}

fn local_git_branches() -> (Vec<Branch>, usize) {
    // Do not set e.g. GIT_CONFIG_NOSYSTEM, because we want the branch order to
    // match what the user is used to
    let stdout = Command::new("git")
        .args(["branch", "--list", "--color=never"])
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
        Key::Down | Key::Right | Key::Ctrl('n') | Key::Char('j') => Action::MoveDown,
        Key::Up | Key::Left | Key::Ctrl('p') | Key::Char('k') => Action::MoveUp,
        Key::Esc | Key::Char('q') | Key::Ctrl('c') => Action::Quit,
        Key::Delete | Key::Char('d') => Action::Delete,
        Key::Char('D') => Action::ForceDelete,
        Key::Char('c' | '\n') => Action::Checkout,
        _ => Action::None,
    }
}

impl Branch {
    fn from_line(line: impl AsRef<str>) -> Self {
        let status = line
            .as_ref()
            .starts_with('*')
            .then(|| "(current branch)".to_owned())
            .unwrap_or_default();

        Self {
            name: line.as_ref().split_at(2).1.to_owned(),
            status,
        }
    }

    fn run_cmd(&mut self, mut cmd: Command) {
        let output = cmd.output().unwrap();
        self.status = String::from_utf8_lossy(&output.stderr).to_string()
            + &String::from_utf8_lossy(&output.stdout);
        self.status = self.status.replace('\n', " ");
    }

    fn delete(&mut self, delete_arg: &str) {
        let mut cmd = Command::new("git");
        cmd.arg("branch").arg(delete_arg).arg(self.name.as_str());
        self.run_cmd(cmd);
    }

    fn checkout(&mut self) -> std::io::Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("checkout")
            .arg("--progress")
            .arg(self.name.as_str());

        // Run cmd which will make it print its output
        cmd.spawn()?.wait()?;
        Ok(())
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
