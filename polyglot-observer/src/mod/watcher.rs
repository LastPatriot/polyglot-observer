use regex::Regex;
use linemux::MuxedLines;
use tokio::sync::mpsc;
use std::collections::HashSet;
use tokio::time::{sleep, Duration, timeout};

pub struct LogWatcher {
    base_path: String,
    tx: mpsc::Sender<(String, String, String, String)>, // (namespace, pod, container, content)
    exclude_namespaces: Vec<String>,
}

impl LogWatcher {
    pub fn new(base_path: String, tx: mpsc::Sender<(String, String, String, String)>, exclude_namespaces: Option<String>) -> Self {
        let exclude = exclude_namespaces
            .map(|s| s.split(',').map(|n| n.trim().to_string()).collect())
            .unwrap_or_default();

        Self { base_path, tx, exclude_namespaces: exclude }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut lines = MuxedLines::new()?;
        let mut watched_files = HashSet::new();
        let pattern = format!("{}/**/*.log", self.base_path);

        println!("Auto-discovery enabled. Monitoring pattern: {}", pattern);

        loop {
            // 1. Check for new files
            for entry in glob::glob(&pattern)? {
                if let Ok(path) = entry {
                    let path_str = path.to_string_lossy().to_string();
                    if !watched_files.contains(&path_str) {
                        println!("New log discovered: {}", path_str);
                        if let Err(e) = lines.add_file(&path).await {
                            eprintln!("Error adding file {}: {}", path_str, e);
                        } else {
                            watched_files.insert(path_str);
                        }
                    }
                }
            }

            // 2. Process pending lines for a short burst
            while let Ok(Ok(Some(line))) = timeout(Duration::from_millis(100), lines.next_line()).await {
                let path = line.source().to_string_lossy();
                let (namespace, pod, container) = Self::extract_identity(&path);
                
                if !self.exclude_namespaces.contains(&namespace) {
                    let content = line.line().to_string();
                    let _ = self.tx.send((namespace, pod, container, content)).await;
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    }

    fn extract_identity(path: &str) -> (String, String, String) {
        // Path format: /var/log/pods/<namespace>_<pod-name>_<pod-uuid>/<container-name>/<retry-count>.log
        let re = Regex::new(r"([^_/]+)_([^_/]+_[^_/]+)/([^_/]+)/").unwrap();
        if let Some(caps) = re.captures(path) {
            let ns = caps.get(1).map_or("unknown".to_string(), |m| m.as_str().to_string());
            let pod = caps.get(2).map_or("unknown".to_string(), |m| m.as_str().to_string());
            let container = caps.get(3).map_or("unknown".to_string(), |m| m.as_str().to_string());
            (ns, pod, container)
        } else {
            ("unknown".to_string(), "unknown".to_string(), "unknown".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_identity() {
        let path = "/var/log/pods/default_auth-service_12345/auth-service/0.log";
        let (ns, pod, container) = LogWatcher::extract_identity(path);
        assert_eq!(ns, "default");
        assert_eq!(pod, "auth-service_12345");
        assert_eq!(container, "auth-service");
    }
}
