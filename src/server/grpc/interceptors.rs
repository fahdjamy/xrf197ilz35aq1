use crate::common::{generate_request_id, RequestId, REQUEST_ID_KEY};
use anyhow::Result;
use futures::Future;
use std::pin::Pin;
use std::str::FromStr;
use tonic::{metadata::{KeyAndValueRef, MetadataKey, MetadataValue}, Request};
use tower::{Layer, Service};
use tracing::{info, info_span, Instrument, Span};

// Define a struct to represent your interceptor layer
#[derive(Clone, Default)]
pub struct RequestIdInterceptorLayer;

// Implement the `Layer` trait for your `RequestIdInterceptorLayer`
impl<S> Layer<S> for RequestIdInterceptorLayer {
    type Service = RequestIdInterceptor<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestIdInterceptor { inner: service }
    }
}

// Define the interceptor struct
#[derive(Clone)]
struct RequestIdInterceptor<S> {
    inner: S,
}

// Implement the `Service` trait for your `RequestIdInterceptor`
impl<S, B> Service<Request<B>> for RequestIdInterceptor<S>
where
    S: Service<Request<B>>,
    S::Future: Send + 'static,
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

        // Store the request ID in the span's extensions.
        span.in_scope(|| {
            if let Some(span_ref) = Span::current().into() {
                let mut extensions = span_ref.extensions_mut();
                extensions.insert::<RequestId>(RequestId(req_id.clone()));
            }
        });

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

                self.inner.call(req).instrument(span).await
            }
        };

        Box::pin(fut.instrument(span))
    }
}
