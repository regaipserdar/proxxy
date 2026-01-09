import React from 'react';
import { useParams, Link } from 'react-router-dom';
import { useQuery, useSubscription } from '@apollo/client';
import { GET_AGENT_DETAILS, SYSTEM_METRICS_UPDATES } from '@/graphql/operations';
import { Agent, SystemMetrics } from '@/types/graphql';
import { ArrowLeft, Server, Activity, HardDrive, Cpu, Wifi } from 'lucide-react';

export const AgentDetailView = () => {
    const { agentId } = useParams<{ agentId: string }>();

    // 1. Fetch initial details (Agent info + Current Metrics)
    const { data: queryData, loading: queryLoading, error: queryError } = useQuery(GET_AGENT_DETAILS, {
        variables: { agentId },
        pollInterval: 5000,
        skip: !agentId
    });

    // 2. Subscribe to real-time metrics
    const { data: subData } = useSubscription(SYSTEM_METRICS_UPDATES, {
        variables: { agentId },
        skip: !agentId
    });

    if (queryLoading && !queryData) return <div className="p-10 text-white">Loading Agent Details...</div>;
    if (queryError) return <div className="p-10 text-red-400">Error: {queryError.message}</div>;

    // Filter the specific agent from the list (since API returns all)
    const agent = queryData?.agents.find((a: Agent) => a.id === agentId);

    // Use latest metrics from subscription if available, otherwise fallback to query data
    const metrics: SystemMetrics | undefined = subData?.systemMetricsUpdates || queryData?.currentSystemMetrics;

    if (!agent) return <div className="p-10 text-white">Agent not found</div>;

    const isOnline = agent.status === 'Online';

    return (
        <div className="p-8 h-full flex flex-col gap-6 w-full max-w-[1200px] mx-auto animate-in slide-in-from-right duration-300">
            {/* Header */}
            <div>
                <Link to="/agents" className="inline-flex items-center gap-2 text-white/40 hover:text-white mb-4 transition-colors">
                    <ArrowLeft size={16} />
                    <span>Back to Agents</span>
                </Link>
                <div className="flex items-start justify-between">
                    <div>
                        <div className="flex items-center gap-3 mb-2">
                            <div className={`p-2 rounded-lg ${isOnline ? 'bg-emerald-500/10' : 'bg-red-500/10'}`}>
                                <Server className={isOnline ? 'text-emerald-400' : 'text-red-400'} size={24} />
                            </div>
                            <h1 className="text-3xl font-bold text-white tracking-tight">{agent.name || agent.id}</h1>
                        </div>
                        <div className="flex items-center gap-4 text-sm text-white/40 font-mono">
                            <span>ID: {agent.id}</span>
                            <span>•</span>
                            <span>{agent.hostname}</span>
                            <span>•</span>
                            <span>v{agent.version}</span>
                        </div>
                    </div>
                    <div className={`px-4 py-2 rounded-full border ${isOnline ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400' : 'bg-red-500/10 border-red-500/20 text-red-400'
                        } font-bold text-sm uppercase tracking-wider`}>
                        {agent.status}
                    </div>
                </div>
            </div>

            {/* Metrics Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <MetricCard
                    icon={<Cpu size={20} className="text-blue-400" />}
                    label="CPU Usage"
                    value={`${metrics?.cpuUsagePercent?.toFixed(1) || 0}%`}
                    subValue="System Load"
                />
                <MetricCard
                    icon={<Activity size={20} className="text-purple-400" />}
                    label="Memory"
                    // Parse memory bytes if string, else number
                    value={formatBytes(Number(metrics?.memoryUsedBytes || 0))}
                    subValue={`of ${formatBytes(Number(metrics?.memoryTotalBytes || 0))}`}
                />
                <MetricCard
                    icon={<Wifi size={20} className="text-orange-400" />}
                    label="Network"
                    value={formatBytes(Number(metrics?.networkRxBytesPerSec || 0)) + '/s'}
                    subValue="Incoming"
                />
                <MetricCard
                    icon={<HardDrive size={20} className="text-emerald-400" />}
                    label="Disk I/O"
                    value={formatBytes(Number(metrics?.diskReadBytesPerSec || 0)) + '/s'}
                    subValue="Read Speed"
                />
            </div>

            {/* Placeholder for more details/logs */}
            <div className="bg-[#111318] border border-white/5 rounded-xl p-6 min-h-[300px] flex items-center justify-center text-white/20">
                <div className="text-center">
                    <Activity size={48} className="mx-auto mb-4 opacity-50" />
                    <p>Detailed historical logs and charts coming soon...</p>
                </div>
            </div>
        </div>
    );
};

// Helper component
const MetricCard = ({ icon, label, value, subValue }: { icon: React.ReactNode, label: string, value: string, subValue: string }) => (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-5 hover:border-white/10 transition-colors">
        <div className="flex items-center justify-between mb-4">
            <span className="text-white/40 text-xs font-bold uppercase tracking-wider">{label}</span>
            {icon}
        </div>
        <div className="text-2xl font-bold text-white mb-1 font-mono">{value}</div>
        <div className="text-xs text-white/30">{subValue}</div>
    </div>
);

function formatBytes(bytes: number, decimals = 2) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}
