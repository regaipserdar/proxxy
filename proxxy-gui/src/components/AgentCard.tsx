import { Server, Activity, Clock, Cpu, Database, ChevronRight, Loader2 } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Agent } from '../types/graphql';
import { formatDistanceToNow } from 'date-fns';
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { useQuery } from '@apollo/client';
import { GET_CURRENT_SYSTEM_METRICS } from '@/graphql/operations';

interface AgentCardProps {
    agent: Agent;
}

export function AgentCard({ agent }: AgentCardProps) {
    const isOnline = agent.status?.toLowerCase() === 'online';

    const { data: metricsData, loading: metricsLoading } = useQuery(GET_CURRENT_SYSTEM_METRICS, {
        variables: { agentId: agent.id },
        skip: !isOnline,
        pollInterval: 10000,
    });

    const metrics = metricsData?.currentSystemMetrics;

    const getLastSeen = () => {
        try {
            if (!agent.lastHeartbeat) return 'Never';
            return formatDistanceToNow(new Date(agent.lastHeartbeat), { addSuffix: true });
        } catch {
            return 'Unknown';
        }
    };

    const cpuUsage = metrics?.cpuUsagePercent ?? 0;
    const memTotal = metrics?.memoryTotalBytes ? parseInt(metrics.memoryTotalBytes, 10) : 0;
    const memUsed = metrics?.memoryUsedBytes ? parseInt(metrics.memoryUsedBytes, 10) : 0;
    const memUsagePercent = memTotal > 0 ? (memUsed / memTotal) * 100 : 0;
    const memUsedGB = (memUsed / (1024 * 1024 * 1024)).toFixed(1);

    return (
        <Link to={`/agents/${agent.id}`} className="block group">
            <Card className="bg-[#111318] border-white/5 group-hover:border-indigo-500/30 transition-all duration-300 shadow-xl relative overflow-hidden h-full">
                {/* Status Glow Indicator */}
                <div className={`absolute top-0 left-0 w-full h-[1px] ${isOnline ? 'bg-emerald-500/50 shadow-[0_0_8px_rgba(16,185,129,0.3)]' : 'bg-slate-700/50'} z-20`} />

                <CardContent className="p-5 flex flex-col h-full gap-4 relative z-10">
                    <div className="flex items-start justify-between">
                        <div className="flex items-center gap-3">
                            <div className={`p-2 rounded-xl border shadow-inner transition-all duration-300 ${isOnline
                                    ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400 group-hover:scale-110'
                                    : 'bg-slate-800/20 border-white/5 text-slate-500'
                                }`}>
                                <Server size={18} />
                            </div>
                            <div className="min-w-0">
                                <h3 className="text-sm font-black text-white truncate tracking-tight group-hover:text-indigo-400 transition-colors uppercase">
                                    {agent.name || agent.id.substring(0, 8)}
                                </h3>
                                <p className="text-[10px] text-slate-500 font-mono font-medium truncate uppercase tracking-tighter">
                                    {agent.hostname}
                                </p>
                            </div>
                        </div>
                        <Badge variant="outline" className={`h-6 rounded-full font-black text-[9px] uppercase tracking-[0.15em] px-2.5 transition-all ${isOnline
                                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                                : 'bg-slate-900 text-slate-500 border-white/5'
                            }`}>
                            {isOnline && <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 mr-2 animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.5)]" />}
                            {agent.status || 'Offline'}
                        </Badge>
                    </div>

                    {/* Quick Stats Grid */}
                    <div className="grid grid-cols-2 gap-2">
                        <div className="bg-white/[0.02] rounded-xl p-3 border border-white/5 flex flex-col gap-2 transition-colors group-hover:bg-white/[0.04]">
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-1.5 text-[9px] font-black text-slate-500 uppercase tracking-widest">
                                    <Cpu size={10} className="text-indigo-400/70" />
                                    CPU
                                </div>
                                {isOnline && metricsLoading && !metrics && <Loader2 size={8} className="animate-spin text-slate-700" />}
                            </div>
                            <div className="space-y-1.5">
                                <div className="flex items-center justify-between">
                                    <span className="text-[11px] font-mono font-bold text-slate-300">{cpuUsage.toFixed(1)}%</span>
                                </div>
                                <Progress value={cpuUsage} className="h-1 bg-white/5" />
                            </div>
                        </div>
                        <div className="bg-white/[0.02] rounded-xl p-3 border border-white/5 flex flex-col gap-2 transition-colors group-hover:bg-white/[0.04]">
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-1.5 text-[9px] font-black text-slate-500 uppercase tracking-widest">
                                    <Database size={10} className="text-emerald-400/70" />
                                    RAM
                                </div>
                                {isOnline && metricsLoading && !metrics && <Loader2 size={8} className="animate-spin text-slate-700" />}
                            </div>
                            <div className="space-y-1.5">
                                <div className="flex items-center justify-between">
                                    <span className="text-[11px] font-mono font-bold text-slate-300">{memUsedGB} GB</span>
                                </div>
                                <Progress value={memUsagePercent} className="h-1 bg-white/5" />
                            </div>
                        </div>
                    </div>

                    {/* Footer Info */}
                    <div className="mt-auto pt-3 border-t border-white/5 flex items-center justify-between">
                        <div className="flex flex-col gap-1">
                            <div className="flex items-center gap-1.5 text-[10px] text-slate-500 font-bold uppercase tracking-tight">
                                <Clock size={10} className="opacity-50" />
                                <span>Seen {getLastSeen()}</span>
                            </div>
                            <div className="flex items-center gap-1.5 text-[10px] text-slate-600 font-black uppercase tracking-widest">
                                <Activity size={10} className="opacity-50" />
                                <span>v{agent.version || '1.0.0'}</span>
                            </div>
                        </div>
                        <ChevronRight size={16} className="text-slate-700 group-hover:text-indigo-400 transition-all translate-x-2 opacity-0 group-hover:translate-x-0 group-hover:opacity-100" />
                    </div>
                </CardContent>
            </Card>
        </Link>
    );
}
