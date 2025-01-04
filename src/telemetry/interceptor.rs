use crate::common::RequestId;
use std::fmt::Debug;
use tracing::field::Field;
use tracing::span::Attributes;
use tracing::{Event, Id};
use tracing_subscriber::layer::Context;

const REQ_ID: &'static str = "request_id";

// Custom layer for adding request-ids to the logs
#[derive(Debug, Clone)]
pub struct RequestIdLayer;

impl<S> tracing_subscriber::Layer<S> for RequestIdLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(&self,
                   attrs: &Attributes<'_>,
                   id: &Id,
                   ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        // Iterate on given span attributes to check if we have a request_id to assign
        let mut visitor = RequestIdVisitor {
            request_id: "".to_string(),
            found: false,
        };
        attrs.record(&mut visitor);
        // If we find a request_id in the span attributes, assign it
        if visitor.found {
            extensions.insert(RequestId(visitor.request_id));
        }
    }

    fn on_event(&self,
                event: &Event<'_>,
                ctx: Context<'_, S>) {
        // Retrieve the current span's data.
        if let Some(span_ref) = ctx.lookup_current() {
            let extensions = span_ref.extensions();
            // Check if the request ID extension is present.
            if let Some(request_id) = extensions.get::<RequestId>() {
                // Create a new visitor to inject the request ID.
                let mut visitor = RequestIdVisitor {
                    request_id: request_id.0.clone(),
                    found: false,
                };

                // Visit the event fields and add the request ID.
                event.record(&mut visitor);

                // If the request_id field was already present, log a warning
                if visitor.found {
                    // let meta_data = event.metadata();
                    tracing::warn!( "request_id field already present in event");
                }
            }
        }
    }
}

// A visitor to record fields in a tracing event.
struct RequestIdVisitor {
    request_id: String,
    found: bool,
}

impl tracing::field::Visit for RequestIdVisitor {
    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == REQ_ID {
            self.request_id = value.to_string();
            self.found = true;
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == REQ_ID {
            let Ok(request_id) = format!("{:?}", value).parse::<String>();
            self.request_id = request_id;
            self.found = true;
        }
    }
}
