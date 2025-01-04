use crate::configs::LogConfig;
use crate::telemetry::interceptor::RequestIdLayer;
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer};

pub fn tracing_setup(app_name: &str, log_config: LogConfig) -> WorkerGuard {
    // Get the current crate name.
    let crate_name = option_env!("CARGO_PKG_NAME")
        .unwrap_or_else(|| app_name);
    let log_level = log_config.level.to_lowercase();

    // // as a deny filter (DEBUG, but remove noisy logs)
    let console_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .expect("Env log level needs to be set to a valid level")
        .add_directive(format!("{crate_name}={log_level}")
            .parse()
            .expect("Failed to parse directive for console log"));

    let file_filter = EnvFilter::from(format!("{crate_name}=info"));

    // Create a file appender for logging to a file
    let file_appender = file_log_dest(log_config);
    let (non_blocking, guard) = non_blocking(file_appender);

    let file_log_dest =
        BunyanFormattingLayer::new(app_name.into(), non_blocking).with_filter(file_filter);

    let stdout_log_dest = fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_target(false)
        .with_line_number(false)
        .compact()
        .with_writer(std::io::stdout)
        .with_filter(console_filter);

    tracing_subscriber::registry()
        .with(file_log_dest) // File logging
        .with(JsonStorageLayer) // Only concerned w/ info storage, it doesn't do any formatting or provide any output.
        .with(stdout_log_dest) // Console logging
        // add the request ID layer.
        .with(RequestIdLayer)
        // Set the registry as the global default subscriber.
        // init Attempts to set self as the global default subscriber in the current scope, panics if this fails
        .init();

    // this is returned so as logs get written to the file
    // if it is not returned in main.rs, logs will not be written to the file
    guard
}

fn file_log_dest(log_config: LogConfig) -> RollingFileAppender {
    let output = log_config.output;
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // rotate log files once every day
        .filename_prefix(log_config.prefix) // log file names will be prefixed with `xrf1`
        .filename_suffix(log_config.suffix) // log file names will be suffixed with `.log`
        .build(output.clone()) // build an appender that stores log files in `.logs`
        .expect(format!("Failed to build {output}").as_str());
    file_appender
}
