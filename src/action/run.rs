mod background;
mod tui;

use std::sync::mpsc::{channel, Receiver, Sender};
use tui::ProgressEvent;

use crate::{
    command::run::CommandFlags,
    config::{Config, DatabasePair},
    platform_specific::get_config,
};

pub struct ReceiverContext {
    event_receiver: Receiver<ProgressEvent>,
}

pub struct SenderContext {
    event_sender: Sender<ProgressEvent>,
    config: Config,
    database_pair: DatabasePair,
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
        println!("database connection pair is not set. try to [magiceye init] first.");
        return;
    };

    let (sender, receiver) = channel();

    // 2. 실질적인 작업은 백그라운드 스레드로 작업합니다.
    tokio::spawn(async move {
        if let Err(error) = background::generate_report(SenderContext {
            event_sender: sender.clone(),
            config: config.clone(),
            database_pair: database_pair.clone(),
        })
        .await
        {
            _ = sender.send(ProgressEvent::Error(format!("{error:?}")));
        }
    });

    // 3. 진행상황은 TUI 기반으로 확인할 수 있게 합니다.
    if let Err(error) = tui::run_progress_view(ReceiverContext {
        event_receiver: receiver,
    }) {
        println!("failed to run progress view: {:?}", error);
    }
}
