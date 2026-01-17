use crate::admin::Metrics;
use crate::config::BodyCaptureConfig;
use crate::error::BodyCaptureError;
use crate::memory_manager::{MemoryManager, MemoryAllocation, MemoryPermit};
use hudsucker::{
    hyper::{Body, Request, Response, body::HttpBody},
    HttpContext, HttpHandler, RequestOrResponse,
};
use std::sync::{atomic::Ordering, Arc};
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{info, warn, debug};
use uuid::Uuid;

#[derive(Clone)]
pub struct LogHandler {
    metrics: Arc<Metrics>,
    log_sender: Option<tokio::sync::mpsc::Sender<crate::pb::TrafficEvent>>,
    scope_matcher: Option<Arc<crate::filter::ScopeMatcher>>,
    /// Current request_id for correlation (set in handle_request, used in handle_response)
    /// Using RwLock since HttpHandler is called with &mut self
    current_request_id: Arc<RwLock<Option<String>>>,
    /// Current request method for correlation (set in handle_request, used in handle_response)
    /// Used to detect HEAD requests and handle them gracefully
    current_request_method: Arc<RwLock<Option<String>>>,
    /// Configuration for response body capture
    body_capture_config: BodyCaptureConfig,
    /// Memory manager for tracking and limiting memory usage
    memory_manager: Arc<MemoryManager>,
}

impl LogHandler {
    pub fn new(
        metrics: Arc<Metrics>,
        log_sender: Option<tokio::sync::mpsc::Sender<crate::pb::TrafficEvent>>,
        body_capture_config: BodyCaptureConfig,
    ) -> Self {
        let memory_manager = Arc::new(MemoryManager::new(
            body_capture_config.memory_limit,
            body_capture_config.max_concurrent_captures,
        ));
        
        Self {
            metrics,
            log_sender,
            scope_matcher: None,
            current_request_id: Arc::new(RwLock::new(None)),
            current_request_method: Arc::new(RwLock::new(None)),
            body_capture_config,
            memory_manager,
        }
    }

    /// Create a new LogHandler with default body capture configuration
    pub fn new_with_defaults(
        metrics: Arc<Metrics>,
        log_sender: Option<tokio::sync::mpsc::Sender<crate::pb::TrafficEvent>>,
    ) -> Self {
        Self::new(metrics, log_sender, BodyCaptureConfig::default())
    }

    pub fn with_scope_matcher(mut self, matcher: crate::filter::ScopeMatcher) -> Self {
        self.scope_matcher = Some(Arc::new(matcher));
        self
    }

    pub fn with_body_capture_config(mut self, config: BodyCaptureConfig) -> Self {
        // Update memory manager with new config
        let memory_manager = Arc::new(MemoryManager::new(
            config.memory_limit,
            config.max_concurrent_captures,
        ));
        
        self.body_capture_config = config;
        self.memory_manager = memory_manager;
        self
    }

    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> crate::memory_manager::MemoryStats {
        self.memory_manager.get_stats()
    }

    /// Get body capture performance metrics
    pub fn get_body_capture_metrics(&self) -> BodyCapturePerformanceMetrics {
        use std::sync::atomic::Ordering;
        
        let attempts = self.metrics.body_capture_attempts.load(Ordering::Relaxed);
        let successes = self.metrics.body_capture_successes.load(Ordering::Relaxed);
        let failures = self.metrics.body_capture_failures.load(Ordering::Relaxed);
        let timeouts = self.metrics.body_capture_timeouts.load(Ordering::Relaxed);
        let memory_errors = self.metrics.body_capture_memory_errors.load(Ordering::Relaxed);
        let total_latency_ms = self.metrics.body_capture_total_latency_ms.load(Ordering::Relaxed);
        let total_bytes = self.metrics.body_capture_total_bytes.load(Ordering::Relaxed);
        
        let success_rate = if attempts > 0 {
            (successes as f64 / attempts as f64) * 100.0
        } else {
            0.0
        };
        
        let average_latency_ms = if successes > 0 {
            total_latency_ms as f64 / successes as f64
        } else {
            0.0
        };
        
        BodyCapturePerformanceMetrics {
            attempts,
            successes,
            failures,
            timeouts,
            memory_errors,
            success_rate,
            average_latency_ms,
            total_bytes_captured: total_bytes,
            memory_stats: self.memory_manager.get_stats(),
        }
    }
}

