use async_trait::async_trait;

#[async_trait]
pub trait Localizer {
    async fn localize(&self, text: &str) -> String;
}

#[async_trait]
pub trait Exporter {
    async fn export(&self, service_name: &str, localized_text: &str);
}
