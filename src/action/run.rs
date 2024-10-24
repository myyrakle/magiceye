use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::database;

use crate::{
    command::run::CommandFlags,
    platform_specific::get_config,
    sql::postgres::{describe_table, get_connection_pool, get_table_list, ping},
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

    let target_connection_pool = match target_connection_pool {
        Ok(pool) => pool,
        Err(error) => {
            println!("failed to connect to target database: {:?}", error);
            return;
        }
    };

    // 3. base 테이블 목록을 조회합니다.
    let base_table_list = get_table_list(&base_connection_pool).await;

    // 해당 테이블별 상세 목록을 조회합니다.
    let mut base_table_map = HashMap::new();

    for table_name in base_table_list {
        let base_table = describe_table(&base_connection_pool, &table_name).await;

        base_table_map.insert(table_name, base_table);
    }

    // 4. 대상 테이블 목록을 조회합니다.
    let target_table_list = get_table_list(&target_connection_pool).await;

    // 해당 테이블별 상세 목록을 조회합니다.
    let mut target_table_map = HashMap::new();

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

    for (base_table_name, base_table) in base_table_map {
        let target_table = target_table_map.get(&base_table_name);

        let mut has_report = false;

        let mut report_table = ReportTable {
            table_name: base_table_name.clone(),
            report_list: vec![],
        };

        match target_table {
            Some(target_table) => {
                for column in base_table.columns {}

                for index in base_table.indexes {}
            }
            None => {
                // Table "{}" exists in the base database, but not in the target database.
                let report_text = format!(
                    "Table: {base_table_name}가 base 데이터베이스에는 있지만, target 데이터베이스에는 없습니다."
                );

                report_table.report_list.push(report_text);
                has_report = true;
            }
        }

        if has_report {
            report.report_table_list.push(report_table);
        }
    }

    // 6. 보고서를 파일로 생성합니다.
    let current_date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let report_file_name = format!("report_{}.json", current_date);

    let report_json = serde_json::to_string_pretty(&report).unwrap();

    std::fs::write(&report_file_name, &report_json).unwrap();

    println!("report file saved: {}", report_file_name);
}
