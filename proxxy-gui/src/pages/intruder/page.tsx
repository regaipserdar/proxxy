import { Target, Play, Layers } from 'lucide-react';

export const IntruderView = () => {
  return (
    <div className="flex flex-col h-full bg-[#0A0E14]">
      <div className="h-14 border-b border-white/10 flex items-center justify-between px-6 bg-[#111318]">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-amber-500/10 flex items-center justify-center border border-amber-500/20">
            <Target size={16} className="text-amber-500" />
          </div>
          <h2 className="text-sm font-bold text-white uppercase tracking-wider">Intruder Engine</h2>
        </div>
        <button className="flex items-center gap-2 px-6 py-2 rounded-lg bg-amber-500 text-black text-[11px] font-bold hover:bg-white transition-all shadow-[0_0_20px_rgba(245,158,11,0.3)]">
          <Play size={14} fill="currentColor" /> Start Attack
        </button>
      </div>

      <div className="flex-1 grid grid-cols-12 overflow-hidden">
        {/* Config Sidebar */}
        <div className="col-span-3 border-r border-white/10 bg-[#0D0F13] p-8 space-y-8">
          <section className="space-y-4">
            <h3 className="text-[10px] font-bold text-white/20 uppercase tracking-widest">Target Configuration</h3>
            <div className="space-y-3">
              <IntruderField label="Host" value="api.proxxy.dev" />
              <IntruderField label="Port" value="443" />
              <IntruderField label="Protocol" value="HTTPS" />
            </div>
          </section>

          <section className="space-y-4">
            <h3 className="text-[10px] font-bold text-white/20 uppercase tracking-widest">Attack Type</h3>
            <select className="w-full bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-xs text-white/60 focus:outline-none focus:border-amber-500/50 appearance-none">
              <option>Sniper</option>
              <option>Battering Ram</option>
              <option>Pitchfork</option>
              <option>Cluster Bomb</option>
            </select>
          </section>

          <section className="space-y-4">
            <h3 className="text-[10px] font-bold text-white/20 uppercase tracking-widest">Payload Sets</h3>
            <button className="w-full flex items-center justify-center gap-2 py-3 rounded-xl border border-dashed border-white/10 text-white/20 text-xs hover:border-amber-500/50 hover:text-white transition-all">
              <Layers size={14} /> Add Payload Set
            </button>
          </section>
        </div>

        {/* Positions View */}
        <div className="col-span-9 bg-[#080A0E] flex flex-col">
          <div className="px-6 py-4 border-b border-white/5 bg-white/[0.02] flex items-center justify-between">
            <div className="flex items-center gap-4">
              <span className="text-[10px] font-bold text-white/60">Payload Positions</span>
              <div className="px-2 py-0.5 rounded bg-amber-500/10 text-[9px] text-amber-500 border border-amber-500/20 font-mono">0 Markers</div>
            </div>
            <div className="flex gap-2">
              <button className="px-3 py-1.5 rounded-lg bg-white/5 text-[10px] font-bold text-white/40 hover:text-white transition-all">Clear §</button>
              <button className="px-3 py-1.5 rounded-lg bg-amber-500/10 text-[10px] font-bold text-amber-500 border border-amber-500/20 transition-all">Auto §</button>
            </div>
          </div>
          <div className="flex-1 p-8 overflow-auto">
            <pre className="font-mono text-sm text-white/30 leading-relaxed italic">
              {`POST /api/v1/auth/login HTTP/1.1\nHost: api.proxxy.dev\n\n{ "user": "§admin§", "pass": "§password§" }`}
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
};

const IntruderField = ({ label, value }: { label: string, value: string }) => (
  <div className="space-y-1.5">
    <p className="text-[10px] font-medium text-white/30 px-1">{label}</p>
    <div className="bg-black/40 border border-white/5 rounded-xl px-4 py-3 text-xs text-white/80 font-mono tracking-tight">
      {value}
    </div>
  </div>
);
