import React, { useMemo } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useQuery, useSubscription } from '@apollo/client';
import { GET_AGENT_DETAILS, SYSTEM_METRICS_UPDATES, GET_HTTP_TRANSACTIONS, TRAFFIC_UPDATES } from '@/graphql/operations';
import { Agent, SystemMetrics } from '@/types/graphql';
import {
    ArrowLeft,
    Server,
    Activity,
    HardDrive,
    Cpu,
    Wifi,
    Clock,
    Database,
    Zap,
    Box,
    RefreshCw,
    Terminal,
    ChevronRight,
    Network
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { formatDistanceToNow } from 'date-fns';


export const AgentDetailView = () => {
    const { agentId } = useParams<{ agentId: string }>();

    // 1. Fetch initial details (Agent info + Current Metrics)
    const { data: queryData, loading: queryLoading, error: queryError } = useQuery(GET_AGENT_DETAILS, {
        variables: { agentId },
        pollInterval: 10000,
        skip: !agentId
    });

    // 2. Subscribe to real-time metrics
    const { data: subData } = useSubscription(SYSTEM_METRICS_UPDATES, {
        variables: { agentId },
        skip: !agentId
    });

    // 3. Fetch traffic for logs (filtered by agentId in backend)
    const { data: trafficData } = useQuery(GET_HTTP_TRANSACTIONS, {
        variables: { agentId: agentId?.toLowerCase() },
        pollInterval: 5000,
    });

    useSubscription(TRAFFIC_UPDATES, {
        variables: { agentId: agentId?.toLowerCase() },
    });

    // Filter agents and traffic
    const agent = queryData?.agents.find((a: Agent) => a.id?.toLowerCase() === agentId?.toLowerCase());
    const agentTraffic = useMemo(() => {
        return [...(trafficData?.requests || [])]
            .sort((a: any, b: any) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
    }, [trafficData]);

    // Use latest metrics from subscription if available, otherwise fallback to query data
    const metrics: SystemMetrics | undefined = subData?.systemMetricsUpdates || queryData?.currentSystemMetrics;

    const isOnline = agent?.status?.toLowerCase() === 'online';
    const agentName = agent?.name && agent.name.toLowerCase() !== 'unknown' ? agent.name : (agent?.id?.substring(0, 8) || "UNNAMED_NODE");

    const getUptime = (seconds: number = 0) => {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        return `${h}h ${m}m ${s}s`;
    };

    if (queryLoading && !queryData) {
        return (
            <div className="p-8 space-y-8 animate-pulse">
                <div className="h-20 bg-white/5 rounded-2xl w-1/3" />
                <div className="grid grid-cols-4 gap-4">
                    <div className="h-32 bg-white/5 rounded-2xl" />
                    <div className="h-32 bg-white/5 rounded-2xl" />
                    <div className="h-32 bg-white/5 rounded-2xl" />
                    <div className="h-32 bg-white/5 rounded-2xl" />
                </div>
                <div className="h-96 bg-white/5 rounded-2xl" />
            </div>
        );
    }

    if (queryError || !agent) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center p-20">
                <div className="p-6 rounded-3xl bg-red-500/10 border border-red-500/20 mb-6">
                    <Server size={48} className="text-red-400" />
                </div>
                <h3 className="text-2xl font-black text-white uppercase tracking-tighter mb-2">Agent Sync Lost</h3>
                <p className="text-slate-500 font-medium text-center max-w-md">
                    {queryError ? queryError.message : "The requested node could not be located in the current workspace cluster."}
                </p>
                <Link to="/agents" className="mt-8">
                    <Badge className="px-6 py-2 bg-white/5 hover:bg-white/10 text-white cursor-pointer border-white/10 uppercase font-black tracking-widest transition-all">
                        Return to Fleet
                    </Badge>
                </Link>
            </div>
        );
    }

    return (
        <div className="p-8 h-full flex flex-col gap-8 w-full max-w-[1600px] mx-auto overflow-y-auto custom-scrollbar animate-in slide-in-from-right duration-500">
            {/* Breadcrumbs & Navigation */}
            <div className="flex items-center gap-4 text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">
                <Link to="/agents" className="hover:text-white transition-colors">Fleet Console</Link>
                <ChevronRight size={10} className="opacity-30" />
                <span className="text-indigo-400">Node Detail</span>
            </div>

            {/* Header Section */}
            <div className="flex flex-col lg:flex-row justify-between items-start lg:items-center gap-6">
                <div className="flex items-center gap-5">
                    <div className={`p-5 rounded-[2rem] border transition-all duration-500 ${isOnline
                        ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400 shadow-[0_0_30px_rgba(16,185,129,0.1)]'
                        : 'bg-red-500/10 border-red-500/20 text-red-400'
                        }`}>
                        <Server size={40} className={isOnline ? 'animate-pulse' : ''} />
                    </div>
                    <div className="space-y-1">
                        <div className="flex items-center gap-3">
                            <h1 className="text-4xl font-black text-white uppercase tracking-tighter">
                                {agentName}
                            </h1>
                            <Badge className={`${isOnline
                                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                                : 'bg-red-500/10 text-red-400 border-red-500/20'
                                } px-3 py-1 uppercase font-black tracking-widest h-fit`}>
                                {isOnline && <span className="w-2 h-2 rounded-full bg-emerald-500 mr-2 animate-ping" />}
                                {agent.status}
                            </Badge>
                        </div>
                        <div className="flex flex-wrap items-center gap-x-6 gap-y-2 text-xs font-bold text-slate-500 uppercase tracking-widest">
                            <div className="flex items-center gap-2">
                                <Terminal size={14} className="opacity-50" />
                                <span className="text-indigo-400/70 font-mono select-all uppercase">{agent.id}</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <Box size={14} className="opacity-50" />
                                <span className="text-slate-400">{agent.hostname}</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <Zap size={14} className="opacity-50 text-amber-500/70" />
                                <span>Version {agent.version}</span>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="flex items-center gap-3">
                    <button className="h-12 px-6 bg-white/[0.03] border border-white/5 rounded-2xl font-black uppercase tracking-widest text-[10px] text-white hover:bg-white/[0.06] transition-all active:scale-95 flex items-center gap-2">
                        <RefreshCw size={14} className="text-indigo-400" />
                        Restart Node
                    </button>
                    <button className="h-12 px-6 bg-red-500/10 border border-red-500/20 rounded-2xl font-black uppercase tracking-widest text-[10px] text-red-400 hover:bg-red-500/20 transition-all active:scale-95 flex items-center gap-2">
                        <Activity size={14} />
                        Decommission
                    </button>
                </div>
            </div>

            <Separator className="bg-white/5" />

            {/* Main Content Area */}
            <div className="grid grid-cols-1 xl:grid-cols-3 gap-8">
                {/* Left Column: Real-time Telemetry */}
                <div className="xl:col-span-2 space-y-8">
                    {/* Primary Metrics */}
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                        <MetricTile
                            icon={<Cpu className="text-indigo-400" />}
                            label="System CPU"
                            value={`${metrics?.cpuUsagePercent?.toFixed(1) || 0}%`}
                            progress={metrics?.cpuUsagePercent || 0}
                            subValue="Host utilization"
                        />
                        <MetricTile
                            icon={<Database className="text-emerald-400" />}
                            label="Memory"
                            value={formatBytes(Number(metrics?.memoryUsedBytes || 0))}
                            progress={metrics?.memoryUsedBytes && metrics?.memoryTotalBytes ? (Number(metrics.memoryUsedBytes) / Number(metrics.memoryTotalBytes)) * 100 : 0}
                            subValue={`of ${formatBytes(Number(metrics?.memoryTotalBytes || 0))}`}
                        />
                        <MetricTile
                            icon={<Network className="text-purple-400" />}
                            label="Net Throughput"
                            value={formatBytes(Number(metrics?.networkRxBytesPerSec || 0)) + '/s'}
                            subText={`TX: ${formatBytes(Number(metrics?.networkTxBytesPerSec || 0))}/s`}
                            subValue="In/Out Speed"
                            glow="purple"
                        />
                        <MetricTile
                            icon={<Clock className="text-amber-400" />}
                            label="Uptime"
                            value={getUptime(metrics?.processUptimeSeconds)}
                            subValue="Active Session"
                        />
                    </div>

                    {/* Tabs for Analysis */}
                    <Tabs defaultValue="overview" className="space-y-6">
                        <TabsList className="bg-[#111318] border border-white/5 p-1 rounded-2xl h-14">
                            <TabsTrigger value="overview" className="h-12 px-8 rounded-xl font-black uppercase tracking-widest text-[11px] data-[state=active]:bg-white/5 data-[state=active]:text-white transition-all">Telemetry</TabsTrigger>
                            <TabsTrigger value="history" className="h-12 px-8 rounded-xl font-black uppercase tracking-widest text-[11px] data-[state=active]:bg-white/5 data-[state=active]:text-white transition-all">Full Logs</TabsTrigger>
                        </TabsList>

                        <TabsContent value="overview" className="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-400">
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <Card className="bg-[#111318] border-white/5 overflow-hidden">
                                    <CardHeader className="pb-2">
                                        <div className="flex justify-between items-center">
                                            <CardTitle className="text-xs font-black uppercase tracking-[0.2em] text-slate-500">I/O Performance</CardTitle>
                                            <HardDrive size={14} className="text-indigo-400 opacity-50" />
                                        </div>
                                    </CardHeader>
                                    <CardContent className="space-y-6 pt-4">
                                        <div className="flex items-center justify-between">
                                            <div className="flex items-center gap-3">
                                                <div className="w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center text-indigo-400">
                                                    <ArrowLeft size={14} className="rotate-90" />
                                                </div>
                                                <div>
                                                    <p className="text-[10px] font-black uppercase tracking-widest text-slate-500">Disk Read</p>
                                                    <p className="text-lg font-mono font-bold text-white">{formatBytes(Number(metrics?.diskReadBytesPerSec || 0))}/s</p>
                                                </div>
                                            </div>
                                            <div className="flex items-center gap-3 text-right">
                                                <div>
                                                    <p className="text-[10px] font-black uppercase tracking-widest text-slate-500">Disk Write</p>
                                                    <p className="text-lg font-mono font-bold text-white">{formatBytes(Number(metrics?.diskWriteBytesPerSec || 0))}/s</p>
                                                </div>
                                                <div className="w-8 h-8 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400">
                                                    <ArrowLeft size={14} className="-rotate-90" />
                                                </div>
                                            </div>
                                        </div>
                                        <div className="h-24 bg-black/40 border border-white/5 rounded-2xl relative flex items-center justify-center p-4">
                                            <div className="text-center">
                                                <Activity size={24} className="mx-auto mb-2 text-indigo-500 opacity-20" />
                                                <p className="text-[9px] font-black text-slate-600 uppercase tracking-widest">Real-time I/O Stream Active</p>
                                            </div>
                                        </div>
                                    </CardContent>
                                </Card>

                                <Card className="bg-[#111318] border-white/5 overflow-hidden">
                                    <CardHeader className="pb-2">
                                        <div className="flex justify-between items-center">
                                            <CardTitle className="text-xs font-black uppercase tracking-[0.2em] text-slate-500">Network Distribution</CardTitle>
                                            <Wifi size={14} className="text-purple-400 opacity-50" />
                                        </div>
                                    </CardHeader>
                                    <CardContent className="space-y-6 pt-4">
                                        <div className="space-y-4">
                                            <div className="space-y-2">
                                                <div className="flex justify-between text-[10px] font-bold uppercase text-slate-400">
                                                    <span>Inbound Payload</span>
                                                    <span>{formatBytes(Number(metrics?.networkRxBytesPerSec || 0))}/s</span>
                                                </div>
                                                <Progress value={45} className="h-1 bg-white/5" />
                                            </div>
                                            <div className="space-y-2">
                                                <div className="flex justify-between text-[10px] font-bold uppercase text-slate-400">
                                                    <span>Outbound Payload</span>
                                                    <span>{formatBytes(Number(metrics?.networkTxBytesPerSec || 0))}/s</span>
                                                </div>
                                                <Progress value={20} className="h-1 bg-white/5" />
                                            </div>
                                        </div>
                                        <div className="grid grid-cols-2 gap-3">
                                            <div className="p-3 bg-white/[0.02] border border-white/5 rounded-xl">
                                                <p className="text-[8px] font-black uppercase tracking-widest text-indigo-400/50 mb-1">Latency Avg</p>
                                                <p className="text-sm font-mono font-bold text-white">14ms</p>
                                            </div>
                                            <div className="p-3 bg-white/[0.02] border border-white/5 rounded-xl">
                                                <p className="text-[8px] font-black uppercase tracking-widest text-emerald-400/50 mb-1">Packet Loss</p>
                                                <p className="text-sm font-mono font-bold text-white">0.02%</p>
                                            </div>
                                        </div>
                                    </CardContent>
                                </Card>
                            </div>

                        </TabsContent>

                        <TabsContent value="history" className="space-y-4 animate-in fade-in slide-in-from-bottom-2 duration-400">
                            <Card className="bg-[#111318] border-white/5">
                                <CardHeader className="flex flex-row items-center justify-between">
                                    <CardTitle className="text-sm font-black uppercase tracking-widest text-white">Full Traffic Logs</CardTitle>
                                    <Badge variant="outline" className="text-[8px] border-white/10 text-slate-500">Live Updates</Badge>
                                </CardHeader>
                                <CardContent className="p-0">
                                    <div className="overflow-x-auto">
                                        <table className="w-full text-left border-collapse">
                                            <thead className="bg-white/[0.02] border-y border-white/5">
                                                <tr>
                                                    <th className="px-6 py-3 text-[9px] font-black text-slate-500 uppercase tracking-widest">Method</th>
                                                    <th className="px-6 py-3 text-[9px] font-black text-slate-500 uppercase tracking-widest">Target URL</th>
                                                    <th className="px-6 py-3 text-[9px] font-black text-slate-500 uppercase tracking-widest">Status</th>
                                                    <th className="px-6 py-3 text-[9px] font-black text-slate-500 uppercase tracking-widest text-right">Time</th>
                                                </tr>
                                            </thead>
                                            <tbody className="divide-y divide-white/5">
                                                {agentTraffic.length > 0 ? agentTraffic.map((t: any) => (
                                                    <tr key={t.requestId} className="hover:bg-white/[0.01] transition-colors group">
                                                        <td className="px-6 py-4">
                                                            <Badge variant="outline" className="text-[9px] font-black bg-indigo-500/5 text-indigo-400 border-indigo-500/20">{t.method}</Badge>
                                                        </td>
                                                        <td className="px-6 py-4">
                                                            <div className="text-[11px] font-mono text-slate-400 truncate max-w-[300px] group-hover:text-white transition-colors">{t.url}</div>
                                                        </td>
                                                        <td className="px-6 py-4">
                                                            <span className={`text-[10px] font-black ${t.status < 400 ? 'text-emerald-400' : 'text-red-400'}`}>{t.status}</span>
                                                        </td>
                                                        <td className="px-6 py-4 text-right">
                                                            <span className="text-[10px] font-mono text-slate-600">{formatDistanceToNow(new Date(t.timestamp), { addSuffix: true })}</span>
                                                        </td>
                                                    </tr>
                                                )) : (
                                                    <tr>
                                                        <td colSpan={4} className="px-6 py-20 text-center">
                                                            <p className="text-[10px] font-black text-slate-600 uppercase tracking-widest">No traffic patterns recorded for this node</p>
                                                        </td>
                                                    </tr>
                                                )}
                                            </tbody>
                                        </table>
                                    </div>
                                </CardContent>
                            </Card>
                        </TabsContent>
                    </Tabs>
                </div>

                {/* Right Column: Node Info & Health */}
                <div className="space-y-6">
                    <Card className="bg-[#111318] border-white/5 p-6">
                        <div className="flex items-center justify-between mb-6">
                            <span className="text-xs font-black uppercase tracking-widest text-white">Event Stream</span>
                            <Badge variant="outline" className="text-[9px] border-white/10 text-slate-500 uppercase tracking-widest">Live</Badge>
                        </div>
                        <div className="space-y-4">
                            {agentTraffic.length > 0 ? agentTraffic.slice(0, 15).map((t: any) => (
                                <div key={t.requestId} className="flex gap-3 text-[11px]">
                                    <span className="text-slate-600 font-bold whitespace-nowrap">{formatDistanceToNow(new Date(t.timestamp), { addSuffix: false }).replace('about ', '')}</span>
                                    <div className="flex-1 min-w-0">
                                        <p className="text-white font-bold tracking-tight uppercase leading-none mb-1 flex items-center gap-2">
                                            <span className={`${t.status < 400 ? 'text-emerald-400' : 'text-red-400'}`}>{t.method}</span>
                                            <span className="text-slate-400 text-[10px]">{t.status}</span>
                                        </p>
                                        <p className="text-[9px] text-slate-500 font-mono truncate">{t.url}</p>
                                    </div>
                                </div>
                            )) : (
                                <div className="text-center py-20 opacity-30">
                                    <Activity className="mx-auto mb-2" size={32} />
                                    <p className="text-[10px] font-black uppercase tracking-widest">No Active Traffic</p>
                                </div>
                            )}
                        </div>
                        <Link to="/traffic-tree">
                            <Button variant="ghost" className="w-full mt-6 text-[10px] font-black uppercase tracking-widest hover:bg-white/5 border border-white/5 h-10">
                                Global Traffic Monitor
                            </Button>
                        </Link>
                    </Card>

                    <Card className="bg-[#111318] border-white/5 p-6">
                        <div className="space-y-4">
                            <div className="flex justify-between items-center text-[11px]">
                                <span className="font-bold text-slate-600 uppercase tracking-tighter">Status Pulse</span>
                                <span className={`font-mono font-bold ${isOnline ? 'text-emerald-400' : 'text-red-400'} text-right uppercase`}>{agent.status}</span>
                            </div>
                            <Separator className="bg-white/5" />
                            <div className="flex justify-between items-center text-[11px]">
                                <span className="font-bold text-slate-600 uppercase tracking-tighter">Last Seen</span>
                                <span className="font-mono font-bold text-indigo-400 text-right">
                                    {agent.lastHeartbeat ? formatDistanceToNow(new Date(agent.lastHeartbeat), { addSuffix: true }) : 'Never'}
                                </span>
                            </div>
                        </div>
                    </Card>
                </div>
            </div>
        </div>
    );
};

