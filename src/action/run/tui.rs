use crate::action::enter_tui;

use super::ReceiverContext;

pub(super) struct FetchingTableList {
    pub total_count: Option<usize>,
    pub current_count: usize,
}

impl FetchingTableList {
    // 소수점을 버린 N% 텍스트 반환
    fn percentage(&self) -> String {
        let percent = if let Some(total_count) = self.total_count {
            (self.current_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        format!("{:.0}%", percent)
    }
}

pub(super) struct ComparingTable {
    pub total_count: usize,
    pub current_count: usize,
}

impl ComparingTable {
    // 소수점을 버린 N% 텍스트 반환
    fn percentage(&self) -> String {
        let percent = (self.current_count as f64 / self.total_count as f64) * 100.0;

        format!("{:.0}%", percent)
    }
}

pub(super) enum ProgressEvent {
    Start,
    StartFetchingBaseTableList,
    StartFetchingTargetTableList,
    FetchingTableList(FetchingTableList),
    StartComparingTable,
    ComparingTable(ComparingTable),
    SavingReportFile,
    Finished,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    Start = 0,
    FetchingBaseTableList = 1,
    FetchingTargetTableList = 2,
    ComparingTable = 3,
    SavingReportFile = 4,
    Finished = 5,
}

const STEP_COUNT: usize = Step::SavingReportFile as usize + 1;

pub(super) fn run_progress_view(context: ReceiverContext) -> anyhow::Result<()> {
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

    let mut terminal = enter_tui();

    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();

    let mut current_step = Step::Start;

    let mut render_text = String::new();
    let mut stacked_text = String::new();
    let mut description_text = String::new();

    let mut error_text = None;

    let mut fetching_table_list_status = FetchingTableList {
        total_count: None,
        current_count: 0,
    };

    let mut comparison_table_status = ComparingTable {
        total_count: 0,
        current_count: 0,
    };

    let event_receiver = context.event_receiver;

    loop {
        description_text.clear();
        render_text.clear();

        // handle new event
        if let Ok(event) = event_receiver.try_recv() {
            match event {
                ProgressEvent::Start => {}
                ProgressEvent::StartFetchingBaseTableList => {
                    current_step = Step::FetchingBaseTableList;
                    stacked_text.push_str(">> Database Connected - DONE ☑ \n");
                }
                ProgressEvent::StartFetchingTargetTableList => {
                    current_step = Step::FetchingTargetTableList;
                    stacked_text.push_str(">> Fetching base table list - DONE ☑ \n");
                }
                ProgressEvent::FetchingTableList(fetching_table_list) => {
                    fetching_table_list_status = fetching_table_list;
                }
                ProgressEvent::StartComparingTable => {
                    current_step = Step::ComparingTable;
                    stacked_text.push_str(">> Fetching target table list - DONE ☑ \n");
                }
                ProgressEvent::ComparingTable(comparing_table) => {
                    comparison_table_status = comparing_table;
                }
                ProgressEvent::SavingReportFile => {
                    current_step = Step::SavingReportFile;
                    stacked_text.push_str(">> Comparing table - DONE ☑ \n");
                }
                ProgressEvent::Finished => {
                    current_step = Step::Finished;
                    stacked_text.push_str(">> Saving report file - DONE ☑ \n");
                }
                ProgressEvent::Error(error) => {
                    error_text = Some(error);
                }
            }
        }

        // rendering text 생성

        // 최상단 스텝 표시
        render_text.push_str(" **Step** ");
        for i in 0..STEP_COUNT {
            if i >= current_step as usize {
                render_text.push_str(" □");
            } else {
                render_text.push_str(" ■");
            }
        }
        render_text.push('\n');

        render_text.push_str(&stacked_text);

        match current_step {
            Step::Start => {
                render_text.push_str(">>> Database Connecting...\n");
            }
            Step::FetchingBaseTableList => {
                render_text.push_str(
                    format!(
                        ">>> Fetching base table list... - [{}]\n",
                        fetching_table_list_status.percentage()
                    )
                    .as_str(),
                );
            }
            Step::FetchingTargetTableList => {
                render_text.push_str(
                    format!(
                        ">>> Fetching target table list... - [{}]\n",
                        fetching_table_list_status.percentage()
                    )
                    .as_str(),
                );
            }
            Step::ComparingTable => {
                render_text.push_str(
                    format!(
                        ">>> Comparing table... - [{}]\n",
                        comparison_table_status.percentage()
                    )
                    .as_str(),
                );
            }
            Step::SavingReportFile => {
                render_text.push_str(">>> Saving report file... \n");
            }
            Step::Finished => {
                render_text.push_str(">> Finished\n");
                description_text.push_str("All process has been finished.\n");
                description_text.push_str("Press [Enter] to close the interactive window.\n");
            }
        }

        if let Some(ref error_text) = error_text {
            render_text.push_str("!! Error\n");
            render_text.push_str(format!("{error_text}\n").as_str());
            description_text.push_str("An error occurred during the process.\n");
            description_text.push_str("Press [Enter] to close the interactive window.\n");
        }

        if !description_text.is_empty() {
            render_text.push_str("\n\n\n");
            render_text.push_str("------------ Description ------------\n");

            render_text.push_str(description_text.as_str());
            render_text.push('\n');
        }

        let block = Block::default()
            .title("Report Generation")
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
                    match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Enter => {
                            if current_step == Step::Finished || error_text.is_some() {
                                break;
                            }
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
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
