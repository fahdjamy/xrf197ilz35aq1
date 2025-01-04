use crate::server::REQUEST_ID_KEY;
use futures::future::BoxFuture;
use std::str::FromStr;
use std::task::{Context, Poll};
use tonic::metadata::MetadataValue;
use tower::{Layer, Service};

// Define a new struct for your response interceptor
#[derive(Debug, Clone, Default)]
pub struct ResponseIdInterceptor;

impl<S> Layer<S> for ResponseIdInterceptor {
    type Service = ResponseIdService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ResponseIdService { inner: service }
    }
}

// Implement the Service trait for your response interceptor
#[derive(Debug, Clone)]
pub struct ResponseIdService<S> {
    inner: S,
}

impl<S, Request, Response> Service<Request> for ResponseIdService<S>
where
    S: Service<Request, Response=tonic::Response<Response>> + Send + Clone + 'static,
    S::Future: Send + 'static,
    Request: Send + Clone + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Convert the request and store it in a variable
            // let mut tonic_request = req.clone().into_request();
            //
            // // Get the metadata from the stored request
            // let metadata = tonic_request.metadata_mut();
            //
            // // Get the request ID from the metadata
            // let req_id = metadata.get(REQUEST_ID_KEY)
            //     .and_then(|v| v.to_str().ok())
            //     .unwrap_or("unknown");

            // Call the inner service
            let mut response = inner.call(req).await?;

            // Add the request ID to the response metadata
            response.metadata_mut().insert(
                REQUEST_ID_KEY,
                MetadataValue::from_str("req_id").unwrap(),
            );

            Ok(response)
        })
    }
}