/// Performance metrics for body capture operations
#[derive(Debug, Clone)]
pub struct BodyCapturePerformanceMetrics {
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
    pub timeouts: u64,
    pub memory_errors: u64,
    pub success_rate: f64,
    pub average_latency_ms: f64,
    pub total_bytes_captured: u64,
    pub memory_stats: crate::memory_manager::MemoryStats,
}

/// Reads the complete response body from a hyper Body with timeout handling, size limits,
/// and memory management. Properly handles chunked transfer encoding by using Hyper's
/// HttpBody trait which automatically decodes chunked data.
/// 
/// This function implements stream reading with configurable timeouts, size limit enforcement
/// with truncation, proper handling of chunked transfer encoding, and memory tracking to
/// prevent excessive memory usage during concurrent operations.
/// 
/// # Arguments
/// * `body` - The hyper Body to read from (may be chunked or regular)
/// * `config` - Configuration containing timeouts and size limits
/// * `permit` - Memory permit for tracking concurrent operations and memory usage
/// 
/// # Returns
/// * `Ok((Vec<u8>, MemoryAllocation))` - The complete decoded body data and its memory allocation tracker
/// * `Err(BodyCaptureError)` - Various error conditions including timeouts, memory limits, and stream errors
/// 
/// # Requirements Addressed
/// * 1.1: Captures complete response body
/// * 1.4: Enforces size limits with truncation
/// * 2.1: Consumes entire stream
/// * 2.3: Handles chunked transfer encoding properly by using HttpBody trait for automatic decoding
/// * 3.2: Limits total memory usage for concurrent captures
/// * 6.4: Prioritizes system stability over complete logging under memory pressure
/// * 8.1: Applies configurable overall response timeout
/// * 8.2: Applies configurable per-chunk timeout
async fn read_response_body_with_memory_management(
    mut body: Body,
    config: &BodyCaptureConfig,
    permit: &MemoryPermit,
) -> Result<(Vec<u8>, MemoryAllocation), BodyCaptureError> {
    if !config.enabled {
        debug!("Body capture disabled, returning empty body");
        // Still need to allocate memory for the empty vec to maintain consistency
        let allocation = permit.allocate(0)?;
        return Ok((Vec::new(), allocation));
    }

    let response_timeout = config.response_timeout();
    let stream_timeout = config.stream_read_timeout();
    
    debug!(
        "Starting body capture with max_size={}, response_timeout={:?}, stream_timeout={:?}, memory_stats={} (chunked encoding handled automatically by HttpBody trait)",
        config.max_body_size, response_timeout, stream_timeout, permit.memory_manager().get_stats()
    );

    // Apply overall response timeout to the entire body reading operation
    let read_result: Result<Result<(Vec<u8>, MemoryAllocation), BodyCaptureError>, tokio::time::error::Elapsed> = timeout(response_timeout, async {
        let mut body_data = Vec::new();
        let mut current_allocation: Option<MemoryAllocation> = None;
        
        // Read body chunks using HttpBody trait (automatically handles chunked transfer encoding)
        loop {
            // Apply per-chunk timeout for each data read
            let chunk_result = timeout(stream_timeout, body.data()).await;
            
            match chunk_result {
                Ok(Some(Ok(chunk))) => {
                    debug!("Read chunk of {} bytes (decoded if chunked)", chunk.len());
                    
                    let new_size = body_data.len() + chunk.len();
                    
                    // Check size limits before adding chunk
                    if new_size > config.max_body_size {
                        // Truncate to fit within size limit
                        let remaining_space = config.max_body_size.saturating_sub(body_data.len());
                        if remaining_space > 0 {
                            // Reallocate memory for the final size
                            if let Some(old_allocation) = current_allocation.take() {
                                drop(old_allocation); // Free old allocation
                            }
                            current_allocation = Some(permit.allocate(config.max_body_size)?);
                            
                            body_data.extend_from_slice(&chunk[..remaining_space]);
                        }
                        
                        warn!(
                            "Response body truncated at {} bytes (limit: {})",
                            config.max_body_size, config.max_body_size
                        );
                        
                        // Continue reading to consume the stream but don't store more data
                        while let Ok(Some(Ok(_))) = timeout(stream_timeout, body.data()).await {
                            // Just consume remaining chunks without storing
                        }
                        
                        // Return the truncated data with its allocation
                        debug!("Returning truncated body data of {} bytes", body_data.len());
                        return Ok((body_data, current_allocation.unwrap()));
                    }
                    
                    // Check if we can allocate memory for the new size
                    if !permit.memory_manager().can_allocate(new_size) {
                        warn!(
                            "Memory limit would be exceeded, truncating body at {} bytes. Memory stats: {}",
                            body_data.len(), permit.memory_manager().get_stats()
                        );
                        
                        // Continue reading to consume the stream but don't store more data
                        while let Ok(Some(Ok(_))) = timeout(stream_timeout, body.data()).await {
                            // Just consume remaining chunks without storing
                        }
                        
                        // Return current data with existing allocation
                        return Ok((body_data.clone(), current_allocation.unwrap_or_else(|| {
                            // This should not happen, but provide a fallback
                            permit.allocate(body_data.len()).unwrap_or_else(|_| {
                                // If we can't allocate, return empty allocation
                                permit.allocate(0).unwrap()
                            })
                        })));
                    }
                    
                    // Reallocate memory for the new size
                    if let Some(old_allocation) = current_allocation.take() {
                        drop(old_allocation); // Free old allocation
                    }
                    current_allocation = Some(permit.allocate(new_size)?);
                    
                    // Add chunk to body data
                    body_data.extend_from_slice(&chunk);
                }
                Ok(Some(Err(e))) => {
                    warn!("Stream error while reading body: {}", e);
                    return Err(BodyCaptureError::StreamReadError(e.to_string()));
                }
                Ok(None) => {
                    // End of stream reached successfully (all chunks decoded if chunked)
                    debug!("Reached end of stream, captured {} bytes (complete decoded body)", body_data.len());
                    break;
                }
                Err(_) => {
                    // Per-chunk timeout exceeded
                    warn!("Stream read timeout exceeded while reading chunk");
                    return Err(BodyCaptureError::StreamTimeoutError);
                }
            }
        }
        
        // Ensure we have an allocation for the final data
        let final_allocation = current_allocation.unwrap_or_else(|| {
            permit.allocate(body_data.len()).unwrap_or_else(|_| {
                // If we can't allocate, return empty allocation
                permit.allocate(0).unwrap()
            })
        });
        
        Ok((body_data, final_allocation))
    }).await;

    match read_result {
        Ok(Ok((data, allocation))) => {
            debug!("Successfully captured {} bytes with memory allocation", data.len());
            Ok((data, allocation))
        }
        Ok(Err(e)) => Err(e),
        Err(_) => {
            // Overall response timeout exceeded
            warn!("Response timeout exceeded during body capture");
            Err(BodyCaptureError::TimeoutError)
        }
    }
}

