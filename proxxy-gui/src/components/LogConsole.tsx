
export function LogConsole() {
    // Mock Logs (Gerçekte WebSocket'ten gelebilir)
    const logs = [
        "[INFO] Orchestrator started on port 9090",
        "[INFO] MOCK"
    ];

    return (
        <div className="bg-[#0A0E14] border-t border-white/10 h-32 overflow-y-auto p-2 font-mono text-[10px] text-white/60">
            {logs.map((log, i) => (
                <div key={i} className="border-b border-white/5 pb-0.5 mb-0.5">
                    <span className={log.includes("WARN") ? "text-yellow-500" : "text-emerald-400"}>➜</span> {log}
                </div>
            ))}
        </div>
    );
}
