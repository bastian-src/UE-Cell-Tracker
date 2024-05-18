use anyhow::{anyhow, Result};
use bus::BusReader;
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::logic::{
    check_not_stopped, wait_until_running,
    MessageCellInfo, MessageDci, MessageRnti, WorkerState, WorkerType,
    DEFAULT_WORKER_SLEEP_MS,
};

pub struct CellSinkArgs {
    pub rx_app_state: BusReader<WorkerState>,
    pub tx_sink_state: Sender<WorkerState>,
    pub rx_cell_info: BusReader<MessageCellInfo>,
    pub rx_dci: BusReader<MessageDci>,
    pub rx_rnti: BusReader<MessageRnti>,
}

pub fn deploy_cell_sink(args: CellSinkArgs) -> Result<JoinHandle<()>> {
    let thread = thread::spawn(move || {
        let _ = run(
            args.rx_app_state,
            args.tx_sink_state,
            args.rx_cell_info,
            args.rx_dci,
            args.rx_rnti,
        );
    });
    Ok(thread)
}

fn send_final_state(tx_sink_state: &Sender<WorkerState>) -> Result<()> {
    Ok(tx_sink_state.send(WorkerState::Stopped(WorkerType::CellSink))?)
}

fn wait_for_running(
    rx_app_state: &mut BusReader<WorkerState>,
    tx_sink_state: &Sender<WorkerState>,
) -> Result<()> {
    match wait_until_running(rx_app_state) {
        Ok(_) => Ok(()),
        _ => {
            send_final_state(tx_sink_state)?;
            Err(anyhow!("[sink] Main did not send 'Running' message"))
        }
    }
}

fn run(
    mut rx_app_state: BusReader<WorkerState>,
    tx_sink_state: Sender<WorkerState>,
    _rx_cell_info: BusReader<MessageCellInfo>,
    _rx_dci: BusReader<MessageDci>,
    _rx_rnti: BusReader<MessageRnti>,
) -> Result<()> {
    tx_sink_state.send(WorkerState::Running(WorkerType::CellSink))?;
    wait_for_running(&mut rx_app_state, &tx_sink_state)?;

    loop {
        /* <precheck> */
        thread::sleep(Duration::from_millis(DEFAULT_WORKER_SLEEP_MS));
        if check_not_stopped(&mut rx_app_state).is_err() {
            break;
        }
        /* </precheck> */

        // TODO: Consume rx_dci, rx_cell_info, and rx_rnti
        // TODO: -> Send combined message to some remote
    }

    send_final_state(&tx_sink_state)?;
    Ok(())
}
