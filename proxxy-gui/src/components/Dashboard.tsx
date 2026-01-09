import React, { useState, useEffect } from 'react';
import { AgentStatusCard } from './dashboard/AgentStatusCard';
import { TrafficSummaryCard } from './dashboard/TrafficSummaryCard';
import { SystemHealthCard } from './dashboard/SystemHealthCard';
import { QuickActionsCard } from './dashboard/QuickActionsCard';
import { RecentTrafficTable } from './dashboard/RecentTrafficTable';
import { SystemMetricsOverview } from './dashboard/SystemMetricsOverview';
import { LoadingSpinner } from './ui/LoadingSpinner';
import { Agent, HttpTransaction } from '../types/graphql';

// API service for orchestrator endpoints
const API_BASE_URL = 'http://localhost:9090'; // Orchestrator HTTP port

const apiService = {
  async fetchAgents(): Promise<Agent[]> {
    const response = await fetch(`${API_BASE_URL}/api/agents`);
    if (!response.ok) throw new Error(`Failed to fetch agents: ${response.statusText}`);
    const data = await response.json();
    return data.agents || [];
  },

  async fetchTraffic(): Promise<HttpTransaction[]> {
    const response = await fetch(`${API_BASE_URL}/api/traffic/recent`);
    if (!response.ok) throw new Error(`Failed to fetch traffic: ${response.statusText}`);
    const data = await response.json();
    return data.requests || [];
  },

  async fetchSystemHealth() {
    const response = await fetch(`${API_BASE_URL}/api/system/health`);
    if (!response.ok) throw new Error(`Failed to fetch system health: ${response.statusText}`);
    return response.json();
  },

  async startSystem() {
    const response = await fetch(`${API_BASE_URL}/api/system/start`, { method: 'POST' });
    if (!response.ok) throw new Error(`Failed to start system: ${response.statusText}`);
    return response.json();
  },

  async stopSystem() {
    const response = await fetch(`${API_BASE_URL}/api/system/stop`, { method: 'POST' });
    if (!response.ok) throw new Error(`Failed to stop system: ${response.statusText}`);
    return response.json();
  },

  async restartSystem() {
    const response = await fetch(`${API_BASE_URL}/api/system/restart`, { method: 'POST' });
    if (!response.ok) throw new Error(`Failed to restart system: ${response.statusText}`);
    return response.json();
  }
};

// Fallback mock data when orchestrator is not available
const mockAgents: Agent[] = [
  {
    id: 'agent-1',
    name: 'Agent US-East',
    hostname: 'proxy-us-east.example.com',
    status: 'Offline',
    version: '1.0.0',
    lastHeartbeat: new Date(Date.now() - 300000).toISOString(),
  },
  {
    id: 'agent-2',
    name: 'Agent EU-West',
    hostname: 'proxy-eu-west.example.com',
    status: 'Offline',
    version: '1.0.0',
    lastHeartbeat: new Date(Date.now() - 300000).toISOString(),
  },
  {
    id: 'agent-3',
    name: 'Agent Dev',
    hostname: 'proxy-dev.example.com',
    status: 'Offline',
    version: '0.9.0',
    lastHeartbeat: new Date(Date.now() - 600000).toISOString(),
  },
];

const mockTraffic: HttpTransaction[] = [];

export const Dashboard: React.FC = () => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [data, setData] = useState<{ agents: Agent[]; requests: HttpTransaction[] } | null>(null);
  const [isOrchestratorOnline, setIsOrchestratorOnline] = useState(false);

  // Fetch real data from orchestrator
  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        
        // Try to fetch from orchestrator
        const [agents, traffic] = await Promise.all([
          apiService.fetchAgents(),
          apiService.fetchTraffic()
        ]);
        
        setData({
          agents,
          requests: traffic,
        });
        setIsOrchestratorOnline(true);
        setError(null);
      } catch (err) {
        console.warn('Orchestrator not reachable, using offline mode:', err);
        // Use mock data when orchestrator is offline
        setData({
          agents: mockAgents,
          requests: mockTraffic,
        });
        setIsOrchestratorOnline(false);
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
    
    // Set up polling interval
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  }, []);

  const refetch = () => {
    setError(null);
    setLoading(true);
    // Trigger re-fetch
    setTimeout(async () => {
      try {
        const [agents, traffic] = await Promise.all([
          apiService.fetchAgents(),
          apiService.fetchTraffic()
        ]);
        
        setData({
          agents,
          requests: traffic,
        });
        setIsOrchestratorOnline(true);
        setError(null);
      } catch (err) {
        setData({
          agents: mockAgents,
          requests: mockTraffic,
        });
        setIsOrchestratorOnline(false);
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    }, 1000);
  };

  if (loading && !data) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  const agents = data?.agents || [];
  const recentTraffic = data?.requests || [];

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Dashboard</h1>
          <p className="text-white/60 mt-1">System overview and real-time monitoring</p>
        </div>
        <div className="flex items-center gap-4">
          {/* Connection Status */}
          <div className="flex items-center gap-2">
            <div className={`w-2 h-2 rounded-full ${isOrchestratorOnline ? 'bg-emerald-400 animate-pulse' : 'bg-red-400'}`}></div>
            <span className="text-xs text-white/60">
              {isOrchestratorOnline ? 'Orchestrator Online' : 'Orchestrator Offline'}
            </span>
          </div>
          
          {/* Error indicator */}
          {error && (
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 bg-amber-400 rounded-full animate-pulse"></div>
              <span className="text-xs text-amber-400">Connection Issues</span>
            </div>
          )}
        </div>
      </div>

      {/* Connection Error Banner */}
      {error && !isOrchestratorOnline && (
        <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-red-400 font-bold text-sm">Orchestrator Connection Failed</h3>
              <p className="text-red-300/80 text-xs mt-1">
                Cannot connect to orchestrator at {API_BASE_URL}. Showing cached/offline data.
              </p>
            </div>
            <button
              onClick={refetch}
              className="px-3 py-1 bg-red-500/20 border border-red-500/30 rounded text-red-400 text-xs font-bold hover:bg-red-500/30 transition-all"
            >
              Retry
            </button>
          </div>
        </div>
      )}

      {/* Summary Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <AgentStatusCard agents={agents} />
        <TrafficSummaryCard traffic={recentTraffic} />
        <SystemHealthCard isOnline={isOrchestratorOnline} />
        <QuickActionsCard 
          isOrchestratorOnline={isOrchestratorOnline} 
          onSystemAction={async (action: string) => {
            try {
              switch (action) {
                case 'start':
                  await apiService.startSystem();
                  break;
                case 'stop':
                  await apiService.stopSystem();
                  break;
                case 'restart':
                  await apiService.restartSystem();
                  break;
              }
              // Refresh data after system action
              refetch();
            } catch (err) {
              console.error(`Failed to ${action} system:`, err);
            }
          }}
        />
      </div>

      {/* Detailed Views Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <RecentTrafficTable traffic={recentTraffic} />
        <SystemMetricsOverview isOnline={isOrchestratorOnline} />
      </div>
    </div>
  );
};