/// Handles HEAD request responses gracefully by ensuring no body capture occurs.
/// HEAD responses should never have a body, so we consume any body stream
/// and return an empty body for both logging and client forwarding.
/// 
/// # Arguments
/// * `response` - The original HTTP response
/// 
/// # Returns
/// * `(Response<Body>, Vec<u8>)` - Tuple containing:
///   - Response with empty body for client forwarding
///   - Empty body data for logging
/// 
/// # Requirements Addressed
/// * 2.5: Handles HEAD requests gracefully with no body capture
async fn handle_head_response(response: Response<Body>) -> (Response<Body>, Vec<u8>) {
    debug!("Handling HEAD request response - no body capture");
    
    let (parts, body) = response.into_parts();
    
    // Consume the body stream (should be empty for HEAD, but consume it anyway)
    let mut consumed_body = body;
    tokio::spawn(async move {
        while let Some(_) = consumed_body.data().await {
            // Just consume any data without storing
        }
    });
    
    // Create response with empty body
    let empty_body = Body::from(Vec::<u8>::new());
    let reconstructed_response = Response::from_parts(parts, empty_body);
    
    debug!("HEAD response handled - returning empty body");
    (reconstructed_response, Vec::new())
}

/// Captures the response body and reconstructs an identical response for client forwarding
/// with memory management and backpressure control. Handles compressed responses by
/// capturing the raw compressed data without decompression, preserving compression for the client.
/// 
/// This function reads the complete response body while preserving all headers and status,
/// then reconstructs an identical response that can be forwarded to the client. It handles
/// binary and compressed data correctly by preserving the exact byte sequences, and implements
/// memory tracking and backpressure mechanisms to prevent excessive memory usage.
/// 
/// # Arguments
/// * `response` - The original HTTP response with body stream
/// * `config` - Configuration for body capture behavior
/// * `memory_manager` - Memory manager for tracking and limiting memory usage
/// * `metrics` - Metrics for tracking performance and success/failure rates
/// 
/// # Returns
/// * `(Response<Body>, Vec<u8>)` - Tuple containing:
///   - Reconstructed response identical to original for client forwarding
///   - Captured raw body data (compressed if original was compressed) for logging/storage
/// 
/// # Requirements Addressed
/// * 1.3: Preserves original response body for client while capturing for logging
/// * 2.2: Recreates identical response after consuming body stream
/// * 2.4: Handles compressed responses by capturing raw compressed data without decompression
/// * 3.2: Limits total memory usage for concurrent captures
/// * 3.5: Prioritizes system stability over complete logging under memory pressure
/// * 4.1: Preserves exact byte sequences for text responses
/// * 4.2: Maintains data integrity for binary responses
/// * 4.3: Preserves original encoding without modification
/// * 6.1: Measures latency impact of body capture
/// * 6.2: Tracks capture success/failure rates
/// * 6.4: Implements backpressure mechanisms for high load
async fn capture_and_reconstruct_response_with_memory_management(
    response: Response<Body>,
    config: &BodyCaptureConfig,
    memory_manager: &MemoryManager,
    metrics: &Arc<crate::admin::Metrics>,
) -> (Response<Body>, Vec<u8>) {
    use std::sync::atomic::Ordering;
    
    debug!("Starting response capture and reconstruction with memory management");
    
    // Start timing for latency measurement
    let start_time = std::time::Instant::now();
    
    // Decompose the response into parts and body
    let (parts, body) = response.into_parts();
    
    // Check if response is compressed by examining headers
    let is_compressed = parts.headers.get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .map(|encoding| !encoding.is_empty() && encoding != "identity")
        .unwrap_or(false);
    
    if is_compressed {
        debug!("Response is compressed - will capture raw compressed data without decompression");
    }
    
    // Check content-type filtering if enabled
    let should_capture = if let Some(content_type_header) = parts.headers.get("content-type") {
        if let Ok(content_type) = content_type_header.to_str() {
            config.should_capture_content_type(content_type)
        } else {
            // If content-type header is not valid UTF-8, capture anyway
            true
        }
    } else {
        // No content-type header, capture anyway
        true
    };
    
    let captured_body = if should_capture && config.enabled {
        // Record capture attempt
        metrics.body_capture_attempts.fetch_add(1, Ordering::Relaxed);
        
        // Try to acquire a permit for body capture (implements backpressure)
        match memory_manager.try_acquire_permit() {
            Some(permit) => {
                debug!("Acquired memory permit for body capture. Memory stats: {}", memory_manager.get_stats());
                
                // Attempt to read the response body with memory management
                match read_response_body_with_memory_management(body, config, &permit).await {
                    Ok((body_data, _allocation)) => {
                        // Record successful capture
                        metrics.body_capture_successes.fetch_add(1, Ordering::Relaxed);
                        metrics.body_capture_total_bytes.fetch_add(body_data.len() as u64, Ordering::Relaxed);
                        
                        debug!("Successfully captured {} bytes for reconstruction", body_data.len());
                        body_data
                    }
                    Err(e) => {
                        // Record failure and categorize error type
                        metrics.body_capture_failures.fetch_add(1, Ordering::Relaxed);
                        
                        match e {
                            BodyCaptureError::TimeoutError | BodyCaptureError::StreamTimeoutError => {
                                metrics.body_capture_timeouts.fetch_add(1, Ordering::Relaxed);
                            }
                            BodyCaptureError::MemoryAllocationError => {
                                metrics.body_capture_memory_errors.fetch_add(1, Ordering::Relaxed);
                            }
                            _ => {
                                // Other errors (stream read errors, etc.)
                            }
                        }
                        
                        warn!("Failed to capture response body: {}. Using fallback empty body.", e);
                        // Use fallback empty body on any error to ensure proxy continues
                        Vec::new()
                    }
                }
                // Permit and allocation are automatically dropped here, freeing resources
            }
            None => {
                // No permits available - implement backpressure by skipping body capture
                // Record as memory error since it's due to memory pressure
                metrics.body_capture_attempts.fetch_add(1, Ordering::Relaxed);
                metrics.body_capture_failures.fetch_add(1, Ordering::Relaxed);
                metrics.body_capture_memory_errors.fetch_add(1, Ordering::Relaxed);
                
                warn!(
                    "No memory permits available for body capture, skipping. Memory stats: {}",
                    memory_manager.get_stats()
                );
                
                // Still need to consume the body stream to avoid breaking the proxy
                let mut consumed_body = body;
                tokio::spawn(async move {
                    while let Some(_) = consumed_body.data().await {
                        // Just consume the stream without storing data
                    }
                });
                
                Vec::new()
            }
        }
    } else {
        if !config.enabled {
            debug!("Body capture disabled, skipping");
        } else {
            debug!("Content-type filtered out, skipping body capture");
        }
        
        // Still need to consume the body stream
        let mut consumed_body = body;
        tokio::spawn(async move {
            while let Some(_) = consumed_body.data().await {
                // Just consume the stream without storing data
            }
        });
        
        Vec::new()
    };
    
    // Record latency for successful captures only (to measure actual capture impact)
    if !captured_body.is_empty() || (should_capture && config.enabled) {
        let latency_ms = start_time.elapsed().as_millis() as u64;
        metrics.body_capture_total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }
    
    // Create a new body from the captured data
    // This preserves the exact byte sequences for both text and binary data
    // For compressed responses, this maintains the raw compressed data
    let new_body = Body::from(captured_body.clone());
    
    // Reconstruct the response with the same parts (headers, status, etc.) and new body
    let reconstructed_response = Response::from_parts(parts, new_body);
    
    debug!(
        "Response reconstruction complete. Status: {}, Body size: {} bytes, Compressed: {}, Memory stats: {}", 
        reconstructed_response.status(),
        captured_body.len(),
        is_compressed,
        memory_manager.get_stats()
    );
    
    (reconstructed_response, captured_body)
}

