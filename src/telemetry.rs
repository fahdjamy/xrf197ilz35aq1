use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, registry, EnvFilter};

pub fn tracing_setup() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::from("info"));

    registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}
