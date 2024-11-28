use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, registry, EnvFilter, Layer};

pub fn tracing_setup() -> WorkerGuard {
    // Get the current crate name.
    let crate_name = env!("CARGO_PKG_NAME");

    let console_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::from("info"));

    let file_filter = EnvFilter::from(format!("{crate_name}=info"));

    // Create a file appender for logging to a file
    let file_appender = file_log_dest();
    let (non_blocking, guard) = non_blocking(file_appender);

    let file_log_dest =
        BunyanFormattingLayer::new("xrf1".into(), non_blocking).with_filter(file_filter);

    let stdout_log_dest = fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_target(false)
        .with_line_number(false)
        .compact()
        .with_writer(std::io::stdout)
        .with_filter(console_filter);

    registry()
        .with(file_log_dest) // File logging
        .with(JsonStorageLayer) // Only concerned w/ info storage, it doesn't do any formatting or provide any output.
        .with(stdout_log_dest) // Console logging
        .init();

    // this is returned so as logs get written to the file
    // if it is not returned in main.rs, logs will not be written to the file
    guard
}

fn file_log_dest() -> RollingFileAppender {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // rotate log files once every hour
        .filename_prefix("xrf1.log") // log file names will be prefixed with `xrf1.log`
        .filename_suffix("log") // log file names will be suffixed with `.log`
        .build(".logs") // build an appender that stores log files in `.logs`
        .expect("Failed to build .logs");
    file_appender
}
