use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    command::run::CommandFlags,
    config::Language,
    platform_specific::get_config,
    sql::postgres::{describe_table, get_connection_pool, get_table_list},
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

pub async fn execute(flags: CommandFlags) {
    log::info!("execute action: run");

    let config = get_config();

    log::debug!("current config: {:?}", config);

    log::debug!("flags: {:?}", flags);

    // 1. 커넥션 정보가 없다면 에러 문구를 출력하고 종료합니다.
    let Some(database_pair) = config.default_database_pair else {
        println!("database connection pair is not set. try to [magiceye init] first.");
        return;
    };

    // 2. 커넥션 정보를 기반으로 실제 데이터베이스에 연결합니다.
    let base_connection_url = database_pair.base_connection.as_str();
    let target_connection_url = database_pair.target_connection.as_str();

    let base_connection_pool = get_connection_pool(base_connection_url).await;
    let target_connection_pool = get_connection_pool(target_connection_url).await;

    let base_connection_pool = match base_connection_pool {
        Ok(pool) => pool,
        Err(error) => {
            println!("failed to connect to base database: {:?}", error);
            return;
        }
    };

    println!(">> connected to base database");

    let target_connection_pool = match target_connection_pool {
        Ok(pool) => pool,
        Err(error) => {
            println!("failed to connect to target database: {:?}", error);
            return;
        }
    };

    println!(">> connected to target database");

    // 3. base 테이블 목록을 조회합니다.
    println!(">> fetching base table list...");
    let base_table_list = get_table_list(&base_connection_pool).await;

    // 해당 테이블별 상세 목록을 조회합니다.
    println!(">> fetching base table details...");
    let mut base_table_map = HashMap::new();

    for table_name in base_table_list {
        let base_table = describe_table(&base_connection_pool, &table_name).await;

        base_table_map.insert(table_name, base_table);
    }

    // 4. 대상 테이블 목록을 조회합니다.
    println!(">> fetching target table list...");
    let target_table_list = get_table_list(&target_connection_pool).await;

    // 해당 테이블별 상세 목록을 조회합니다.
    let mut target_table_map = HashMap::new();

    println!(">> fetching target table details...");
    for table_name in target_table_list {
        let target_table = describe_table(&target_connection_pool, &table_name).await;

        target_table_map.insert(table_name, target_table);
    }

    // 5. base 테이블을 기준점으로 삼아서, target 테이블과 비교합니다.
    // A. base에 있는데 target에 없는 것은 보고 대상입니다.
    // B. base에 있고 target에도 있지만, 내용이 다른 것도 보고 대상입니다.
    // C. base에 없고 target에만 있는 것은 보고 대상이 아닙니다. 무시합니다.

    let mut report = ReportSchema {
        report_table_list: vec![],
    };

    let table_count = base_table_map.len();
    let mut i = 0;

    println!(">> table count: {}", table_count);

    for (base_table_name, base_table) in base_table_map {
        let target_table = target_table_map.get(&base_table_name);

        println!(
            ">> comparing table: {}... ({i}/{table_count})",
            base_table_name
        );

        let mut has_report = false;

        let mut report_table = ReportTable {
            table_name: base_table_name.clone(),
            report_list: vec![],
        };

        match target_table {
            Some(target_table) => {
                for column in base_table.columns {
                    let target_column = target_table.columns.iter().find(|c| c.name == column.name);

                    let base_column_name = column.name;

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

                for index in base_table.indexes {
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

        i += 1;
    }

    println!(">> comparison done.");

    // 6. 보고서를 파일로 생성합니다.

    println!(">> saving report file...");

    let current_date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let report_file_name = format!("report_{}.json", current_date);

    let report_json = serde_json::to_string_pretty(&report).unwrap();

    std::fs::write(&report_file_name, &report_json).unwrap();

    println!("@ report file saved: {}", report_file_name);
}
