use proxy_core::pb::{
    traffic_event, RegisterAgentRequest, TlsDetails, TrafficEvent, WebSocketFrame,
};

#[test]
fn test_register_agent_serialization() {
    let req = RegisterAgentRequest {
        agent_id: "test-agent-1".to_string(),
        hostname: "localhost".to_string(),
        version: "0.1.0".to_string(),
        name: "Test Agent".to_string(),
    };

    // In a real gRPC scenario, tonic handles serialization.
    // Here we just verify we can construct and access fields.
    assert_eq!(req.agent_id, "test-agent-1");
    assert_eq!(req.hostname, "localhost");
}

#[test]
fn test_websocket_frame_construction() {
    let frame = WebSocketFrame {
        payload: vec![1, 2, 3, 4],
        is_binary: true,
        direction_outbound: true,
    };

    let event = TrafficEvent {
        request_id: "req-123".to_string(),
        event: Some(traffic_event::Event::Websocket(frame)),
    };

    match event.event {
        Some(traffic_event::Event::Websocket(f)) => {
            assert_eq!(f.payload, vec![1, 2, 3, 4]);
            assert!(f.is_binary);
            assert!(f.direction_outbound);
        }
        _ => panic!("Expected Websocket event"),
    }
}

#[test]
fn test_tls_details_structure() {
    let tls = TlsDetails {
        version: "TLSv1.3".to_string(),
        cipher: "TLS_AES_256_GCM_SHA384".to_string(),
    };

    assert_eq!(tls.version, "TLSv1.3");
    assert_eq!(tls.cipher, "TLS_AES_256_GCM_SHA384");
}
