import { useQuery, useSubscription, useMutation } from '@apollo/client';
import { useNavigate } from 'react-router-dom';
import { AgentStatusCard } from '@/components/dashboard/AgentStatusCard';
import { TrafficSummaryCard } from '@/components/dashboard/TrafficSummaryCard';
import { SystemHealthCard } from '@/components/dashboard/SystemHealthCard';
import { QuickActionsCard } from '@/components/dashboard/QuickActionsCard';
import { RecentTrafficTable } from '@/components/dashboard/RecentTrafficTable';
import { SystemMetricsOverview } from '@/components/dashboard/SystemMetricsOverview';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Database, ShieldCheck, Activity, LogOut } from 'lucide-react';
import { GET_AGENTS, GET_PROJECTS, TRAFFIC_UPDATES, GET_HTTP_TRANSACTIONS, UNLOAD_PROJECT } from '@/graphql/operations';

export const DashboardPage: React.FC = () => {
  const navigate = useNavigate();
  // 1. Proje Bilgisi (Active Project derived from list)
  const { data: projectsData } = useQuery(GET_PROJECTS, {
    pollInterval: 5000, // Sync with other tabs
    fetchPolicy: 'cache-first'
  });

  const [unloadProject] = useMutation(UNLOAD_PROJECT);

  const handleSwitchProject = async () => {
    try {
      await unloadProject();
      navigate('/projects');
    } catch (err) {
      console.error("Failed to unload project", err);
    }
  };

  // 2. Ajan Bilgisi (Polling ile güncel tutulur)
  const { data: agentsData, loading: agentsLoading } = useQuery(GET_AGENTS, {
    pollInterval: 5000,
    fetchPolicy: 'cache-and-network'
  });

  // 3. Trafik Verisi (Initial Load)
  const { data: trafficData, loading: trafficLoading, error: trafficError } = useQuery(GET_HTTP_TRANSACTIONS, {
    fetchPolicy: 'network-only',
    variables: { limit: 50 }
  });

  // 4. Canlı Trafik (Subscription)
  useSubscription(TRAFFIC_UPDATES, {
    onData: () => {
      // Apollo Cache otomatik güncellenir (requestId sayesinde), 
      // ama liste güncellemesi için cache merge stratejisi client.ts içinde olmalı.
    }
  });

  const isLoading = agentsLoading || trafficLoading;
  const isOnline = !trafficError;
  const activeProject = projectsData?.projects?.find((p: any) => p.isActive);
  const agents = agentsData?.agents || [];
  const traffic = trafficData?.requests || [];

  if (isLoading && !trafficData) {
    return (
      <div className="flex h-full items-center justify-center">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6 w-full min-h-full dotted-bg text-slate-200">

      {/* HEADER SECTION */}
      <div data-tauri-drag-region className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-white/5 pb-6">
        <div>
          <h1 className="text-2xl font-bold text-white flex items-center gap-3">
            <Activity className="w-6 h-6 text-emerald-400" />
            Dashboard
          </h1>
          <p className="text-white/60 text-sm mt-1">Real-time system overview and monitoring</p>
        </div>

        {/* PROJECT CONTEXT BADGE */}
        <div className="flex items-center gap-4">
          <div className="flex flex-col items-end">
            <div className="text-[10px] uppercase font-bold text-slate-500 tracking-wider mb-0.5">
              Current Workspace
            </div>
            {activeProject ? (
              <div className="flex flex-col items-end">
                <div className="flex items-center gap-2 bg-emerald-500/10 border border-emerald-500/20 px-3 py-1.5 rounded-lg text-emerald-400">
                  <Database className="w-4 h-4" />
                  <span className="font-mono text-sm font-bold">{activeProject.name}</span>
                </div>
                <div className="text-[10px] text-slate-500 mt-1 font-mono">
                  Loaded project '<span className="text-emerald-500/70">{activeProject.name}</span>' from <span className="text-slate-400">{activeProject.path}/proxxy.db</span>
                </div>
              </div>
            ) : (
              <div className="flex items-center gap-2 bg-red-500/10 border border-red-500/20 px-3 py-1.5 rounded-lg text-red-400">
                <ShieldCheck className="w-4 h-4" />
                <span className="font-mono text-sm font-bold">NO ACTIVE PROJECT</span>
              </div>
            )}
          </div>

          {/* Switch Project Button */}
          <button
            onClick={handleSwitchProject}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-slate-400 hover:bg-white/10 hover:text-white transition-all group"
            title="Switch Workspace"
          >
            <LogOut className="w-4 h-4 group-hover:-translate-x-0.5 transition-transform" />
            <span className="text-xs font-bold uppercase tracking-wider">Switch</span>
          </button>

          {/* Connection Indicator */}
          <div className="h-8 w-[1px] bg-white/10 mx-2 hidden md:block"></div>

          <div className="flex flex-col items-end">
            <div className="text-[10px] uppercase font-bold text-slate-500 tracking-wider mb-0.5">
              System Status
            </div>
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${isOnline ? 'bg-emerald-400 animate-pulse' : 'bg-red-500'}`} />
              <span className={`text-xs font-bold ${isOnline ? 'text-emerald-400' : 'text-red-500'}`}>
                {isOnline ? 'ORCHESTRATOR ONLINE' : 'DISCONNECTED'}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* ERROR BANNER */}
      {!isOnline && (
        <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4 flex items-center gap-3">
          <div className="p-2 bg-red-500/20 rounded-full">
            <Activity className="w-5 h-5 text-red-500" />
          </div>
          <div>
            <h3 className="text-red-400 font-bold text-sm">Connection Lost</h3>
            <p className="text-red-300/60 text-xs">
              Unable to reach Orchestrator. Real-time updates are paused.
            </p>
          </div>
        </div>
      )}

      {/* KPI CARDS GRID */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <AgentStatusCard agents={agents} />
        <TrafficSummaryCard traffic={traffic} />
        <SystemHealthCard isOnline={isOnline} />
        <QuickActionsCard isOrchestratorOnline={isOnline} />
      </div>

      {/* MAIN CONTENT GRID */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6 h-[600px]">
        {/* TRAFFIC TABLE (2/3 Width) */}
        <div className="xl:col-span-2 h-full">
          <RecentTrafficTable traffic={traffic} isConnected={isOnline} />
        </div>

        {/* METRICS (1/3 Width) */}
        <div className="xl:col-span-1 h-full">
          <SystemMetricsOverview isOnline={isOnline} />
        </div>
      </div>
    </div>
  );
};