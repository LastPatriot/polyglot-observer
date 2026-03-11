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
    let (tx, mut rx) = mpsc::channel::<(String, String, String, String)>(100);

    // 4. Start Log Watcher (as a background task)
    let mut watcher = LogWatcher::new(bootstrapper.base_log_path.clone(), tx, config.exclude_namespaces.clone());
    
    // For this demonstration, we'll spawn the watcher and process logs.
    tokio::spawn(async move {
        if let Err(e) = watcher.run().await {
            eprintln!("Watcher error: {}", e);
        }
    });

    println!("Observer is now watching logs...");

    // 5. Process log entries
    while let Some((namespace, pod, container, log_line)) = rx.recv().await {
        let localizer = &bootstrapper.localizer;
        let exporter = &bootstrapper.exporter;

        // Localize log entry
        let localized = localizer.localize(&log_line).await;

        if localized.is_empty() {
            continue;
        }

        // 🔍 DEBUG HOOK: Visualize the Observer in action
        println!("\n🚀 [{}/{}/{}] -> {}", namespace, pod, container, localized);

        // Export to Grafana Loki
        exporter.export(&namespace, &pod, &container, &localized).await;
    }

    Ok(())
}
