import React from 'react';
import { useQuery, useSubscription } from '@apollo/client';
import { Cpu, Database, Network, HardDrive, Loader2, BarChart3 } from 'lucide-react';
import { GET_CURRENT_SYSTEM_METRICS, SYSTEM_METRICS_UPDATES, GET_AGENTS } from '../../graphql/operations';
import { Card, CardContent } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";

interface SystemMetricsOverviewProps {
  isOnline?: boolean;
  agentId?: string;
}

export const SystemMetricsOverview: React.FC<SystemMetricsOverviewProps> = ({
  isOnline = false,
  agentId = 'orchestrator'
}) => {
  const isAgent = agentId !== 'orchestrator';

  // 1. Orchestrator Metrics
  const { data: orchData, loading: orchLoading } = useQuery(GET_CURRENT_SYSTEM_METRICS, {
    variables: { agentId },
    skip: !isOnline || isAgent,
    pollInterval: 15000,
  });

  const { data: orchSubData } = useSubscription(SYSTEM_METRICS_UPDATES, {
    variables: { agentId },
    skip: !isOnline || isAgent,
  });

  // 2. Agent Metrics
  const { data: agentsData, loading: agentsLoading } = useQuery(GET_AGENTS, {
    skip: !isOnline || !isAgent,
    pollInterval: 5000,
  });

  // Determine source
  let latestMetrics: any = null;
  let cpuVal = 0;
  let memVal = 0; // %

  if (isAgent) {
    const agent = agentsData?.agents?.find((a: any) => a.id === agentId);
    if (agent) {
      latestMetrics = {
        agentId: agent.id,
        cpuUsagePercent: agent.cpuUsage,
        memoryUsageMb: agent.memoryUsageMb,
        // Missing network/disk in heartbeat
      };
      cpuVal = agent.cpuUsage || 0;
      // We don't have total memory, so we can't calculate % accurately.
      // Default to 0 to avoid misleading 100%.
      memVal = 0;
    }
  } else {
    latestMetrics = orchSubData?.systemMetricsUpdates || orchData?.currentSystemMetrics;
    if (latestMetrics) {
      cpuVal = latestMetrics.cpuUsagePercent || 0;
      memVal = latestMetrics.memoryUsedBytes && latestMetrics.memoryTotalBytes
        ? (parseInt(latestMetrics.memoryUsedBytes, 10) / parseInt(latestMetrics.memoryTotalBytes, 10)) * 100
        : 0;
    }
  }

  const getUsageColor = (percentage: number) => {
    if (!isOnline) return 'text-muted-foreground';
    if (percentage > 80) return 'text-red-500';
    if (percentage > 60) return 'text-yellow-500';
    return 'text-indigo-400';
  };

  const metrics = [
    { id: 'cpu', label: 'CPU', icon: Cpu, value: cpuVal, color: 'text-indigo-400' },
    { id: 'mem', label: 'RAM', icon: Database, value: memVal, color: 'text-emerald-400' },
  ];

  const loadingState = isAgent ? agentsLoading : orchLoading;
  const hasData = !!latestMetrics;

  return (
    <Card className="bg-[#111318] border-white/5 hover:border-indigo-500/40 transition-all group overflow-hidden shadow-2xl h-full">
      <CardContent className="p-4 flex flex-col justify-between h-full gap-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <BarChart3 className="w-3.5 h-3.5 text-muted-foreground/50" />
            <span className="text-[10px] font-black text-muted-foreground/50 uppercase tracking-[0.2em]">Resources</span>
          </div>
          {!isOnline ? (
            <span className="text-[8px] font-black text-red-500/50 uppercase tracking-widest">OFFLINE</span>
          ) : loadingState && !hasData ? (
            <Loader2 className="w-3 h-3 animate-spin text-primary" />
          ) : null}
        </div>

        <div className="space-y-3">
          {metrics.map((m) => (
            <div key={m.id} className="space-y-1.5">
              <div className="flex justify-between items-center text-[9px] font-bold uppercase tracking-widest">
                <div className="flex items-center gap-1.5 grayscale group-hover:grayscale-0 transition-all">
                  <m.icon className={`w-3 h-3 ${m.color}`} />
                  <span className="text-muted-foreground">{m.label}</span>
                </div>
                <span className={`font-mono ${getUsageColor(m.value)}`}>{m.value.toFixed(1)}%</span>
              </div>
              <Progress value={m.value} className="h-1 bg-white/5" />
            </div>
          ))}
        </div>

        <div className="grid grid-cols-2 gap-2 mt-1">
          <div className="bg-white/[0.02] border border-white/5 rounded-lg p-1.5 flex flex-col items-center justify-center gap-0.5">
            <Network className="w-2.5 h-2.5 text-purple-400/50" />
            <span className="text-[8px] font-mono text-slate-500 truncate w-full text-center">
              {latestMetrics?.networkRxBytesPerSec ? 'DATA_RX' : (isAgent ? 'N/A' : 'NET_IDLE')}
            </span>
          </div>
          <div className="bg-white/[0.02] border border-white/5 rounded-lg p-1.5 flex flex-col items-center justify-center gap-0.5">
            <HardDrive className="w-2.5 h-2.5 text-yellow-400/50" />
            <span className="text-[8px] font-mono text-slate-500 truncate w-full text-center">
              {latestMetrics?.diskReadBytesPerSec ? 'IO_ACTIVE' : (isAgent ? 'N/A' : 'DSK_IDLE')}
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};
