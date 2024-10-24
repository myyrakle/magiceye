pub(crate) mod init;
pub(crate) mod run;

use std::io::{stdout, Stdout};

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

type TerminalType = Terminal<CrosstermBackend<Stdout>>;

pub fn enter_tui() -> TerminalType {
    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();

    let backend = CrosstermBackend::new(stdout());

    Terminal::new(backend).unwrap()
}

pub fn exit_tui() {
    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
