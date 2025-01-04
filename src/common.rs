use uuid::Uuid;

pub const REQUEST_ID_KEY: &str = "request-id";

pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}
