use crate::config::AppConfig;
use crate::r#mod::localizer::LingoLocalizer;
use crate::r#mod::exporter::LokiExporter;
use crate::traits::{Localizer, Exporter};

pub struct Bootstrapper {
    pub localizer: Box<dyn Localizer + Send + Sync>,
    pub exporter: Box<dyn Exporter + Send + Sync>,
    pub base_log_path: String,
}

impl Bootstrapper {
    pub fn new(config: &AppConfig) -> Self {
        let localizer = Box::new(LingoLocalizer::new(
            config.lingo_api_url.clone(),
            config.lingo_api_key.clone(),
            config.target_language.clone()
        ));
        let exporter = Box::new(LokiExporter::new(
            config.loki_url.clone(),
            config.target_language.clone()
        ));
        
        Self {
            localizer,
            exporter,
            base_log_path: config.base_log_path.clone(),
        }
    }
}