// Sub-components for cleaner structure
const MetricTile = ({ icon, label, value, subValue, subText, progress, glow }: {
    icon: React.ReactNode,
    label: string,
    value: string,
    subValue: string,
    subText?: string,
    progress?: number,
    glow?: string
}) => (
    <Card className={`bg-[#111318] border-white/5 transition-all duration-300 hover:border-white/20 group/tile relative overflow-hidden ${glow === 'purple' ? 'hover:shadow-[0_0_20px_rgba(168,85,247,0.1)]' : 'hover:shadow-[0_0_20px_rgba(99,102,241,0.1)]'
        }`}>
        <CardContent className="p-5 flex flex-col justify-between h-full gap-4">
            <div className="flex items-center justify-between">
                <div className="p-2 rounded-xl bg-white/[0.03] border border-white/5 group-hover/tile:scale-110 transition-transform">
                    {React.cloneElement(icon as React.ReactElement, { size: 16 })}
                </div>
                <span className="text-[9px] font-black uppercase tracking-[0.2em] text-slate-600">{label}</span>
            </div>

            <div className="space-y-1">
                <div className="text-2xl font-mono font-bold text-white tracking-tighter tabular-nums">{value}</div>
                <div className="flex items-center justify-between gap-2">
                    <span className="text-[10px] font-bold text-slate-600 truncate uppercase mt-1">{subValue}</span>
                    {subText && <span className="text-[9px] font-black text-slate-500 whitespace-nowrap">{subText}</span>}
                </div>
            </div>

            {progress !== undefined && (
                <div className="pt-1">
                    <Progress value={progress} className="h-1 bg-white/5" />
                </div>
            )}
        </CardContent>
    </Card>
);


function formatBytes(bytes: number, decimals = 1) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}
