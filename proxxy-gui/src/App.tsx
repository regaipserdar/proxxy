import { useQuery } from '@tanstack/react-query';
import {
  Activity, Server, Globe, Cpu,
  AlertCircle, RefreshCw, Code
} from 'lucide-react';

import { api } from './lib/api';
import { AgentsResponse, HealthStatus, MetricsResponse } from './types';
import { MetricCard } from './components/MetricCard';
import { AgentCard } from './components/AgentCard';
import { LogConsole } from './components/LogConsole';
import { TrafficTable } from './components/TrafficTable';
import { useState } from 'react';

export default function App() {
  const [showDebug, setShowDebug] = useState(false);

  // 1. Data Fetching (Polling her 2 saniyede bir)
  const { data: health } = useQuery<HealthStatus>({
    queryKey: ['health'],
    queryFn: () => api.get('/health/detailed').then(res => res.data),
    refetchInterval: 5000
  });

  const { data: agentsData, isLoading: agentsLoading } = useQuery<AgentsResponse>({
    queryKey: ['agents'],
    queryFn: () => api.get('/agents').then(res => res.data),
    refetchInterval: 2000
  });

  const { data: metrics } = useQuery<MetricsResponse>({
    queryKey: ['metrics'],
    queryFn: () => api.get('/metrics').then(res => res.data),
    refetchInterval: 5000
  });

  const endpoints = [
    { method: 'GET', path: '/health/detailed', desc: 'Orchestrator health status' },
    { method: 'GET', path: '/agents', desc: 'List all registered agents' },
    { method: 'GET', path: '/metrics', desc: 'Traffic metrics (real-time from DB)' },
    { method: 'GET', path: '/traffic', desc: 'Recent HTTP transactions' },
    { method: 'POST', path: '/graphql', desc: 'GraphQL endpoint' },
  ];

  return (
    <div className="flex flex-col h-screen bg-[#0A0E14] text-white overflow-hidden">

      {/* HEADER */}
      <header className="h-16 border-b border-white/5 bg-[#111318] flex items-center justify-between px-6">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 bg-[#9DCDE8]/10 rounded flex items-center justify-center border border-[#9DCDE8]/20">
            <Globe size={18} className="text-[#9DCDE8]" />
          </div>
          <div>
            <h1 className="font-bold text-lg">Orchestrator Control</h1>
            <div className="flex items-center gap-2 text-xs text-gray-400">
              <span className={`w-2 h-2 rounded-full ${health?.database_connected ? 'bg-emerald-500' : 'bg-red-500'}`} />
              {health?.database_connected ? 'Database Connected' : 'DB Disconnected'}
              <span className="text-gray-700">|</span>
              Uptime: {Math.floor((health?.uptime_seconds || 0) / 60)}m
            </div>
          </div>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowDebug(!showDebug)}
            className={`text-xs px-3 py-1.5 rounded flex items-center gap-2 transition-colors ${showDebug
                ? 'bg-[#9DCDE8]/20 text-[#9DCDE8] border border-[#9DCDE8]/30'
                : 'bg-white/5 hover:bg-white/10 text-white/60'
              }`}
          >
            <Code size={12} /> API Endpoints
          </button>
          <button className="text-xs bg-white/5 hover:bg-white/10 px-3 py-1.5 rounded text-white/60 flex items-center gap-2">
            <RefreshCw size={12} /> Sync
          </button>
        </div>
      </header>

      {/* DEBUG PANEL */}
      {showDebug && (
        <div className="bg-[#111318] border-b border-white/5 p-4">
          <div className="max-w-4xl">
            <h3 className="text-xs font-bold text-white/60 mb-3 uppercase tracking-wider">Available Endpoints</h3>
            <div className="space-y-1">
              {endpoints.map((ep, idx) => (
                <div key={idx} className="flex items-center gap-3 text-xs font-mono">
                  <span className={`px-2 py-0.5 rounded font-bold ${ep.method === 'GET' ? 'bg-emerald-500/10 text-emerald-400' : 'bg-blue-500/10 text-blue-400'
                    }`}>
                    {ep.method}
                  </span>
                  <span className="text-white/60">{api.defaults.baseURL}{ep.path}</span>
                  <span className="text-white/30">→ {ep.desc}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* DASHBOARD CONTENT */}
      <main className="flex-1 overflow-y-auto p-6">

        {/* Metrics Grid */}
        <div className="grid grid-cols-4 gap-4 mb-8">
          <MetricCard
            title="Total Agents"
            value={agentsData?.total_count || 0}
            sub={`${agentsData?.online_count || 0} Online`}
            icon={Server}
            color="emerald"
          />
          <MetricCard
            title="Total Traffic"
            value={metrics?.total_requests || 0}
            sub="Requests Captured"
            icon={Activity}
            color="blue"
          />
          <MetricCard
            title="Avg Latency"
            value={`${Math.round(metrics?.average_response_time_ms || 0)}ms`}
            icon={Cpu}
            color="yellow"
          />
          <MetricCard
            title="Error Rate"
            value={`${((metrics?.error_rate || 0) * 100).toFixed(1)}%`}
            icon={AlertCircle}
            color="red"
          />
        </div>

        {/* Agents Section */}
        <div className="mb-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-bold uppercase tracking-widest text-white/40">Registered Nodes</h2>
            <span className="text-[10px] text-white/20">Auto-refreshing (2s)</span>
          </div>

          {agentsLoading ? (
            <div className="text-white/20 text-sm animate-pulse">Loading agents...</div>
          ) : (agentsData?.agents?.length === 0) ? (
            <div className="p-8 border border-dashed border-white/10 rounded-xl text-center text-white/20 text-sm">
              No agents registered. Start an agent with `cargo run -p proxy-agent -- --name "Test-Agent"`
            </div>
          ) : (
            <div className="grid grid-cols-3 gap-4">
              {agentsData?.agents.map(agent => (
                <AgentCard key={agent.id} agent={agent} />
              ))}
            </div>
          )}
        </div>

        {/* Traffic Table */}
        <TrafficTable />

      </main>

      {/* Footer Console */}
      <LogConsole />
    </div>
  );
}
