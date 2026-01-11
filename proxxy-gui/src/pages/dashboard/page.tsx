import { useQuery, useSubscription, useMutation } from '@apollo/client';
import { useNavigate } from 'react-router-dom';
import { AgentStatusCard } from '@/components/dashboard/AgentStatusCard';
import { TrafficSummaryCard } from '@/components/dashboard/TrafficSummaryCard';
import { SystemHealthCard } from '@/components/dashboard/SystemHealthCard';
import { QuickActionsCard } from '@/components/dashboard/QuickActionsCard';
import { RecentTrafficTable } from '@/components/dashboard/RecentTrafficTable';
import { SystemMetricsOverview } from '@/components/dashboard/SystemMetricsOverview';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { LayoutDashboard, Activity } from 'lucide-react';
import { GET_AGENTS, GET_PROJECTS, TRAFFIC_UPDATES, GET_HTTP_TRANSACTIONS, UNLOAD_PROJECT } from '@/graphql/operations';

// Shadcn UI Components
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

export const DashboardPage: React.FC = () => {
  const navigate = useNavigate();
  // 1. Proje Bilgisi
  const { data: projectsData } = useQuery(GET_PROJECTS, {
    pollInterval: 5000,
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

  // 2. Ajan Bilgisi
  const { data: agentsData, loading: agentsLoading } = useQuery(GET_AGENTS, {
    pollInterval: 5000,
    fetchPolicy: 'cache-and-network'
  });

  // 3. Trafik Verisi
  const { data: trafficData, loading: trafficLoading, error: trafficError } = useQuery(GET_HTTP_TRANSACTIONS, {
    fetchPolicy: 'network-only',
    variables: { limit: 50 }
  });

  // 4. Canlı Trafik
  useSubscription(TRAFFIC_UPDATES);

  const isLoading = agentsLoading || trafficLoading;
  const isOnline = !trafficError;
  const activeProject = projectsData?.projects?.find((p: any) => p.isActive);
  const agents = agentsData?.agents || [];
  const traffic = trafficData?.requests || [];

  if (isLoading && !trafficData) {
    return (
      <div className="flex h-full items-center justify-center bg-background">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="min-h-full bg-background/95 text-foreground p-6 space-y-8 animate-in fade-in duration-500">

      {/* COMPACT HEADER */}
      <header className="flex flex-col md:flex-row md:items-center justify-between gap-6 pb-2">
        <div className="space-y-1">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 rounded-lg border border-primary/20">
              <LayoutDashboard className="w-5 h-5 text-primary" />
            </div>
            <h1 className="text-3xl font-extrabold tracking-tight">Control Center</h1>
          </div>
          <p className="text-muted-foreground text-xs font-medium ml-1 uppercase tracking-widest">Real-time system pulse</p>
        </div>

        <div className="flex items-center gap-4">
          <Badge variant="outline" className={`h-8 px-4 rounded-xl text-[10px] font-black gap-2.5 bg-white/[0.02] border-white/5 shadow-inner ${isOnline ? 'text-emerald-500 border-emerald-500/20' : 'text-red-500 border-red-500/20'}`}>
            <div className={`w-1.5 h-1.5 rounded-full ${isOnline ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]' : 'bg-red-500'}`} />
            {isOnline ? 'Orchestrator live' : 'Orchestrator offline'}
          </Badge>
        </div>
      </header>

      {/* COMPACT KPI GRID (Now includes Resources) */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
        <div className="h-full"><AgentStatusCard agents={agents} /></div>
        <div className="h-full"><TrafficSummaryCard traffic={traffic} /></div>
        <div className="h-full"><SystemHealthCard isOnline={isOnline} /></div>
        <div className="h-full"><SystemMetricsOverview isOnline={isOnline} /></div>
        <div className="h-full">
          <QuickActionsCard
            isOrchestratorOnline={isOnline}
            activeProjectName={activeProject?.name}
            onSwitchProject={handleSwitchProject}
            onSystemAction={async (action) => { console.log("Action:", action); }}
          />
        </div>
      </div>

      {/* LOWER CONTENT: RECENT INTERCEPTS (Highly compact) */}
      <Card className="bg-[#111318] border-white/5 overflow-hidden shadow-2xl flex flex-col">
        <CardHeader className="flex flex-row items-center justify-between p-4 py-3 border-b border-white/5 bg-white/[0.01]">
          <div className="flex items-center gap-3">
            <Activity className="w-4 h-4 text-primary" />
            <CardTitle className="text-xs font-black uppercase tracking-widest">Recent Intercepts</CardTitle>
          </div>
          <Button variant="ghost" size="sm" onClick={() => navigate('/proxy')} className="h-7 px-3 text-[9px] font-black text-primary hover:text-primary hover:bg-primary/10 tracking-widest uppercase">
            Full Log →
          </Button>
        </CardHeader>
        <CardContent className="p-0">
          {/* Sadece 5 satır civarı gösterecek şekilde yüksekliği sınırlıyoruz */}
          <div className="max-h-[320px] overflow-hidden">
            <RecentTrafficTable traffic={traffic} isConnected={isOnline} />
          </div>
        </CardContent>
      </Card>

    </div>
  );
};