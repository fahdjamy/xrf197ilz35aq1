use crate::helpers::{start_test_app, TestApp};
use futures::FutureExt;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::panic::{self, AssertUnwindSafe};

pub type TestError = Box<dyn Error + Send + Sync>;

/// Sets up a `TestApp` instance, passes a mutable reference to the test function,
/// ensures `TestApp::drop_db` is called afterward (even on panic),
/// and propagates panics or reports test errors.
pub async fn run_test_async<F, Fut, E>(test_fn: F)
where
    F: FnOnce(TestApp) -> Fut + panic::UnwindSafe, // test_fn takes a mut reference to TestApp
    Fut: Future<Output=Result<(), E>>, // The test function returns a Future
    E: Into<TestError>, // The error type must be convertible to a boxed error
{
    let app = start_test_app().await;

    // We need AssertUnwindSafe because we are passing a mutable reference
    // across a potential unwind boundary (`catch_unwind`).
    // We are asserting that our test function `F` maintains safety invariants
    // even if it panics while holding the &mut TestApp. This is generally
    // true for test logic if it doesn't misuse unsafe code.
    let test_future = test_fn(app.clone());
    let result = AssertUnwindSafe(test_future).catch_unwind().await;

    // 3. Teardown: Always call drop_db *after* the test future completes or panics
    //    'app' is still valid here because we only passed a reference to test_fn.
    app.drop_db().await;

    match result {
        Ok(Ok(())) => {
            // Test completed successfully
        }
        Ok(Err(error)) => {
            let boxed_err = error.into();
            panic!("Test failed with error: {}", boxed_err);
        }
        Err(err) => {
            // Test panicked
            println!("Test panicked. Resuming unwind...");
            panic::resume_unwind(err);
        }
    }
}

// THIS WAS THE PLACEHOLDER DEFINITION IN THE EXAMPLE CODE
#[derive(Debug)]
pub struct FormattedError(pub String); // Simple struct holding the formatted error string

impl fmt::Display for FormattedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0) // Display the stored string
    }
}

impl Error for FormattedError {} // Basic Error implementation
