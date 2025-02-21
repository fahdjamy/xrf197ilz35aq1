use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use tracing::error;
use xrf1::configs::load_config;
use xrf1::startup::Application;
use xrf1::telemetry::tracing_setup;

use xrf1::context::AppContext;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().expect("Failed to load configurations");
    let _guard = tracing_setup(&config.app.name, config.log.clone());
    let _ = AppContext::get_or_load().map_err(|err| {
        error!("Failed to load application context :: err={}", err);
        return err
    })?;

    let app = Application::build(config)
        .await
        .expect("Failed to build application");

    // start the servers
    // these tasks are currently running concurrently (read NOTE)
    let api_server_task = tokio::spawn(app.http_server.run_until_stopped());
    let grpc_server_task = tokio::spawn(app.grpc_server.run_until_stopped());

    // tokio::select! returns as soon as one of the two tasks completes or errors out
    // There's a pitfall to be mindful of when using tokio::select! - all selected Futures are
    // polled as a single task. This has consequences, as tokio’s documentation highlights:

    // **NOTE**
    // “By running all async expressions on the current task, the expressions are able to run
    // concurrently but not in parallel. This means all expressions are run on the same thread and
    // if one branch blocks the thread, all other expressions will be unable to continue.
    // If parallelism is required, spawn each async expression using tokio::spawn and pass the join
    // handle to select!.”

    tokio::select! {
        outcome = api_server_task => report_exit("api-worker", outcome),
        outcome = grpc_server_task =>  report_exit("gRPC-worker", outcome),
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
