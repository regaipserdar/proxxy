import { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { ProxxyNodeData } from '@/types';
import { FileText, Layers, Cpu, BrainCircuit } from 'lucide-react';

export const AgentNode = memo(({ data }: NodeProps<ProxxyNodeData>) => {
  return (
    <div className="relative group">
      {/* Outer Glow */}
      <div className="absolute -inset-2 bg-sky-500/10 blur-2xl rounded-full group-hover:bg-sky-500/20 transition-all duration-700 animate-pulse"></div>

      <div className="relative flex flex-col items-center gap-3">
        <div className="w-20 h-20 rounded-full glass-panel flex items-center justify-center p-1 shadow-[0_0_40px_rgba(56,189,248,0.2)] border-white/10 group-hover:border-sky-400/30 transition-all">
          <div className="w-full h-full rounded-full bg-gradient-to-br from-sky-400/20 to-sky-600/20 flex items-center justify-center overflow-hidden border border-white/5">
            <BrainCircuit size={32} className="text-sky-300 drop-shadow-[0_0_8px_rgba(125,211,252,0.8)]" />
          </div>
        </div>
        <div className="text-center">
          <p className="text-[13px] font-semibold text-white/90 tracking-tight">{data.label}</p>
          <p className="text-[10px] text-white/40 uppercase tracking-[0.15em] font-medium">{data.subLabel || 'Core Action'}</p>
        </div>
      </div>

      <Handle type="source" position={Position.Right} className="!w-3 !h-3" />
    </div>
  );
});

export const ActionNode = memo(({ data }: NodeProps<ProxxyNodeData>) => {
  const isTNode = data.label === 'T';
  return (
    <div className="relative group flex items-center gap-3 px-1">
      <div className={`w-11 h-11 rounded-xl glass-panel flex items-center justify-center shadow-lg transition-all group-hover:scale-110 group-hover:shadow-sky-500/10 ${isTNode ? 'bg-white/5' : ''}`}>
        {isTNode ? (
          <span className="text-lg font-bold text-white/30 italic font-mono">T</span>
        ) : data.label === 'Add file' ? (
          <FileText size={18} className="text-white/60 group-hover:text-white transition-colors" />
        ) : (
          <Layers size={18} className="text-white/60" />
        )}
      </div>
      <div className="absolute left-14 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
        <p className="text-[11px] font-semibold text-white/80">{data.label}</p>
        <p className="text-[9px] text-white/30">Core action</p>
      </div>
      <Handle type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} />
    </div>
  );
});

export const ConditionNode = memo(({ data }: NodeProps<ProxxyNodeData>) => {
  return (
    <div className="group relative">
      <div className="px-5 py-2 rounded-full glass-panel border border-white/10 hover:border-sky-400/40 hover:bg-white/5 transition-all shadow-xl">
        <span className="text-[11px] font-medium text-white/70 tracking-wide">{data.label}</span>
      </div>
      <Handle type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} />
    </div>
  );
});

export const IntegrationNode = memo(({ data }: NodeProps<ProxxyNodeData>) => {
  return (
    <div className="relative group flex items-center gap-4">
      <div className="w-16 h-16 rounded-full glass-panel flex items-center justify-center p-3 shadow-2xl border-white/5 group-hover:border-white/20 transition-all">
        <div className="w-full h-full rounded-full bg-white/5 border border-white/5 flex items-center justify-center">
          <Cpu size={20} className="text-white/40 group-hover:text-white/80 transition-colors" />
        </div>
      </div>
      <div>
        <p className="text-[12px] font-semibold text-white/80">{data.label}</p>
        <p className="text-[10px] text-white/30 font-medium tracking-wide uppercase">{data.subLabel || 'Integration'}</p>
      </div>
      <Handle type="target" position={Position.Left} />
    </div>
  );
});
