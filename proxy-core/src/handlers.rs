use crate::admin::Metrics;
use hudsucker::{
    hyper::{Body, Request, Response},
    HttpContext, HttpHandler, RequestOrResponse,
};
use std::sync::{atomic::Ordering, Arc};
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct LogHandler {
    metrics: Arc<Metrics>,
    log_sender: Option<tokio::sync::mpsc::Sender<crate::pb::TrafficEvent>>,
    scope_matcher: Option<Arc<crate::filter::ScopeMatcher>>,
}

impl LogHandler {
    pub fn new(
        metrics: Arc<Metrics>,
        log_sender: Option<tokio::sync::mpsc::Sender<crate::pb::TrafficEvent>>,
    ) -> Self {
        Self {
            metrics,
            log_sender,
            scope_matcher: None,
        }
    }

    pub fn with_scope_matcher(mut self, matcher: crate::filter::ScopeMatcher) -> Self {
        self.scope_matcher = Some(Arc::new(matcher));
        self
    }
}

#[async_trait::async_trait]
impl HttpHandler for LogHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);

        // Check Scope
        if let Some(matcher) = &self.scope_matcher {
            if let Some(host) = req.uri().host() {
                if !matcher.is_allowed(host) {
                    // If not allowed, forward directly without interception/logging?
                    // Story says: "!scope.is_allowed -> forward_directly"
                    // In hudsucker, returning RequestOrResponse::Request(req) IS forwarding.
                    // But we want to avoid LOGGING it.
                    // So we just return here.
                    return RequestOrResponse::Request(req);
                }
            }
        }

        let req_id = Uuid::new_v4().to_string();
        info!("Request [{}] {} {}", req_id, req.method(), req.uri());

        if let Some(sender) = &self.log_sender {
            // Create traffic event
            use crate::pb::{traffic_event, HttpHeaders, HttpRequestData, TrafficEvent};

            // Extract headers
            let mut header_map = std::collections::HashMap::new();
            for (k, v) in req.headers() {
                if let Ok(s) = v.to_str() {
                    header_map.insert(k.to_string(), s.to_string());
                }
            }

            let event = TrafficEvent {
                request_id: req_id.clone(),
                event: Some(traffic_event::Event::Request(HttpRequestData {
                    method: req.method().to_string(),
                    url: req.uri().to_string(),
                    headers: Some(HttpHeaders {
                        headers: header_map,
                    }),
                    body: vec![], // Body capturing is complex, skipping for MVP
                    tls: None,
                })),
            };

            let _ = sender.try_send(event); // Fire and forget
        }

        RequestOrResponse::Request(req)
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        info!("Response status: {}", res.status());
        // For MVP, we are not streaming response logs because we haven't solved correlation yet.
        // We will just log to stdout.
        res
    }
}
