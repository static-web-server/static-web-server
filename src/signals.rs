use ctrlc;
use std::{process, sync::mpsc::channel};

/// It waits for a `Ctrl-C` incoming signal.
pub fn wait_for_ctrl_c() {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("could not send signal on channel"))
        .expect("error setting Ctrl-C handler");

    info!("press Ctrl+C to shutdown the server");

    rx.recv().expect("could not receive signal from channel");

    warn!("Ctrl+C signal caught, shutting down the server execution");

    process::exit(1)
}
