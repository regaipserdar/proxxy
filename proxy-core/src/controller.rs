use dashmap::DashMap;
use tokio::sync::oneshot;
use tracing::info;
use crate::pb::InterceptCommand;

use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct InterceptController {
    /// Maps Request ID -> Sender for resume signal
    pending_requests: Arc<DashMap<String, oneshot::Sender<InterceptCommand>>>,
}

impl InterceptController {
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(DashMap::new()),
        }
    }

    /// Pause a request and wait for a decision.
    /// Returns a Receiver that will trigger when a decision is made.
    pub fn register_request(&self, request_id: String) -> oneshot::Receiver<InterceptCommand> {
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(request_id, tx);
        rx
    }

    /// Resume a request with a command (e.g. forward, drop, modify).
    pub fn resume_request(&self, request_id: &str, command: InterceptCommand) -> bool {
        if let Some((_, tx)) = self.pending_requests.remove(request_id) {
            info!("Resuming request {}", request_id);
            let _ = tx.send(command);
            true
        } else {
            false
        }
    }
}
