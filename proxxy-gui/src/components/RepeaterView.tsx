
import { useState, useRef, useEffect } from 'react';
import {
  Send, History, Terminal, Search,
  Server, Globe, Wifi, ChevronDown, Check
} from 'lucide-react';

// Mock Data
const MOCK_AGENTS = [
  { id: 'local', name: 'Localhost (You)', type: 'local', region: 'Local', status: 'online', ping: '0ms' },
  { id: 'aws-fr', name: 'AWS Worker 01', type: 'cloud', region: 'Frankfurt', status: 'online', ping: '45ms' },
  { id: 'do-nyc', name: 'DigitalOcean Proxy', type: 'cloud', region: 'New York', status: 'offline', ping: '-' },
];

export const RepeaterView = () => {
  // State
  const [requestRaw, setRequestRaw] = useState(`GET /api/v1/user/profile HTTP/1.1\nHost: api.proxxy.dev\nAuthorization: Bearer {{token}}\nAccept: application/json\nContent-Type: application/json\n\n{ "query": "current_user" }`);
  const [responseRaw, setResponseRaw] = useState(``);
  const [isSending, setIsSending] = useState(false);
  const [reqSearch, setReqSearch] = useState('');
  const [resSearch, setResSearch] = useState('');

  // Agent Selection State
  const [selectedAgent, setSelectedAgent] = useState(MOCK_AGENTS[0]);
  const [isAgentMenuOpen, setIsAgentMenuOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsAgentMenuOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [dropdownRef]);

  const handleSend = () => {
    if (selectedAgent.status === 'offline') {
      alert("Bu agent şu an offline!");
      return;
    }

    setIsSending(true);

    setTimeout(() => {
      setResponseRaw(`HTTP/1.1 200 OK\nVia: ${selectedAgent.name}\nServer: ProxxyEngine/2.0\nDate: ${new Date().toUTCString()}\n\n{ "id": "USR-9921", "msg": "Hello from ${selectedAgent.region}!" }`);
      setIsSending(false);
    }, 600);
  };

  return (
    <div className="flex flex-col h-full bg-[#0A0E14] text-white/80 font-sans selection:bg-[#9DCDE8]/30">

      {/* --- HEADER --- */}
      <div className="h-14 border-b border-white/10 flex items-center justify-between px-6 bg-[#111318]">

        <div className="flex items-center gap-6">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-[#9DCDE8]/10 flex items-center justify-center border border-[#9DCDE8]/20">
              <Terminal size={16} className="text-[#9DCDE8]" />
            </div>
            <h2 className="text-sm font-bold text-white uppercase tracking-wider">Repeater #1</h2>
          </div>

          <div className="h-4 w-px bg-white/10" />

          <div className="relative" ref={dropdownRef}>
            <button
              onClick={() => setIsAgentMenuOpen(!isAgentMenuOpen)}
              className="flex items-center gap-2 px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 transition-colors border border-white/5 hover:border-white/10 group"
            >
              {selectedAgent.type === 'local' ? <Server size={14} className="text-emerald-400" /> : <Globe size={14} className="text-blue-400" />}
              <div className="flex flex-col items-start leading-none">
                <span className="text-[10px] text-white/40 font-bold uppercase tracking-wider">Egress Node</span>
                <span className="text-[12px] font-medium text-white group-hover:text-[#9DCDE8] transition-colors">{selectedAgent.name}</span>
              </div>
              <ChevronDown size={12} className={`text-white/40 ml-2 transition-transform ${isAgentMenuOpen ? 'rotate-180' : ''}`} />
            </button>

            {isAgentMenuOpen && (
              <div className="absolute top-full left-0 mt-2 w-64 bg-[#1A1D24] border border-white/10 rounded-lg shadow-2xl z-50 overflow-hidden ring-1 ring-black/50">
                <div className="px-3 py-2 bg-black/20 text-[10px] font-bold text-white/40 uppercase tracking-wider border-b border-white/5">
                  Available Agents
                </div>
                {MOCK_AGENTS.map((agent) => (
                  <button
                    key={agent.id}
                    onClick={() => { setSelectedAgent(agent); setIsAgentMenuOpen(false); }}
                    disabled={agent.status === 'offline'}
                    className={`w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors border-b border-white/5 last:border-0
                      ${selectedAgent.id === agent.id ? 'bg-[#9DCDE8]/10' : 'hover:bg-white/5'}
                      ${agent.status === 'offline' ? 'opacity-50 cursor-not-allowed' : ''}
                    `}
                  >
                    <div className={`w-2 h-2 rounded-full ${agent.status === 'online' ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]' : 'bg-red-500'}`} />
                    <div className="flex-1">
                      <div className="text-[12px] font-medium text-white flex items-center gap-2">
                        {agent.name}
                        {selectedAgent.id === agent.id && <Check size={12} className="text-[#9DCDE8]" />}
                      </div>
                      <div className="flex items-center gap-2 mt-0.5">
                        <span className="text-[10px] text-white/40 uppercase">{agent.region}</span>
                        {agent.status === 'online' && (
                          <span className="flex items-center gap-1 text-[10px] text-emerald-400/80 bg-emerald-400/10 px-1 rounded">
                            <Wifi size={8} /> {agent.ping}
                          </span>
                        )}
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button className="flex items-center gap-2 px-4 py-2 rounded-lg bg-white/5 text-[11px] font-bold text-white/60 hover:text-white hover:bg-white/10 transition-all border border-transparent hover:border-white/10">
            <History size={14} /> History
          </button>

          <button
            onClick={handleSend}
            disabled={isSending || selectedAgent.status === 'offline'}
            className="group flex items-center gap-2 px-6 py-2 rounded-lg bg-[#9DCDE8] text-[#0A0E14] text-[11px] font-bold hover:bg-white transition-all shadow-[0_0_20px_rgba(157,205,232,0.1)] hover:shadow-[0_0_20px_rgba(255,255,255,0.3)] disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSending ? (
              <span className="flex items-center gap-2">Sending...</span>
            ) : (
              <>
                Send Request <Send size={14} className="group-hover:translate-x-0.5 transition-transform" />
              </>
            )}
          </button>
        </div>
      </div>

      <div className="flex-1 grid grid-cols-2 gap-px bg-white/5 overflow-hidden">

        <div className="flex flex-col bg-[#0D0F13]">
          <div className="px-4 py-2 border-b border-white/5 flex items-center justify-between bg-white/[0.02]">
            <div className="flex items-center gap-3 flex-1">
              <span className="text-[10px] font-bold uppercase tracking-widest text-[#9DCDE8]">Request</span>
              <div className="relative group max-w-[200px]">
                <Search size={10} className="absolute left-2 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-[#9DCDE8]" />
                <input
                  type="text"
                  placeholder="Find in request..."
                  value={reqSearch}
                  onChange={(e) => setReqSearch(e.target.value)}
                  className="w-full bg-black/20 border border-white/5 rounded px-6 py-1 text-[10px] text-white/60 focus:outline-none focus:border-[#9DCDE8]/30 focus:text-white font-mono transition-colors"
                />
              </div>
            </div>
          </div>
          <textarea
            value={requestRaw}
            onChange={(e) => setRequestRaw(e.target.value)}
            className="flex-1 bg-transparent p-6 font-mono text-sm text-white/80 outline-none resize-none leading-relaxed placeholder:text-white/10"
            spellCheck={false}
          />
        </div>

        <div className="flex flex-col bg-[#080A0E]">
          <div className="px-4 py-2 border-b border-white/5 flex items-center justify-between bg-white/[0.02]">
            <div className="flex items-center gap-3 flex-1">
              <span className="text-[10px] font-bold uppercase tracking-widest text-emerald-400">Response</span>
              <div className="relative group max-w-[200px]">
                <Search size={10} className="absolute left-2 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-emerald-400" />
                <input
                  type="text"
                  placeholder="Find in response..."
                  value={resSearch}
                  onChange={(e) => setResSearch(e.target.value)}
                  className="w-full bg-black/20 border border-white/5 rounded px-6 py-1 text-[10px] text-white/60 focus:outline-none focus:border-emerald-500/30 focus:text-white font-mono transition-colors"
                />
              </div>
            </div>
            {responseRaw && (
              <div className="flex gap-3 items-center text-[10px] font-mono">
                <span className="text-emerald-400 bg-emerald-400/10 px-2 py-0.5 rounded border border-emerald-400/20">200 OK</span>
                <span className="text-white/40">245ms</span>
                <span className="text-white/40">1.2 KB</span>
              </div>
            )}
          </div>
          {responseRaw ? (
            <div className="flex-1 overflow-auto">
              <pre className="p-6 font-mono text-sm leading-relaxed text-white/80">
                {responseRaw.split('\n').map((line, i) => {
                  const isMatch = resSearch && line.toLowerCase().includes(resSearch.toLowerCase());
                  return (
                    <div key={i} className={`px-1 rounded-sm ${isMatch ? 'bg-[#9DCDE8]/20 text-[#9DCDE8] font-bold' : ''}`}>
                      {line}
                    </div>
                  );
                })}
              </pre>
            </div>
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center opacity-20 gap-4 select-none">
              <div className="w-16 h-16 rounded-2xl bg-white/10 flex items-center justify-center">
                <Terminal size={32} />
              </div>
              <div className="text-center">
                <p className="text-xs font-bold uppercase tracking-widest mb-1">Waiting for Response</p>
                <p className="text-[10px] text-white/50">Select an agent and hit send to inspect</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};