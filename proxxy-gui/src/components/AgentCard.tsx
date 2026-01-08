import { Server, Activity, Clock } from 'lucide-react';
import { AgentInfo } from '../types';
import { formatDistanceToNow } from 'date-fns';

interface AgentCardProps {
    agent: AgentInfo;
}

export function AgentCard({ agent }: AgentCardProps) {
    const isOnline = agent.status === 'Online';

    const getLastSeen = () => {
        try {
            return formatDistanceToNow(new Date(agent.last_heartbeat), { addSuffix: true });
        } catch {
            return 'Unknown';
        }
    };

    return (
        <div className="bg-[#111318] border border-white/5 rounded-xl p-4 hover:border-white/10 transition-colors">
            <div className="flex items-start justify-between mb-3">
                <div className="flex items-center gap-2">
                    <div className={`w-8 h-8 rounded flex items-center justify-center ${isOnline ? 'bg-emerald-500/10 border border-emerald-500/20' : 'bg-gray-500/10 border border-gray-500/20'
                        }`}>
                        <Server size={16} className={isOnline ? 'text-emerald-400' : 'text-gray-400'} />
                    </div>
                    <div>
                        <h3 className="text-sm font-bold text-white">{agent.id.substring(0, 12)}...</h3>
                        <p className="text-[10px] text-white/40 font-mono">{agent.address}:{agent.port}</p>
                    </div>
                </div>
                <span className={`px-2 py-0.5 rounded text-[10px] font-bold ${isOnline
                        ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                        : 'bg-gray-500/10 text-gray-400 border border-gray-500/20'
                    }`}>
                    {agent.status}
                </span>
            </div>

            <div className="space-y-2 text-[11px]">
                <div className="flex items-center gap-2 text-white/50">
                    <Clock size={12} className="text-white/30" />
                    <span>Last seen: {getLastSeen()}</span>
                </div>
                <div className="flex items-center gap-2 text-white/50">
                    <Activity size={12} className="text-white/30" />
                    <span>v{agent.version}</span>
                </div>
            </div>

            {agent.capabilities && agent.capabilities.length > 0 && (
                <div className="mt-3 pt-3 border-t border-white/5">
                    <div className="flex flex-wrap gap-1">
                        {agent.capabilities.map((cap, idx) => (
                            <span
                                key={idx}
                                className="px-2 py-0.5 bg-white/5 rounded text-[9px] text-white/40 font-mono"
                            >
                                {cap}
                            </span>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
