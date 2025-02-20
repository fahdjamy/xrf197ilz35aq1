use std::fmt;
use std::fmt::Display;
use uuid::Uuid;

pub const XRF_ENV_KEY: &str = "XRF_ENV";
pub const REQUEST_ID_KEY: &str = "request-id";
pub const KEY_PEM_PATH: &str = "XRF_PERM-SERVER-CERT-PATH";
pub const CERT_PEM_PATH: &str = "XRF_PERM-SERVER-CERT-PATH";

pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

// A simple extension to store the request ID in a span.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