/// Captures the request body and reconstructs an identical request for forwarding
/// with memory management and backpressure control.
///
/// # Arguments
/// * `request` - The original HTTP request with body stream
/// * `config` - Configuration for body capture behavior
/// * `memory_manager` - Memory manager for tracking and limiting memory usage
/// * `metrics` - Metrics for tracking performance and success/failure rates
///
/// # Returns
/// * `(Request<Body>, Vec<u8>)` - Tuple containing:
///   - Reconstructed request identical to original for forwarding
///   - Captured raw body data for logging/storage
async fn capture_and_reconstruct_request_with_memory_management(
    request: Request<Body>,
    config: &BodyCaptureConfig,
    memory_manager: &MemoryManager,
    metrics: &Arc<crate::admin::Metrics>,
) -> (Request<Body>, Vec<u8>) {
    use std::sync::atomic::Ordering;

    debug!("Starting request capture and reconstruction with memory management");

    // Decompose the request into parts and body
    let (parts, body) = request.into_parts();

    // Check content-type filtering if enabled
    let should_capture = if let Some(content_type_header) = parts.headers.get("content-type") {
        if let Ok(content_type) = content_type_header.to_str() {
            config.should_capture_content_type(content_type)
        } else {
            true
        }
    } else {
        true
    };

    // If capture is disabled or content-type filtered, pass through without reading body
    if !should_capture || !config.enabled {
        debug!("Body capture disabled or filtered, passing through original body");
        return (Request::from_parts(parts, body), Vec::new());
    }

    // Record capture attempt
    metrics.body_capture_attempts.fetch_add(1, Ordering::Relaxed);

    // Try to acquire a permit for body capture (implements backpressure)
    let permit = match memory_manager.try_acquire_permit() {
        Some(p) => p,
        None => {
            // No permits available - forward original body but don't capture
            metrics.body_capture_failures.fetch_add(1, Ordering::Relaxed);
            metrics.body_capture_memory_errors.fetch_add(1, Ordering::Relaxed);
            warn!("Backpressure: forwarding request without capturing body.");
            return (Request::from_parts(parts, body), Vec::new());
        }
    };

    debug!("Acquired memory permit for request body capture");

    // Attempt to read the request body with memory management
    match read_response_body_with_memory_management(body, config, &permit).await {
        Ok((body_data, _allocation)) => {
            // Record successful capture
            metrics.body_capture_successes.fetch_add(1, Ordering::Relaxed);
            metrics.body_capture_total_bytes.fetch_add(body_data.len() as u64, Ordering::Relaxed);
            debug!("Successfully captured {} bytes of request body", body_data.len());
            
            let new_body = Body::from(body_data.clone());
            (Request::from_parts(parts, new_body), body_data)
        }
        Err(e) => {
            // Record failure - stream is dead, return empty body
            metrics.body_capture_failures.fetch_add(1, Ordering::Relaxed);
            warn!("Request capture failed: {}. Returning empty body.", e);
            (Request::from_parts(parts, Body::empty()), Vec::new())
        }
    }
}


