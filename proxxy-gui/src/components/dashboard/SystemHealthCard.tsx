import React from 'react';
import { useQuery, useSubscription } from '@apollo/client';
import { Cpu, Database, Zap, Activity } from 'lucide-react';
import { GET_CURRENT_SYSTEM_METRICS, SYSTEM_METRICS_UPDATES } from '../../graphql/operations';
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface SystemHealthCardProps {
  isOnline?: boolean;
  agentId?: string;
}

export const SystemHealthCard: React.FC<SystemHealthCardProps> = ({
  isOnline = false,
  agentId = 'orchestrator'
}) => {
  const { data, loading, error } = useQuery(GET_CURRENT_SYSTEM_METRICS, {
    variables: { agentId },
    skip: !isOnline,
    pollInterval: 10000,
  });

  const { data: subscriptionData } = useSubscription(SYSTEM_METRICS_UPDATES, {
    variables: { agentId },
    skip: !isOnline,
  });

  const metrics = subscriptionData?.systemMetricsUpdates || data?.currentSystemMetrics;

  const getHealthStatus = () => {
    if (!isOnline) return {
      status: 'Offline',
      color: 'text-red-500',
      badge: 'bg-red-500/10 text-red-500 border-red-500/20'
    };
    if (loading || error || !metrics) return {
      status: 'Unknown',
      color: 'text-muted-foreground',
      badge: 'bg-muted border-border'
    };

    const cpuUsage = metrics.cpuUsagePercent || 0;
    const memoryUsage = metrics.memoryUsedBytes && metrics.memoryTotalBytes
      ? (parseInt(metrics.memoryUsedBytes, 10) / parseInt(metrics.memoryTotalBytes, 10)) * 100
      : 0;

    if (cpuUsage > 80 || memoryUsage > 90) {
      return {
        status: 'Critical',
        color: 'text-red-500',
        badge: 'bg-red-500/10 text-red-500 border-red-500/20 shadow-[0_0_15px_rgba(239,68,68,0.1)]'
      };
    } else if (cpuUsage > 60 || memoryUsage > 75) {
      return {
        status: 'Unstable',
        color: 'text-yellow-500',
        badge: 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20'
      };
    } else {
      return {
        status: 'Healthy',
        color: 'text-emerald-500',
        badge: 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20'
      };
    }
  };

  const healthStatus = getHealthStatus();

  const formatBytes = (bytes: string | undefined) => {
    if (!bytes) return '0 B';
    const num = parseInt(bytes, 10);
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let unitIndex = 0;
    let value = num;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }

    return `${value.toFixed(1)} ${units[unitIndex]}`;
  };

  return (
    <Card className="bg-[#111318] border-white/5 hover:border-indigo-500/40 transition-all group overflow-hidden shadow-2xl h-full">
      <CardContent className="p-4 relative">
        <div className={`absolute -right-4 -top-4 w-24 h-24 blur-2xl rounded-full opacity-10 group-hover:opacity-20 transition-opacity pointer-events-none ${isOnline ? 'bg-indigo-500/20' : 'bg-transparent'}`} />

        <div className="flex items-center justify-between mb-4 relative z-10">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-lg ${isOnline ? 'bg-indigo-500/10 border-indigo-500/20 shadow-inner' : 'bg-white/5 border-white/10'}`}>
              <Activity className={`h-4 w-4 ${isOnline ? 'text-indigo-400' : 'text-slate-500'}`} />
            </div>
            <div>
              <p className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest leading-none">System Health</p>
              <div className={`text-[10px] font-black mt-1 uppercase tracking-widest ${healthStatus.color}`}>
                {healthStatus.status}
              </div>
            </div>
          </div>

          <Badge variant="outline" className={`h-6 rounded-md font-bold uppercase tracking-[0.2em] text-[8px] bg-black/40 border-white/5`}>
            {agentId === 'orchestrator' ? 'CORE' : 'AGENT'}
          </Badge>
        </div>

        <div className="grid grid-cols-1 gap-2 relative z-10">
          <div className="flex items-center justify-between bg-white/[0.02] border border-white/5 px-2.5 py-1.5 rounded-xl transition-all group-hover:bg-white/[0.04]">
            <div className="flex items-center gap-2.5">
              <Cpu className="h-3.5 w-3.5 text-blue-500" />
              <span className="text-[10px] text-muted-foreground font-black uppercase tracking-widest">Compute</span>
            </div>
            <span className="text-xs font-bold text-foreground font-mono">
              {metrics?.cpuUsagePercent?.toFixed(1) || 0}%
            </span>
          </div>

          <div className="flex items-center justify-between bg-white/[0.02] border border-white/5 px-2.5 py-1.5 rounded-xl transition-all group-hover:bg-white/[0.04]">
            <div className="flex items-center gap-2.5">
              <Database className="h-3.5 w-3.5 text-emerald-500" />
              <span className="text-[10px] text-muted-foreground font-black uppercase tracking-widest">Memory</span>
            </div>
            <span className="text-xs font-bold text-foreground font-mono uppercase tracking-tight">
              {formatBytes(metrics?.memoryUsedBytes)}
            </span>
          </div>

          <div className="flex items-center justify-between bg-white/[0.02] border border-white/5 px-2.5 py-1.5 rounded-xl transition-all group-hover:bg-white/[0.04]">
            <div className="flex items-center gap-2.5">
              <Zap className="h-3.5 w-3.5 text-yellow-500" />
              <span className="text-[10px] text-muted-foreground font-black uppercase tracking-widest">Uptime</span>
            </div>
            <span className="text-xs font-bold text-foreground font-mono">
              {metrics?.processUptimeSeconds ?
                `${Math.floor(metrics.processUptimeSeconds / 3600)}h ${Math.floor((metrics.processUptimeSeconds % 3600) / 60)}m` :
                '0h 0m'
              }
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};