use xrf1::configs::load_config;
use xrf1::startup::Application;
use xrf1::telemetry::tracing_setup;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = load_config().expect("Failed to load configurations");
    let _guard = tracing_setup(&config.app.name, config.log.clone());

    let app = Application::build(config).await?;
    app.run().await
}
