mod config;
mod traits;
mod startup;
mod r#mod;

use crate::config::AppConfig;
use crate::startup::Bootstrapper;
use crate::r#mod::watcher::LogWatcher;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Universal Node Observer...");

    // 1. Load config
    let config = AppConfig::new()?;
    println!("Config loaded: {:?}", config);

    // 2. Bootstrap dependencies
    let bootstrapper = Bootstrapper::new(&config);

    // 3. Initialize communication channel
    let (tx, mut rx) = mpsc::channel::<(String, String)>(100);

    // 4. Start Log Watcher (as a background task)
    let mut watcher = LogWatcher::new(bootstrapper.base_log_path.clone(), tx);
    
    // For this demonstration, we'll spawn the watcher and process logs.
    tokio::spawn(async move {
        if let Err(e) = watcher.run().await {
            eprintln!("Watcher error: {}", e);
        }
    });

    println!("Observer is now watching logs...");

    // 5. Process log entries
    while let Some((service_name, log_line)) = rx.recv().await {
        println!("DEBUG: Received line for {}: {}", service_name, log_line);
        let localizer = &bootstrapper.localizer;
        let exporter = &bootstrapper.exporter;

        // Localize log entry
        let localized = localizer.localize(&log_line).await;

        // 🔍 DEBUG HOOK: Visualize the Observer in action
        println!("\n🚀 [SERVICE: {}]", service_name);
        println!("   RAW: {}", log_line);
        println!("   LOCALIZED: {}", localized);

        // Export to Grafana Loki
        exporter.export(&service_name, &localized).await;
    }

    Ok(())
}
