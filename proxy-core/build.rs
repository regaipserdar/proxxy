fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false) // We only need server stubs in the proxy-core (Wait, proxy-core is shared?)
        // The prompt says: "Amacımız: `proxy-agent` ve `orchestrator` crate'leri bu proto tanımlarını ortak kullanabilsin."
        // So `proxy-core` should probably expose both or just the types?
        // Actually, `proxy-agent` runs the `ProxyService` (User says: "Girdi: TrafficEvent (Agent -> Orchestrator)").
        // Wait. "Agent" sends TrafficEvent. "Orchestrator" sends InterceptCommand.
        // If "Agent -> Orchestrator", then Agent is the Client and Orchestrator is the Server?
        // Let's re-read: "Bu protokol, sahada çalışan bir "Ajan" (Proxy) ile merkezdeki "Orchestrator" (Tauri/DB) arasında veri taşıyacak."
        // Usually Agents connect TO Orchestrator. So Orchestrator is the Server. Agent is the Client.
        // So `proxy-core` should probably build both just in case, or we check where `proxy-core` is used.
        // `proxy-agent` uses `proxy-core`. `orchestrator` uses `proxy-core`.
        // So `proxy-core` needs to have the TYPES. Functional code (Client/Server) depends on who implements what.
        // `tonic-build` generates both by default usually.
        // Let's enable both to be safe.
        .build_client(true)
        .build_server(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["../proto/proxy.proto"], &["../proto"])?;
    Ok(())
}
