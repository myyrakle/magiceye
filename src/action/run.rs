use std::{collections::HashMap, sync::mpsc::{channel, Receiver, Sender}};

use serde::{Deserialize, Serialize};

use crate::{
    action::enter_tui, command::run::CommandFlags, config::{Config, DatabasePair, DatabaseType, Language}, platform_specific::get_config, sql::{mysql, postgres, ConnectionPool, Table}
};

#[derive(Debug, Serialize, Deserialize)]
struct ReportTable {
    table_name: String,
    report_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReportSchema {
    report_table_list: Vec<ReportTable>,
}

struct FetchingTableList {
    total_count: Option<usize>,
    current_count: usize,
}

impl FetchingTableList {
    // 소수점을 버린 N% 텍스트 반환
    fn percentage(&self) -> String {
        let percent =  if let Some(total_count) = self.total_count {
            (self.current_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        format!("{:.0}%", percent)
    }
}

struct ComparingTable {
    total_count: usize,
    current_count: usize,
}

impl ComparingTable {
    // 소수점을 버린 N% 텍스트 반환
    fn percentage(&self) -> String {
        let percent = (self.current_count as f64 / self.total_count as f64) * 100.0; 

        format!("{:.0}%", percent)
    }
}

enum ProgressEvent {
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

fn run_progress_view(
    event_receiver: Receiver<ProgressEvent>,
) -> anyhow::Result<()> {
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

    loop {
        description_text.clear();
        render_text.clear();

        // handle new event
        if let Ok(event) = event_receiver.try_recv() {
            match event {
                ProgressEvent::Start => {},
                ProgressEvent::StartFetchingBaseTableList =>  {
                    current_step = Step::FetchingBaseTableList;
                    stacked_text.push_str(">> Database Connected - DONE ☑ \n");
                },
                ProgressEvent::StartFetchingTargetTableList => {
                    current_step = Step::FetchingTargetTableList;
                    stacked_text.push_str(">> Fetching base table list - DONE ☑ \n");
                }
                ProgressEvent::FetchingTableList(fetching_table_list) => {
                    fetching_table_list_status = fetching_table_list;
                },
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
                render_text.push_str(format!(">>> Fetching base table list... - [{}]\n", fetching_table_list_status.percentage()).as_str());
            }
            Step::FetchingTargetTableList => {
                render_text.push_str(format!(">>> Fetching target table list... - [{}]\n", fetching_table_list_status.percentage()).as_str());
            }
            Step::ComparingTable => {
                render_text.push_str(format!(">>> Comparing table... - [{}]\n", comparison_table_status.percentage()).as_str());
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

async fn generate_report(sender : Sender<ProgressEvent>, config: Config, database_pair: DatabasePair) -> anyhow::Result<()> {
     // 2. 커넥션 정보를 기반으로 실제 데이터베이스에 연결합니다.
     _= sender.send(ProgressEvent::Start);
     let (base_connection_pool, target_connection_pool) = match connect_database(
         &database_pair).await {
         Ok(pools) => pools,
         Err(error) => {
             return Err(anyhow::anyhow!("failed to connect to database: {:?}", error));
         }
     };
 
     // 3. base 테이블 목록을 조회합니다.
     _ = sender.send(ProgressEvent::StartFetchingBaseTableList);
     let base_table_map = match get_table_list(&sender, &base_connection_pool).await {
         Ok(map) => map,
         Err(error) => {
             return Err(anyhow::anyhow!("failed to get base table list: {:?}", error));
         }
     };
 
     // 4. 대상 테이블 목록을 조회합니다.
     _ = sender.send(ProgressEvent::StartFetchingTargetTableList);
     let target_table_map = match get_table_list(&sender, &target_connection_pool).await {
         Ok(map) => map,
         Err(error) => {
            return Err(anyhow::anyhow!("failed to get target table list: {:?}", error));
         }
     };
 
     // 5. base 테이블을 기준점으로 삼아서, target 테이블과 비교합니다.
     // A. base에 있는데 target에 없는 것은 보고 대상입니다.
     // B. base에 있고 target에도 있지만, 내용이 다른 것도 보고 대상입니다.
     // C. base에 없고 target에만 있는 것은 보고 대상이 아닙니다. 무시합니다.
     _ = sender.send(ProgressEvent::StartComparingTable);
 
     let mut report = ReportSchema {
         report_table_list: vec![],
     };
 
     let table_count = base_table_map.len();
 
     _ = sender.send(ProgressEvent::ComparingTable(ComparingTable{
         total_count: table_count,
         current_count: 0,
     }));
     for (i, (base_table_name, base_table)) in base_table_map.into_iter().enumerate() {
         let target_table = target_table_map.get(&base_table_name);
 
         _ = sender.send(ProgressEvent::ComparingTable(ComparingTable{
             total_count: table_count,
             current_count: i + 1,
         }));
 
         let mut has_report = false;
 
         let mut report_table = ReportTable {
             table_name: base_table_name.clone(),
             report_list: vec![],
         };
 
         match target_table {
             Some(target_table) => {
                 for column in &base_table.columns {
                     let target_column = target_table.columns.iter().find(|c| c.name == column.name);
 
                     let base_column_name = &column.name;
 
                     match target_column {
                         Some(target_column) => {
                             if column.data_type != target_column.data_type {
                                 let base_data_type = &column.data_type;
                                 let target_data_type = &target_column.data_type;
 
                                 let report_text = match config.current_language {
                                     Language::Korean=>format!(
                                         "Column: {base_table_name}.{base_column_name}의 데이터 타입이 다릅니다. => {base_data_type} != {target_data_type}"
                                     ),
                                     Language::English=>format!(
                                         "Column: {base_table_name}.{base_column_name} has different data type. => {base_data_type} != {target_data_type}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
 
                             if column.comment != target_column.comment {
                                 let base_comment = &column.comment;
                                 let target_comment = &target_column.comment;
 
                                 let report_text = match config.current_language {
                                     Language::Korean=>format!(
                                         "Column: {base_table_name}.{base_column_name}의 코멘트가 다릅니다. => {base_comment} != {target_comment}"
                                     ),
                                     Language::English=>format!(
                                         "Column: {base_table_name}.{base_column_name} has different comment. => {base_comment} != {target_comment}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
 
                             if column.nullable != target_column.nullable {
                                 let base_nullable =
                                     if column.nullable { "NULL" } else { "NOT NULL" };
                                 let target_nullable = if target_column.nullable {
                                     "NULL"
                                 } else {
                                     "NOT NULL"
                                 };
 
                                 let report_text = match config.current_language {
                                     Language::Korean=>format!(
                                         "Column: {base_table_name}.{base_column_name}의 NULLABLE이 다릅니다. => {base_nullable} != {target_nullable}"
                                     ),
                                     Language::English=>format!(
                                         "Column: {base_table_name}.{base_column_name} has different nullable. => {base_nullable} != {target_nullable}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
 
                             if column.default != target_column.default {
                                 let base_default = &column.default;
                                 let target_default = &target_column.default;
 
                                 let report_text = match config.current_language {
                                     Language::Korean=>format!(
                                         "Column: {base_table_name}.{base_column_name}의 DEFAULT 값이 다릅니다. => {base_default} != {target_default}"
                                     ),
                                     Language::English=>format!(
                                         "Column: {base_table_name}.{base_column_name} has different default value. => {base_default} != {target_default}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
 
                             if column.is_auto_increment != target_column.is_auto_increment {
                                 let base_auto_increment = if column.is_auto_increment {
                                     "AUTO_INCREMENT"
                                 } else {
                                     "NOT AUTO_INCREMENT"
                                 };
                                 let target_auto_increment = if target_column.is_auto_increment {
                                     "AUTO_INCREMENT"
                                 } else {
                                     "NOT AUTO_INCREMENT"
                                 };
 
                                 let report_text = match config.current_language {
                                     Language::Korean=>format!( 
                                         "Column: {base_table_name}.{base_column_name}의 AUTO_INCREMENT 여부가 다릅니다. => {base_auto_increment} != {target_auto_increment}"
                                     ),
                                     Language::English=>format!(
                                         "Column: {base_table_name}.{base_column_name} has different AUTO_INCREMENT. => {base_auto_increment} != {target_auto_increment}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
                         }
                         None => {
                             let report_text = match config.current_language {
                                 Language::Korean=>format!(
                                     "Column: {base_table_name}.{base_column_name}가 base 데이터베이스에는 있지만, target 데이터베이스에는 없습니다."
                                 ),
                                 Language::English=>format!(
                                     "Column: {base_table_name}.{base_column_name} exists in the base database, but not in the target database."
                                 ),
                             };
 
                             report_table.report_list.push(report_text);
                             has_report = true;
                         }
                     }
                 }
 
                 for index in &base_table.indexes {
                     let target_index = target_table.indexes.iter().find(|i| i.name == index.name);
 
                     let base_index_name = &index.name;
 
                     match target_index {
                         Some(target_index) => {
                             if index.columns != target_index.columns {
                                 let base_columns = index.columns.join(", ");
                                 let target_columns = target_index.columns.join(", ");
 
                                 let report_text = match config.current_language {
                                     Language::Korean=>format!(
                                         "Index: {base_table_name}.{base_index_name}의 컬럼이 다릅니다. 순서까지 확인해주세요. => {base_columns} != {target_columns}"
                                     ),
                                     Language::English=>format!(
                                         "Index: {base_table_name}.{base_index_name} has different columns. Please check the order. => {base_columns} != {target_columns}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
 
                             if index.predicate != target_index.predicate {
                                 let base_predicate = &index.predicate;
                                 let target_predicate = &target_index.predicate;
 
                                 let report_text = match config.current_language {
                                     Language::Korean=>format!(
                                         "Index: {base_table_name}.{base_index_name}의 조건이 다릅니다. => {base_predicate} != {target_predicate}"
                                     ),
                                     Language::English=>format!(
                                         "Index: {base_table_name}.{base_index_name} has different predicate. => {base_predicate} != {target_predicate}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
 
                             if index.is_unique != target_index.is_unique {
                                 let base_uniqueness = if index.is_unique {
                                     "UNIQUE"
                                 } else {
                                     "NOT UNIQUE"
                                 };
                                 let target_uniqueness = if target_index.is_unique {
                                     "UNIQUE"
                                 } else {
                                     "NOT UNIQUE"
                                 };
 
                                 let report_text = match config.current_language {
                                     Language::Korean=>format!(
                                         "Index: {base_table_name}.{base_index_name}의 UNIQUE 여부가 다릅니다. => {base_uniqueness} != {target_uniqueness}"
                                     ),
                                     Language::English=>format!(
                                         "Index: {base_table_name}.{base_index_name} has different uniqueness. => {base_uniqueness} != {target_uniqueness}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
                         }
                         None => {
                             let report_text = match config.current_language {
                                 Language::Korean=>format!(
                                     "Index: {base_table_name}.{base_index_name}가 base 데이터베이스에는 있지만, target 데이터베이스에는 없습니다."
                                 ),
                                 Language::English=>format!(
                                     "Index: {base_table_name}.{base_index_name} exists in the base database, but not in the target database."
                                 ),
                             };
 
                             report_table.report_list.push(report_text);
                             has_report = true;
                         }
                     }
                 }
 
                 for foreign_key in base_table.foreign_keys() {
                     let target_foreign_key = target_table.find_foreign_key_by_key_name(
                         &foreign_key.name
                     );
 
                     let base_foreign_key_name = &foreign_key.name;
 
                     match target_foreign_key {
                         Some(target_foreign_key) => {
                             // 외래키가 참조하는 테이블이 다르면 보고합니다.
                           if foreign_key.foreign_column != target_foreign_key.foreign_column {
                                 let base_foreign_table_name = &foreign_key.foreign_column.table_name;
                                 let base_foreign_column_name = &foreign_key.foreign_column.column_name;
 
                                 let target_foreign_table_name = &target_foreign_key.foreign_column.table_name;
                                 let target_foreign_column_name = &target_foreign_key.foreign_column.column_name;
 
                                 let report_text = match config.current_language {
                                     Language::English=>format!(
                                         "Foreign Key: {base_table_name}.{base_foreign_key_name} references different column. => {base_foreign_table_name}.{base_foreign_column_name} != {target_foreign_table_name}.{target_foreign_column_name}"
                                     ),
                                     Language::Korean=>format!(
                                         "Foreign Key: {base_table_name}.{base_foreign_key_name}의 참조 컬럼이 다릅니다. => {base_foreign_table_name}.{base_foreign_column_name} != {target_foreign_table_name}.{target_foreign_column_name}"
                                     ),
                                 };
 
                                 report_table.report_list.push(report_text);
                                 has_report = true;
                             }
                         }, 
                         None => {
                             let report_text = match config.current_language {
                                 Language::English=>format!(
                                     "Foreign Key: {base_table_name}.{base_foreign_key_name} exists in the base database, but not in the target database."
                                 ),
                                 Language::Korean=>format!(
                                     "Foreign Key: {base_table_name}.{base_foreign_key_name}가 base 데이터베이스에는 있지만, target 데이터베이스에는 없습니다."
                                 ),
                             };
 
                             report_table.report_list.push(report_text);
                             has_report = true;
                         }
                     }
                 } 
             }
             None => {
                 let report_text = match config.current_language {
                     Language::Korean=>format!(
                         "Table: {base_table_name}가 base 데이터베이스에는 있지만, target 데이터베이스에는 없습니다."
                     ),
                     Language::English=>format!(
                         "Table: {base_table_name} exists in the base database, but not in the target database."
                     ),
                 };
 
                 report_table.report_list.push(report_text);
                 has_report = true;
             }
         }
 
         if has_report {
             report.report_table_list.push(report_table);
         }
     }
 
     _ = sender.send(ProgressEvent::ComparingTable(ComparingTable{
         total_count: table_count,
         current_count: table_count,
     }));
 
     // 6. 보고서를 파일로 생성합니다.
     _ = sender.send(ProgressEvent::SavingReportFile);
 
     let current_date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
     let report_file_name = format!("report_{}.json", current_date);
 
     let report_json = serde_json::to_string_pretty(&report).unwrap();
 
     std::fs::write(&report_file_name, &report_json).unwrap();
 
     _ = sender.send(ProgressEvent::Finished);
    
    Ok(())
}

pub async fn execute(flags: CommandFlags) {
    log::info!("execute action: run");

    let config = match get_config() {
        Ok(config) => config,
        Err(error) => {
            println!("failed to get config: {:?}", error);
            return;
        }
    };

    log::debug!("current config: {:?}", config);

    log::debug!("flags: {:?}", flags);

    // 1. 커넥션 정보가 없다면 에러 문구를 출력하고 종료합니다.
    let Some(database_pair) = config.default_database_pair.clone() else {
        // println!("database connection pair is not set. try to [magiceye init] first.");
        return;
    };

    let (sender, receiver) = channel();

    tokio::spawn(async move {
        if let Err(error) = generate_report(sender.clone(), config, database_pair).await {
           _ = sender.send(ProgressEvent::Error(format!("{error:?}")));
        }
    });

    if let Err(error) = run_progress_view(receiver) {
        println!("failed to run progress view: {:?}", error);
    }   
}


async fn connect_database(
    database_pair: &DatabasePair,
)  -> anyhow::Result<(ConnectionPool, ConnectionPool)> {
    let base_connection_url = &database_pair.base_connection;
    let target_connection_url = &database_pair.target_connection;
    let database_type = &database_pair.database_type;

    //println!(">> connecting to base databases...");
    let base_connection_pool = match database_type {
        DatabaseType::Postgres => postgres::get_connection_pool(base_connection_url).await,
        DatabaseType::Mysql => mysql::get_connection_pool(base_connection_url).await,
    };

    //println!(">> connecting to target databases...");
    let target_connection_pool = match database_type {
        DatabaseType::Postgres => postgres::get_connection_pool(target_connection_url).await,
        DatabaseType::Mysql => mysql::get_connection_pool(target_connection_url).await,
    };

    let base_connection_pool = match base_connection_pool {
        Ok(pool) => {
            //println!(">> connected to base database");
            pool
        }
        Err(error) => {
           return Err(anyhow::anyhow!("failed to connect to base database: {:?}", error));
        }
    };

    let target_connection_pool = match target_connection_pool {
        Ok(pool) => {
            //println!(">> connected to target database");
            pool
        }
        Err(error) => {
            return Err(anyhow::anyhow!("failed to connect to target database: {:?}", error));
        }
    };

   Ok( (base_connection_pool, target_connection_pool))
}

async fn get_table_list(
    sender: &Sender<ProgressEvent>,
    connection_pool: &ConnectionPool,
) ->anyhow::Result<HashMap<String, Table>> {
    let table_list_result = match connection_pool {
        ConnectionPool::Postgres(ref pool) => postgres::get_table_list(pool).await,
        ConnectionPool::MySQL(ref pool) => mysql::get_table_list(pool).await,
    };

    let table_list = match table_list_result {
        Ok(list) => list,
        Err(error) => {
            return Err(anyhow::anyhow!("failed to get table list: {:?}", error));
        }
    };

    let mut table_map = HashMap::new();

    for (i,table_name) in table_list.iter().enumerate() {
        _ = sender.send(ProgressEvent::FetchingTableList(FetchingTableList {
            total_count: Some(table_list.len()),
            current_count: i + 1,
        }));

        let table_result = match connection_pool {
            ConnectionPool::Postgres(ref pool) => postgres::describe_table(pool, &table_name).await,
            ConnectionPool::MySQL(ref pool) => mysql::describe_table(pool, &table_name).await,
        };

        let table = match table_result {
            Ok(table) => table,
            Err(error) => {
                return Err(anyhow::anyhow!("failed to describe table: {:?}", error));
            }
        };

        table_map.insert(table_name.to_owned(), table);
    }

    _ = sender.send(ProgressEvent::FetchingTableList(FetchingTableList {
        total_count: Some(table_list.len()),
        current_count: table_list.len(),
    }));

    Ok(table_map)
}

