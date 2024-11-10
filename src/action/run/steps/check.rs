use std::collections::HashMap;

use crate::{action::run::{tui::{ComparingTable, ProgressEvent}, SenderContext}, config::Language, sql::{Column, Table}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ReportTable {
    table_name: String,
    report_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSchema {
    report_table_list: Vec<ReportTable>,
}

pub fn difference_check(
    context: &SenderContext,
    base_table_map: HashMap<String, Table>,
    target_table_map: HashMap<String, Table>,
) -> ReportSchema {
    let mut report = ReportSchema {
        report_table_list: vec![],
    };

    let table_count = base_table_map.len();

    _ = context.event_sender.send(ProgressEvent::ComparingTable(ComparingTable {
        total_count: table_count,
        current_count: 0,
    }));

    for (i, (base_table_name, base_table)) in base_table_map.into_iter().enumerate() {
        let target_table = target_table_map.get(&base_table_name);

        _ = context.event_sender.send(ProgressEvent::ComparingTable(ComparingTable {
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

                    if compare_column(
                        context,
                        &mut report_table,
                        &base_table,
                        column,
                        target_column,
                    ) {
                        has_report = true;
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

                                let report_text = match context.config.current_language {
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

                                let report_text = match context.config.current_language {
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

                                let report_text = match context.config.current_language {
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
                            let report_text = match context.config.current_language {
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
                    let target_foreign_key =
                        target_table.find_foreign_key_by_key_name(&foreign_key.name);

                    let base_foreign_key_name = &foreign_key.name;

                    match target_foreign_key {
                        Some(target_foreign_key) => {
                            // 외래키가 참조하는 테이블이 다르면 보고합니다.
                            if foreign_key.foreign_column != target_foreign_key.foreign_column {
                                let base_foreign_table_name =
                                    &foreign_key.foreign_column.table_name;
                                let base_foreign_column_name =
                                    &foreign_key.foreign_column.column_name;

                                let target_foreign_table_name =
                                    &target_foreign_key.foreign_column.table_name;
                                let target_foreign_column_name =
                                    &target_foreign_key.foreign_column.column_name;

                                let report_text = match context.config.current_language {
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
                        }
                        None => {
                            let report_text = match context.config.current_language {
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
                let report_text = match context.config.current_language {
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

    _ = context.event_sender.send(ProgressEvent::ComparingTable(ComparingTable {
        total_count: table_count,
        current_count: table_count,
    }));

    report
}

fn compare_column(
    context: &SenderContext,
    report_table: &mut ReportTable,
    base_table : &Table,
    base_column: &Column,
    target_column: Option<&Column>,
) -> bool {
    let mut has_report = false;

    let base_table_name = &base_table.name;

    let base_column_name = &base_column.name;

    match target_column {
        Some(target_column) => {
            if base_column.data_type != target_column.data_type {
                let base_data_type = &base_column.data_type;
                let target_data_type = &target_column.data_type;

                let report_text = match context.config.current_language {
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

            if base_column.comment != target_column.comment {
                let base_comment = &base_column.comment;
                let target_comment = &target_column.comment;

                let report_text = match context.config.current_language {
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

            if base_column.nullable != target_column.nullable {
                let base_nullable =
                    if base_column.nullable { "NULL" } else { "NOT NULL" };
                let target_nullable = if target_column.nullable {
                    "NULL"
                } else {
                    "NOT NULL"
                };

                let report_text = match context.config.current_language {
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

            if base_column.default != target_column.default {
                let base_default = &base_column.default;
                let target_default = &target_column.default;

                let report_text = match context.config.current_language {
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

            if base_column.is_auto_increment != target_column.is_auto_increment {
                let base_auto_increment = if base_column.is_auto_increment {
                    "AUTO_INCREMENT"
                } else {
                    "NOT AUTO_INCREMENT"
                };
                let target_auto_increment = if target_column.is_auto_increment {
                    "AUTO_INCREMENT"
                } else {
                    "NOT AUTO_INCREMENT"
                };

                let report_text = match context.config.current_language {
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
            let report_text = match context.config.current_language {
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

    has_report
}