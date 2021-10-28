/// It waits for a `Ctrl-C` incoming signal.
pub async fn wait_for_ctrl_c() {
    tracing::debug!("server waiting for incoming Ctrl+C signals");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install the Ctrl+C signal handler");
    tracing::debug!("server caught an incoming Ctrl+C signal, starting graceful shutdown");
}
