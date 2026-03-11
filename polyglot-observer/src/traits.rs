use async_trait::async_trait;

#[async_trait]
pub trait Localizer {
    async fn localize(&self, text: &str) -> String;
}

#[async_trait]
pub trait Exporter: Send + Sync {
    async fn export(&self, namespace: &str, pod: &str, container: &str, localized_text: &str);
}
