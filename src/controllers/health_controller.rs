pub struct HealthController;
impl HealthController {
    pub async fn health_check() -> &'static str {
        "healthy"
    }
}
