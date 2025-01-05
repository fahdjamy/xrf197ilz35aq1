use uuid::Uuid;

pub const REQUEST_ID_KEY: &str = "request-id";

pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

// A simple extension to store the request ID in a span.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);
