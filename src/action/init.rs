use std::io;

use crate::{
    action::{enter_tui, exit_tui},
    command::init::CommandFlags,
    config::{Config, DatabasePair, Language},
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
        Self::EnterLanguage
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

    let mut current_language = config.current_language.clone();

    let mut stacked_text = String::new();
    let mut render_text = String::new();

    loop {
        render_text.clear();
        render_text.push_str(stacked_text.as_str());

        // 스텝별 전처리
        match step {
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
            }
            Step::EnterTargetConnection => {
                render_text.push_str("▶ Enter Target Connection URL: ");
                render_text.push_str(&target_connection);
            }
            Step::PostProcess => {
                config.default_database_pair = Some(DatabasePair {
                    name: "default".to_string(),
                    database_type: crate::config::DatabaseType::Postgres,
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