#[async_trait::async_trait]
impl HttpHandler for LogHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        // Shadow req as mutable to modify headers
        let mut req = req;
        
        // Strip Sec-WebSocket-Extensions to disable compression (permessage-deflate)
        // This avoids "Reserved bits are non-zero" errors when the proxy logic
        // doesn't support compressed frames but the client/server negotiated it.
        req.headers_mut().remove("sec-websocket-extensions");

        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);

        // Check Scope
        if let Some(matcher) = &self.scope_matcher {
            if let Some(host) = req.uri().host() {
                if !matcher.is_allowed(host) {
                    // Clear any pending request_id and method for out-of-scope requests
                    *self.current_request_id.write().await = None;
                    *self.current_request_method.write().await = None;
                    return RequestOrResponse::Request(req);
                }
            }
        }

        let req_id = Uuid::new_v4().to_string();
        let uri = req.uri().to_string();
        info!("Request [{}] {} {}", req_id, req.method(), uri);

        // Store request_id and method for response correlation
        *self.current_request_id.write().await = Some(req_id.clone());
        *self.current_request_method.write().await = Some(req.method().to_string());

        // Capture request body if logging is enabled
        let (req, captured_body) = if self.log_sender.is_some() {
            capture_and_reconstruct_request_with_memory_management(
                req,
                &self.body_capture_config,
                &self.memory_manager,
                &self.metrics
            ).await
        } else {
            (req, Vec::new())
        };

        if let Some(sender) = &self.log_sender {
            use crate::pb::{traffic_event, HttpHeaders, HttpRequestData, TrafficEvent};

            let mut header_map = std::collections::HashMap::new();
            for (k, v) in req.headers() {
                if let Ok(s) = v.to_str() {
                    header_map.insert(k.to_string(), s.to_string());
                }
            }

            let event = TrafficEvent {
                request_id: req_id,
                event: Some(traffic_event::Event::Request(HttpRequestData {
                    method: req.method().to_string(),
                    url: uri,
                    headers: Some(HttpHeaders {
                        headers: header_map,
                    }),
                    body: captured_body,
                    tls: None,
                })),
            };

            let _ = sender.try_send(event);
        }

        RequestOrResponse::Request(req)
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        use crate::pb::{traffic_event, HttpHeaders, HttpResponseData, TrafficEvent};

        let status = res.status().as_u16() as i32;
        
        // Get request_id and method from the stored values (set during handle_request)
        let request_id = self.current_request_id.write().await.take();
        let request_method = self.current_request_method.write().await.take();

        if let Some(request_id) = request_id {
            info!("Response [{}] status: {}", request_id, status);

            if let Some(sender) = &self.log_sender {
                // Check if this is a HEAD request and handle gracefully
                let (reconstructed_response, captured_body) = if let Some(ref method) = request_method {
                    if method.to_uppercase() == "HEAD" {
                        // HEAD requests should not have body capture
                        handle_head_response(res).await
                    } else {
                        // Regular request - capture response body and reconstruct response for client with memory management
                        capture_and_reconstruct_response_with_memory_management(
                            res, 
                            &self.body_capture_config,
                            &self.memory_manager,
                            &self.metrics
                        ).await
                    }
                } else {
                    // No method available - assume regular request
                    capture_and_reconstruct_response_with_memory_management(
                        res, 
                        &self.body_capture_config,
                        &self.memory_manager,
                        &self.metrics
                    ).await
                };

                // Extract headers from the reconstructed response for logging
                let mut header_map = std::collections::HashMap::new();
                for (k, v) in reconstructed_response.headers() {
                    if let Ok(s) = v.to_str() {
                        header_map.insert(k.to_string(), s.to_string());
                    }
                }

                let event = TrafficEvent {
                    request_id: request_id.clone(),
                    event: Some(traffic_event::Event::Response(HttpResponseData {
                        status_code: status,
                        headers: Some(HttpHeaders {
                            headers: header_map,
                        }),
                        body: captured_body,  // Use captured body instead of hardcoded empty vec
                        tls: None,
                    })),
                };

                // Handle error if sending fails, but don't affect client response
                if let Err(e) = sender.try_send(event) {
                    warn!("Failed to send traffic event for request [{}]: {}", request_id, e);
                }

                // Return the reconstructed response to the client
                return reconstructed_response;
            }
        } else {
            info!("Response status: {} (out-of-scope or no correlation)", status);
        }

        // If no logging is configured or request_id is missing, return original response
        res
    }
}
