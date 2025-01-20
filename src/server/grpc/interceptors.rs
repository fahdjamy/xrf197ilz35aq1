macro_rules! trace_request {
    ($req:expr, $span_name:expr) => {
        // Extract request ID from metadata
        let request_id = $req.metadata().get(REQUEST_ID_KEY)
            .and_then(|id| id.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Create span for this method, linked to the gRPC span by the request ID
        let span = info_span!($span_name, request_id = request_id);
        let _enter = span.enter();
    };
}

pub(crate) use trace_request;
