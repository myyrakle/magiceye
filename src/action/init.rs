use std::io;

use crate::{
    action::{enter_tui, exit_tui},
    command::init::CommandFlags,
    config::{Config, DatabasePair, DatabaseType, Language},
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
    EnterDatabaseType = 0,
    EnterLanguage,
    EnterBaseConnection,
    EnterTargetConnection,
    PostProcess,
    Finished,
}

impl Default for Step {
    fn default() -> Self {
        Self::EnterDatabaseType
    }
}

impl Step {
    fn next(&self) -> Self {
        match self {
            Self::EnterDatabaseType => Self::EnterLanguage,
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
    use ratatui::{
        style::{Color, Style},
        widgets::{Block, BorderType, Borders, Paragraph},
    };

    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();

    let mut step = Step::default();

    let mut base_connection = config
        .default_database_pair
        .clone()
        .unwrap_or_default()
        .base_connection
        .clone();
    let mut target_connection = config
        .default_database_pair
        .clone()
        .unwrap_or_default()
        .target_connection;

    let mut current_databse_type = config
        .default_database_pair
        .clone()
        .unwrap_or_default()
        .database_type;

    let mut current_language = config.current_language.clone();

    let mut stacked_text = String::new();
    let mut render_text = String::new();
    let mut description_text = String::new();

    loop {
        description_text.clear();
        render_text.clear();
        render_text.push_str(stacked_text.as_str());

        // 스텝별 전처리
        match step {
            Step::EnterDatabaseType => {
                render_text.push_str("▶ Select Database Type");

                for database_type in DatabaseType::list() {
                    render_text.push_str("\n");
                    render_text.push_str(format!("  - {database_type:?}").as_str());

                    if database_type == current_databse_type {
                        render_text.push_str(" ◀");
                    }
                }
            }
            Step::EnterLanguage => {
                render_text.push_str("▶ Select Language");

                for language in Language::list() {
                    render_text.push('\n');
                    render_text.push_str(format!("  - {language:?}").as_str());

                    if language == current_language {
                        render_text.push_str(" ◀");
                    }
                }
            }
            Step::EnterBaseConnection => {
                render_text.push_str("▶ Enter Base Connection URL: ");
                render_text.push_str(&base_connection);

                match current_databse_type {
                    DatabaseType::Postgres => {
                        description_text =
                            format!("Enter the full connection URL of the base database. (e.g. postgres://user:password@host:port/dbname)\n");

                        description_text.push_str("");
                    }
                    DatabaseType::Mysql => {
                        // not yet implemented
                    }
                }
            }
            Step::EnterTargetConnection => {
                render_text.push_str("▶ Enter Target Connection URL: ");
                render_text.push_str(&target_connection);

                match current_databse_type {
                    DatabaseType::Postgres => {
                        description_text =
                            format!("Enter the full connection URL of the target database. (e.g. postgres://user:password@host:port/dbname)\n");

                        description_text.push_str("");
                    }
                    DatabaseType::Mysql => {
                        // not yet implemented
                    }
                }
            }
            Step::PostProcess => {
                config.default_database_pair = Some(DatabasePair {
                    name: "default".to_string(),
                    database_type: current_databse_type.clone(),
                    base_connection: base_connection.clone(),
                    target_connection: target_connection.clone(),
                });
                config.current_language = current_language.clone();

                log::debug!("new config: {:?}", config);

                save_config(&config);
                stacked_text.push_str("\nConfig file saved.\n");

                step = step.next();
            }
            Step::Finished => {
                render_text.push_str("\nGoodbye!\n");
            }
        }

        if !description_text.is_empty() {
            render_text.push_str("\n\n\n");
            render_text.push_str("------------ Description ------------\n");

            render_text.push_str(description_text.as_str());
            render_text.push('\n');

            render_text.push_str("- Press [Enter] to confirm, [ESC] to cancel.");
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
                        Step::EnterDatabaseType => match key.code {
                            KeyCode::Down => {
                                current_databse_type = current_databse_type.next();
                            }
                            KeyCode::Up => {
                                current_databse_type = current_databse_type.prev();
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Enter => {
                                step = step.next();

                                stacked_text.push_str(
                                    format!("Database Type: {current_databse_type:?}\n").as_str(),
                                );
                            }
                            _ => {}
                        },
                        Step::EnterLanguage => match key.code {
                            KeyCode::Down => {
                                current_language = current_language.next();
                            }
                            KeyCode::Up => {
                                current_language = current_language.prev();
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Enter => {
                                step = step.next();

                                stacked_text
                                    .push_str(format!("Language: {current_language:?}\n").as_str());
                            }
                            _ => {}
                        },
                        Step::EnterBaseConnection => match key.code {
                            KeyCode::Char(c) => {
                                base_connection.push(c);
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Backspace => {
                                base_connection.pop();
                            }
                            KeyCode::Delete => {
                                base_connection.clear();
                            }
                            KeyCode::Enter => {
                                step = step.next();

                                stacked_text.push_str(
                                    format!("Base Connection URL: {}\n", base_connection).as_str(),
                                );
                            }
                            _ => {}
                        },
                        Step::EnterTargetConnection => match key.code {
                            KeyCode::Char(c) => {
                                target_connection.push(c);
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Backspace => {
                                target_connection.pop();
                            }
                            KeyCode::Delete => {
                                target_connection.clear();
                            }
                            KeyCode::Enter => {
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
