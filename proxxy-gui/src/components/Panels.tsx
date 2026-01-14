import { MousePointer2, Clipboard as ClipboardIcon } from 'lucide-react';

interface BottomPanelProps {
  code: string;
  isRunning: boolean;
}

export const BottomPanel = ({ code, isRunning }: BottomPanelProps) => {
  return (
    <section className="h-[280px] bg-[#0A0E14] border-t border-white/10 flex p-4 gap-4 z-50">
      {/* Code Output Panel */}
      <div className="flex-1 flex flex-col rounded-2xl overflow-hidden border border-white/10 shadow-lg dotted-bg relative">
        <div className="px-6 py-4 flex items-center justify-between border-b border-white/5 bg-black/40 backdrop-blur-md relative z-10">
          <div className="flex items-center gap-4">
            <span className="text-[10px] font-bold text-white tracking-widest uppercase opacity-60">Code Output</span>
            <div className="flex items-center gap-1.5 px-3 py-1 rounded-lg bg-white/5 text-[9px] text-white/40 font-mono">
              TypeScript Engine <MousePointer2 size={10} />
            </div>
          </div>
          <button className="flex items-center gap-2 px-3 py-1 bg-white/5 hover:bg-white/10 rounded-lg text-[10px] font-bold text-white/60 border border-white/5 transition-all active:scale-95">
            <ClipboardIcon size={12} /> Copy
          </button>
        </div>
        <div className="flex-1 p-6 font-mono text-[12px] leading-relaxed overflow-y-auto bg-[#0D0F13]/80 backdrop-blur-sm relative z-0 custom-scrollbar">
          <CodeSnippet code={code} />
        </div>
      </div>

      {/* Execution Logs Panel */}
      <div className="flex-1 flex flex-col rounded-2xl overflow-hidden border border-white/10 shadow-lg dotted-bg relative">
        <div className="px-6 py-4 flex items-center justify-between border-b border-white/5 bg-black/40 backdrop-blur-md relative z-10">
          <div className="flex items-center gap-4">
            <span className="text-[10px] font-bold text-white tracking-widest uppercase opacity-60">Execution Logs</span>
            <span className="text-[9px] text-emerald-400/60 font-medium px-2 py-0.5 rounded bg-emerald-400/5 border border-emerald-400/10">Active</span>
          </div>
          <button className="text-[10px] font-medium text-white/20 hover:text-white transition-colors">Clear</button>
        </div>
        <div className="flex-1 p-6 font-mono text-[11px] overflow-y-auto bg-[#0D0F13]/80 backdrop-blur-sm relative z-0 custom-scrollbar">
          <div className="flex flex-col gap-2">
            <LogEntry time="14:22:49:720" text="Automation engine initialized" />
            <LogEntry time="14:22:50:110" text="Intercepting traffic on port 8080..." highlight />
            <LogEntry time="14:22:52:445" text="Applied 'Auth Filter' to /api/v2/auth" />
            {isRunning && (
              <div className="flex items-center gap-3 text-[#9DCDE8] animate-pulse mt-1 italic text-[10px]">
                <div className="w-1 h-1 bg-[#9DCDE8] rounded-full" />
                Compiling flow logic...
              </div>
            )}
          </div>
        </div>
      </div>
    </section>
  );
};

// --- Helper Components ---

const CodeSnippet = ({ code }: { code: string }) => (
  <div className="flex flex-col gap-1">
    {code.split('\n').map((line, i) => {
      let color = "text-white/40";
      if (line.trim().startsWith('//')) color = "text-purple-400/30";
      else if (line.includes('let') || line.includes('const')) color = "text-emerald-400/60";
      else if (line.includes('function') || line.includes('return')) color = "text-[#9DCDE8]/60";
      return <div key={i} className={color}>{line}</div>;
    })}
  </div>
);

const LogEntry = ({ time, text, highlight }: { time: string, text: string, highlight?: boolean }) => (
  <div className="flex items-start gap-4">
    <span className="text-purple-400/40 shrink-0 text-[10px]">{`[${time}]`}</span>
    <span className={highlight ? 'text-[#9DCDE8]/60' : 'text-white/30'}>{text}</span>
  </div>
);