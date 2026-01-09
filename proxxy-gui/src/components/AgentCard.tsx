import { Server, Activity, Clock } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Agent } from '../types/graphql';
import { formatDistanceToNow } from 'date-fns';

interface AgentCardProps {
    agent: Agent;
}

export function AgentCard({ agent }: AgentCardProps) {
    const isOnline = agent.status === 'Online';

    const getLastSeen = () => {
        try {
            return formatDistanceToNow(new Date(agent.lastHeartbeat), { addSuffix: true });
        } catch {
            return 'Unknown';
        }
    };

    return (
        <Link to={`/agents/${agent.id}`} className="block group">
            <div className="bg-[#111318] border border-white/5 rounded-xl p-4 group-hover:border-white/20 transition-all duration-200 shadow-sm group-hover:shadow-[0_0_15px_rgba(255,255,255,0.05)] cursor-pointer">
                <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-2">
                        <div className={`w-8 h-8 rounded flex items-center justify-center transition-colors ${isOnline ? 'bg-emerald-500/10 border border-emerald-500/20 group-hover:bg-emerald-500/20' : 'bg-gray-500/10 border border-gray-500/20 group-hover:bg-gray-500/20'
                            }`}>
                            <Server size={16} className={isOnline ? 'text-emerald-400' : 'text-gray-400'} />
                        </div>
                        <div>
                            <h3 className="text-sm font-bold text-white group-hover:text-[#9DCDE8] transition-colors">{agent.name || agent.id.substring(0, 12)}</h3>
                            <p className="text-[10px] text-white/40 font-mono">{agent.hostname}</p>
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
            </div>
        </Link>
    );
}
