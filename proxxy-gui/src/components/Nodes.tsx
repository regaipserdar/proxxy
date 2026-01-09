
import { memo, ReactNode } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { ProxxyNodeData, NodeType } from '../types';
import { Filter, Edit3, Repeat, Terminal, Radio } from 'lucide-react';

const NodeWrapper = ({ selected, children, label, subLabel, type }: { selected: boolean, children?: ReactNode, label: string, subLabel?: string, type: NodeType }) => {
  const getAccentColor = () => {
    switch (type) {
      case NodeType.TRIGGER: return 'border-emerald-500 shadow-emerald-500/20';
      case NodeType.MATCHER: return 'border-amber-500 shadow-amber-500/20';
      case NodeType.MODIFIER: return 'border-purple-500 shadow-purple-500/20';
      case NodeType.REPEATER: return 'border-sky-500 shadow-sky-500/20';
      default: return 'border-[#9DCDE8] shadow-[#9DCDE8]/20';
    }
  };

  return (
    <div className={`relative flex flex-col items-center gap-2 transition-all duration-300 ${selected ? 'scale-105' : ''}`}>
      <div className={`w-16 h-16 rounded-2xl glass-panel flex items-center justify-center border transition-all duration-500 ${selected ? `${getAccentColor()} shadow-[0_0_30px_rgba(0,0,0,0.5)] ring-1 ring-white/10` : 'border-white/10 group-hover:border-white/20 shadow-xl'
        }`}>
        <div className={`w-12 h-12 rounded-xl flex items-center justify-center transition-colors ${selected ? 'bg-white/10' : 'bg-white/5'}`}>
          {children}
        </div>
      </div>
      <div className="text-center min-w-[80px]">
        <p className={`text-[11px] font-bold tracking-tight transition-colors ${selected ? 'text-white' : 'text-white/80'}`}>{label}</p>
        <p className="text-[8px] text-white/30 uppercase font-medium tracking-widest">{subLabel}</p>
      </div>
    </div>
  );
};

export const TriggerNode = memo(({ data, selected }: NodeProps<ProxxyNodeData>) => (
  <div className="relative group">
    <NodeWrapper selected={!!selected} label={data.label} subLabel="Trigger" type={data.type}>
      <Radio size={24} className={selected ? 'text-emerald-400' : 'text-white/40'} />
    </NodeWrapper>
    <Handle type="source" position={Position.Right} style={{ top: '32px' }} />
  </div>
));

export const MatcherNode = memo(({ data, selected }: NodeProps<ProxxyNodeData>) => (
  <div className="relative group">
    <Handle type="target" position={Position.Left} style={{ top: '32px' }} />
    <NodeWrapper selected={!!selected} label={data.label} subLabel="Matcher" type={data.type}>
      <Filter size={22} className={selected ? 'text-amber-400' : 'text-white/40'} />
    </NodeWrapper>
    <Handle type="source" position={Position.Right} id="match" style={{ top: '32px' }} />
    <Handle type="source" position={Position.Bottom} id="fail" style={{ left: '50%', bottom: '26px' }} />
  </div>
));

export const ModifierNode = memo(({ data, selected }: NodeProps<ProxxyNodeData>) => (
  <div className="relative group">
    <Handle type="target" position={Position.Left} style={{ top: '32px' }} />
    <NodeWrapper selected={!!selected} label={data.label} subLabel="Modifier" type={data.type}>
      <Edit3 size={22} className={selected ? 'text-purple-400' : 'text-white/40'} />
    </NodeWrapper>
    <Handle type="source" position={Position.Right} style={{ top: '32px' }} />
  </div>
));

export const RepeaterNode = memo(({ data, selected }: NodeProps<ProxxyNodeData>) => (
  <div className="relative group">
    <Handle type="target" position={Position.Left} style={{ top: '32px' }} />
    <NodeWrapper selected={!!selected} label={data.label} subLabel="Repeater" type={data.type}>
      <Repeat size={22} className={selected ? 'text-sky-400' : 'text-white/40'} />
    </NodeWrapper>
    <Handle type="source" position={Position.Right} style={{ top: '32px' }} />
  </div>
));

export const SinkNode = memo(({ data, selected }: NodeProps<ProxxyNodeData>) => (
  <div className="relative group">
    <Handle type="target" position={Position.Left} style={{ top: '32px' }} />
    <NodeWrapper selected={!!selected} label={data.label} subLabel="Output" type={data.type}>
      <Terminal size={22} className={selected ? 'text-white' : 'text-white/40'} />
    </NodeWrapper>
  </div>
));
