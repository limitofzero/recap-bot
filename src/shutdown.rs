use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;

pub fn get_shutdown_token() -> CancellationToken {
    let shutdown_token = CancellationToken::new();

    let cloned = shutdown_token.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        cloned.cancel();
    });

    shutdown_token
}

async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    let mut siging = signal(SignalKind::interrupt()).expect("install SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => log::info!("got SIGTERM"),
        _ = siging.recv() => log::info!("got SIGINT"),
    }
}
