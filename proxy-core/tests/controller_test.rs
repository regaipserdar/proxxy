use proxy_core::{pb::InterceptCommand, InterceptController};

#[tokio::test]
async fn test_intercept_pause_resume() {
    let controller = InterceptController::new();
    let req_id = "req-123".to_string();

    // simulate Request Handler pausing
    let rx = controller.register_request(req_id.clone());

    // simulate Orchestrator deciding to resume
    let command = InterceptCommand { command: None }; // mock command
    let resumed = controller.resume_request(&req_id, command);

    assert!(resumed, "Request should be resumable");

    // Verify receiver gets the signal
    let result = rx.await;
    assert!(result.is_ok(), "Receiver should get message");
}

#[tokio::test]
async fn test_resume_non_existent() {
    let controller = InterceptController::new();
    let resumed = controller.resume_request("invalid-id", InterceptCommand { command: None });
    assert!(!resumed, "Should return false for non-existent request");
}
