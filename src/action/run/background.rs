#[path = "./steps/mod.rs"]
mod steps;

use super::{tui::ProgressEvent, SenderContext};

pub(super) async fn generate_report(context: SenderContext) -> anyhow::Result<()> {
    // 1. 커넥션 정보를 기반으로 실제 데이터베이스에 연결합니다.
    _ = context.event_sender.send(ProgressEvent::Start);
    let (base_connection_pool, target_connection_pool) =
        match steps::connect_database(&context.database_pair).await {
            Ok(pools) => pools,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "failed to connect to database: {:?}",
                    error
                ));
            }
        };

    // 2. base 테이블 목록을 조회합니다.
    _ = context
        .event_sender
        .send(ProgressEvent::StartFetchingBaseTableList);
    let base_table_map = match steps::get_table_list(&context, &base_connection_pool).await {
        Ok(map) => map,
        Err(error) => {
            return Err(anyhow::anyhow!(
                "failed to get base table list: {:?}",
                error
            ));
        }
    };

    // 3. 대상 테이블 목록을 조회합니다.
    _ = context
        .event_sender
        .send(ProgressEvent::StartFetchingTargetTableList);
    let target_table_map = match steps::get_table_list(&context, &target_connection_pool).await {
        Ok(map) => map,
        Err(error) => {
            return Err(anyhow::anyhow!(
                "failed to get target table list: {:?}",
                error
            ));
        }
    };

    // 4. base 테이블을 기준점으로 삼아서, target 테이블과 비교합니다.
    // A. base에 있는데 target에 없는 것은 보고 대상입니다.
    // B. base에 있고 target에도 있지만, 내용이 다른 것도 보고 대상입니다.
    // C. base에 없고 target에만 있는 것은 보고 대상이 아닙니다. 무시합니다.
    _ = context
        .event_sender
        .send(ProgressEvent::StartComparingTable);

    let report = steps::difference_check(&context, base_table_map, target_table_map);

    // 5. 보고서를 파일로 생성합니다.
    _ = context.event_sender.send(ProgressEvent::SavingReportFile);

    let current_date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let report_file_name = format!("report_{}.json", current_date);

    let report_json = serde_json::to_string_pretty(&report).unwrap();

    std::fs::write(&report_file_name, &report_json).unwrap();

    _ = context.event_sender.send(ProgressEvent::Finished);

    Ok(())
}
