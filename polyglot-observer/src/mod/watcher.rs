use regex::Regex;
use linemux::MuxedLines;
use tokio::sync::mpsc;

pub struct LogWatcher {
    base_path: String,
    tx: mpsc::Sender<(String, String)>,
}

impl LogWatcher {
    pub fn new(base_path: String, tx: mpsc::Sender<(String, String)>) -> Self {
        Self { base_path, tx }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut lines = MuxedLines::new()?;

        // Construct the pattern to find all .log files recursively
        let pattern = format!("{}/**/*.log", self.base_path);
        println!("Discovering logs with pattern: {}", pattern);

        for entry in glob::glob(&pattern)? {
            match entry {
                Ok(path) => {
                    println!("Watching file: {:?}", path);
                    lines.add_file(&path).await?;
                }
                Err(e) => eprintln!("Error in glob discovery: {:?}", e),
            }
        }

        while let Ok(Some(line)) = lines.next_line().await {
            let path = line.source().to_string_lossy();
            let service_name = Self::extract_service_name(&path);
            let content = line.line().to_string();
            
            let _ = self.tx.send((service_name, content)).await;
        }
        
        Ok(())
    }

    fn extract_service_name(path: &str) -> String {
        // Expected format: namespace_service-name_id (works with relative or absolute paths)
        let re = Regex::new(r"[^/_]+_([^_/]+)_[^/_/]+").unwrap();
        if let Some(caps) = re.captures(path) {
            caps.get(1).map_or("unknown".to_string(), |m| m.as_str().to_string())
        } else {
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_service_name() {
        let path = "/var/log/pods/default_auth-service_12345/0.log";
        assert_eq!(LogWatcher::extract_service_name(path), "auth-service");

        let path2 = "/var/log/pods/kube-system_coredns_54321/0.log";
        assert_eq!(LogWatcher::extract_service_name(path2), "coredns");
        
        let invalid_path = "/var/log/messages";
        assert_eq!(LogWatcher::extract_service_name(invalid_path), "unknown");
    }
}
