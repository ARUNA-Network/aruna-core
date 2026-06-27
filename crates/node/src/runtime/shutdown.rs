//! Listeners and signal handlers for graceful node shutdown.

pub async fn listen_shutdown() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            println!("CTRL+C signal received. Commencing node shutdown...");
        }
        Err(err) => {
            eprintln!("Error listening for CTRL+C signal: {:?}", err);
        }
    }
}
