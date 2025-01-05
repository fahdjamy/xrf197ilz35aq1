use crate::common::{generate_request_id, RequestId, REQUEST_ID_KEY};
use anyhow::Result;
use futures::Future;
use std::error::Error;
use std::pin::Pin;
use std::str::FromStr;
use tonic::codegen::Bytes;
use tonic::server::NamedService;
use tonic::{metadata::{KeyAndValueRef, MetadataKey, MetadataValue}, Request};
use tower::{Layer, Service};
use tracing::{info, info_span, Instrument};

// Define a struct to represent your interceptor layer
// RequestIdInterceptor (Tower Service):
//
// - Intercepts each incoming request.
// - Generates a unique request_id (UUID).
// - Adds the request_id to the request's metadata under the key x-request-id.
// - Creates a top-level gRPC span with the request_id as a field.
// - Forwards the request to the inner service (which is now RequestSpan).
#[derive(Clone, Default)]
pub struct RequestIdInterceptorLayer;

// Implement the `Layer` trait for your `RequestIdInterceptorLayer`
impl<S> Layer<S> for RequestIdInterceptorLayer
where
    S: Clone,
{
    type Service = RequestIdInterceptor<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestIdInterceptor { inner: service }
    }
}

// Define the interceptor struct
#[derive(Clone)]
pub struct RequestIdInterceptor<S> {
    inner: S,
}

// Implement the `Service` trait for your `RequestIdInterceptor`
impl<S, B> Service<Request<B>> for RequestIdInterceptor<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let req_id = generate_request_id();

        // Insert the request ID into the request's metadata.
        req.metadata_mut()
            .insert(MetadataKey::from_static(REQUEST_ID_KEY), MetadataValue::from_str(&req_id).unwrap());

        // Create a new span for the request with the generated request ID.
        let span = info_span!("gRPC", request_id = req_id);

        // Clone the inner service to be moved into the async block.
        let mut inner = self.inner.clone();

        // Instrument the request handling with the new span.
        let fut = {
            let span = span.clone();
            async move {
                let mut header_string = String::new();

                for key_and_value in req.metadata_mut().iter() {
                    let value_str = match key_and_value {
                        KeyAndValueRef::Ascii(key, val) => {
                            &format!("{}:{:?}", key.as_str(), val)
                        }
                        KeyAndValueRef::Binary(key, val) => {
                            &format!("{}:{:?}", key.as_str(), val)
                        }
                    };

                    header_string.push_str(&format!("{}; ", value_str));
                }

                info!("request-headers: '{}'", header_string);

                inner.call(req).instrument(span).await
            }
        };

        Box::pin(fut.instrument(span))
    }
}

// RequestSpan (Custom Wrapper Service):
//
// - Wraps your actual gRPC service (e.g., AssetServiceServer).
// - Intercepts each request after RequestIdInterceptor.
// - Retrieves the request_id from the request metadata.
// - Creates a new span (e.g., request or the name of the specific method) with the request_id as a
// field. This span automatically becomes a child of the gRPC span because of tracing's context propagation.
// - Calls the inner service's method (e.g., get_paginated_assets) within the context of this new span
#[derive(Clone)]
pub struct RequestSpan<S> {
    inner: S,
}

impl<S> RequestSpan<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> NamedService for RequestSpan<S>
where
    S: NamedService,
{
    const NAME: &'static str = S::NAME;
}

impl<S, B, ResBody> Service<Request<B>> for RequestSpan<S>
where
    B: Send + 'static,
    S: Service<Request<B>, Response=tonic::Response<ResBody>> + Send + Clone + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
    ResBody: http_body::Body<Data=Bytes> + Send + 'static,
    ResBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
{
    type Response = S::Response;
    type Error = S::Error;
    // Pin<Box<...>>: The future is boxed and pinned, which is necessary for async functions to be used in a trait object
    // dyn Future<Output = Result<Self::Response, Self::Error>>: This is the trait object for the future. It returns a Result with the same Response and Error types as the inner service
    // + Send + 'static: The future must be: Send (safe to send between threads) and 'static (no non-static references).
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let request_id = req.metadata()
            .get(REQUEST_ID_KEY)
            .and_then(|id| id.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let span = info_span!("request", request_id = request_id);

        let fut = {
            let span = span.clone();
            let mut inner = self.inner.clone();
            async move {
                inner.call(req).instrument(span.clone()).await
            }
        };

        Box::pin(fut.instrument(span))
    }
}
