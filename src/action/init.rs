use std::{io, thread};

use crate::{
    action::{enter_tui, exit_tui},
    command::init::CommandFlags,
    config::{Config, DatabasePair},
    platform_specific::{get_config, save_config},
};

use super::TerminalType;

pub async fn execute(flags: CommandFlags) {
    log::info!("execute action: init");

    let config = get_config();

    log::debug!("current config: {:?}", config);

    log::debug!("flags: {:?}", flags);

    let mut terminal = enter_tui();
    interactive(&mut terminal, config).unwrap();
}

#[derive(Debug, PartialEq, Eq)]
#[repr(i32)]
enum Step {
    EnterLanguage = 0,
    EnterBaseConnection = 1,
    EnterTargetConnection = 2,
    PostProcess = 3,
    Finished = 4,
}

impl Default for Step {
    fn default() -> Self {
        Self::EnterBaseConnection
    }
}

impl Step {
    fn next(&self) -> Self {
        match self {
            Self::EnterLanguage => Self::EnterBaseConnection,
            Self::EnterBaseConnection => Self::EnterTargetConnection,
            Self::EnterTargetConnection => Self::PostProcess,
            Self::PostProcess => Self::Finished,
            Self::Finished => Self::Finished,
        }
    }
}

fn interactive(terminal: &mut TerminalType, mut config: Config) -> io::Result<()> {
    use std::io::stdout;

    use crossterm::event::{self, KeyCode, KeyEventKind};
    use crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    };
    use crossterm::ExecutableCommand;
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use ratatui::{
        style::{Color, Style},
        widgets::{Block, BorderType, Borders, Paragraph},
    };

    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();

    let backend = CrosstermBackend::new(stdout());

    let mut terminal = Terminal::new(backend).unwrap();

    let mut step = Step::default();

    let mut input_text = String::new();
    let mut base_connection = String::new();
    let mut target_connection = String::new();

    let mut stacked_text = String::new();
    let mut render_text = String::new();

    loop {
        render_text.clear();
        render_text.push_str(stacked_text.as_str());

        // 스텝별 전처리
        match step {
            Step::EnterLanguage => {}
            Step::EnterBaseConnection => {
                render_text.push_str("▶ Enter Base Connection URL: ");
                render_text.push_str(&input_text);
            }
            Step::EnterTargetConnection => {
                render_text.push_str("▶ Enter Target Connection URL: ");
                render_text.push_str(&input_text);
            }
            Step::PostProcess => {
                config.default_database_pair = Some(DatabasePair {
                    name: "default".to_string(),
                    database_type: crate::config::DatabaseType::Postgres,
                    base_connection: base_connection.clone(),
                    target_connection: target_connection.clone(),
                });

                log::debug!("new config: {:?}", config);

                save_config(&config);
                stacked_text.push_str("\nConfig file saved.\n");

                step = step.next();
            }
            Step::Finished => {
                render_text.push_str("\nGoodbye!\n");
            }
        }

        let block = Block::default()
            .title("Initilize Config")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .border_type(BorderType::Rounded);

        let paragraph = Paragraph::new(render_text.clone()).block(block);

        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(paragraph, area);
        })?;

        // 이벤트 핸들링
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match step {
                        Step::EnterLanguage => match key.code {
                            // KeyCode::Char('q') => {
                            //     break;
                            // }
                            // KeyCode::Esc => {
                            //     break;
                            // }
                            // KeyCode::Enter => {
                            //     step = step.next();
                            // }
                            _ => {}
                        },
                        Step::EnterBaseConnection => match key.code {
                            KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Char(c) => {
                                input_text.push(c);
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Backspace | KeyCode::Delete => {
                                input_text.pop();
                            }
                            KeyCode::Enter => {
                                std::mem::swap(&mut base_connection, &mut input_text);

                                step = step.next();

                                stacked_text.push_str(
                                    format!("Base Connection URL: {}\n", base_connection).as_str(),
                                );
                            }
                            _ => {}
                        },
                        Step::EnterTargetConnection => match key.code {
                            KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Char(c) => {
                                input_text.push(c);
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Backspace | KeyCode::Delete => {
                                input_text.pop();
                            }
                            KeyCode::Enter => {
                                std::mem::swap(&mut target_connection, &mut input_text);
                                step = step.next();

                                stacked_text.push_str(
                                    format!("Target Connection URL: {}\n", target_connection)
                                        .as_str(),
                                );
                            }
                            _ => {}
                        },
                        Step::PostProcess => {}
                        Step::Finished => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
                                exit_tui();

                                break;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();

    println!("Goodbye!");

    Ok(())
}
