import React, { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Server,
  Wifi,
  WifiOff,
  Terminal,
  Copy,
  Plus,
  Network,
  ShieldCheck,
  ArrowRight,
  Activity,
  Info
} from 'lucide-react';
import { Agent } from '../../types/graphql';
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface AgentStatusCardProps {
  agents: Agent[];
}

export const AgentStatusCard: React.FC<AgentStatusCardProps> = ({ agents }) => {
  const navigate = useNavigate();
  const [copied, setCopied] = useState<string | null>(null);

  const { onlineCount, offlineCount, totalCount, onlinePercentage } = useMemo(() => {
    const total = agents.length;
    const online = agents.filter(a => a.status?.toLowerCase() === 'online').length;
    const offline = total - online;
    const percent = total > 0 ? (online / total) * 100 : 0;

    return {
      onlineCount: online,
      offlineCount: offline,
      totalCount: total,
      onlinePercentage: percent
    };
  }, [agents]);

  const statusConfig = useMemo(() => {
    if (totalCount === 0) return {
      color: 'text-slate-500',
      indicator: 'bg-slate-500/50',
      glow: 'shadow-[0_-4px_12px_rgba(100,116,139,0.2)]',
      text: 'No Agents Detected'
    };
    if (onlineCount === totalCount) return {
      color: 'text-emerald-400',
      indicator: 'bg-emerald-500',
      glow: 'shadow-[0_-4px_12px_rgba(16,185,129,0.3)]',
      text: 'System Healthy'
    };
    if (onlineCount > totalCount * 0.8) return {
      color: 'text-amber-400',
      indicator: 'bg-amber-400',
      glow: 'shadow-[0_-4px_12px_rgba(251,191,36,0.2)]',
      text: 'Performance Degraded'
    };
    return {
      color: 'text-red-400',
      indicator: 'bg-red-400',
      glow: 'shadow-[0_-4px_12px_rgba(248,113,113,0.3)]',
      text: 'Critical Failure'
    };
  }, [totalCount, onlineCount]);

  const handleCopy = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopied(key);
    setTimeout(() => setCopied(null), 2000);
  };

  return (
    <Card className={`bg-[#0F1116] border-white/5 shadow-2xl h-full relative overflow-hidden group transition-all duration-500 hover:border-white/10`}>
      {/* Dynamic Health Accent */}
      <div className={`absolute top-0 left-0 w-full h-[2px] ${statusConfig.indicator} ${statusConfig.glow} z-20`} />

      {/* Background radial gradient */}
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(99,102,241,0.05),transparent_70%)] pointer-events-none" />

      <CardContent className="p-5 flex flex-col h-full justify-between relative z-10">

        {/* Header Section */}
        <div className="flex justify-between items-start mb-6">
          <div
            className="space-y-1 cursor-pointer group/header"
            onClick={() => navigate('/agents')}
          >
            <div className="flex items-center gap-2">
              <div className="p-1 rounded-md bg-white/[0.03] border border-white/5">
                <Server className="w-3.5 h-3.5 text-slate-400" />
              </div>
              <span className="text-[10px] font-black text-slate-500 uppercase tracking-[0.2em]">Node Operations</span>
            </div>
            <div className="flex items-baseline gap-2 pt-1">
              <span className="text-4xl font-mono font-bold text-white tracking-tighter tabular-nums drop-shadow-2xl">
                {totalCount}
              </span>
              <div className="flex flex-col">
                <span className="text-[10px] text-slate-500 font-black uppercase tracking-widest leading-none">Registered</span>
                <span className="text-[8px] text-slate-600 font-bold uppercase tracking-tight mt-0.5">Proxy Fleet</span>
              </div>
            </div>
          </div>

          <Dialog>
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <DialogTrigger asChild>
                    <button className="flex items-center gap-2 px-3 py-2 rounded-xl bg-indigo-500/10 hover:bg-indigo-500/20 border border-indigo-500/20 text-indigo-400 transition-all duration-300 hover:shadow-[0_0_20px_rgba(99,102,241,0.15)] group/btn relative overflow-hidden">
                      <Plus className="w-3.5 h-3.5 transition-transform group-hover/btn:rotate-90" />
                      <span className="text-[10px] font-black uppercase tracking-widest">Deploy</span>
                    </button>
                  </DialogTrigger>
                </TooltipTrigger>
                <TooltipContent side="left" className="bg-slate-950 border-white/10 text-[10px] font-black uppercase tracking-widest px-3 py-2">
                  Launch Remote Instance
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>

            <DialogContent className="sm:max-w-2xl bg-[#0B0D11] border-white/10 text-slate-200 p-0 overflow-hidden shadow-[0_0_50px_rgba(0,0,0,0.5)]">
              {/* Header inside Dialog */}
              <div className="p-8 bg-[#111318] border-b border-white/5 relative">
                <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-indigo-500/50 to-transparent" />
                <DialogHeader>
                  <div className="flex items-center gap-4">
                    <div className="p-3 bg-indigo-500/10 rounded-2xl border border-indigo-500/20 shadow-inner">
                      <Terminal className="w-6 h-6 text-indigo-400" />
                    </div>
                    <div>
                      <DialogTitle className="text-2xl font-black uppercase tracking-tighter text-white">
                        Agent Orchestration
                      </DialogTitle>
                      <p className="text-slate-500 text-xs font-bold uppercase tracking-widest mt-1 opacity-60">Deployment Matrix v4.0.1</p>
                    </div>
                  </div>
                </DialogHeader>
              </div>

              <div className="p-8 space-y-8 max-h-[70vh] overflow-y-auto custom-scrollbar">
                {/* 1. Quick Connect Command */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <h4 className="text-[11px] font-black text-indigo-400 uppercase tracking-[0.2em] flex items-center gap-2">
                      <Activity className="w-3 h-3" />
                      Primary Gateway
                    </h4>
                    <Badge variant="outline" className="text-[8px] border-emerald-500/20 text-emerald-500 bg-emerald-500/5">PRODUCTION READY</Badge>
                  </div>
                  <div className="relative group/cmd">
                    <pre className="p-5 bg-black/60 border border-white/10 rounded-2xl font-mono text-[11px] text-emerald-400/90 overflow-x-auto leading-relaxed shadow-inner">
                      <span className="opacity-40"># Initialize node-01 locally</span>{"\n"}
                      cargo run -p proxy-agent -- --name "Agent-Alpha" --orchestrator-url http://127.0.0.1:50051
                    </pre>
                    <button
                      onClick={() => handleCopy('cargo run -p proxy-agent -- --name "Agent-Alpha" --orchestrator-url http://127.0.0.1:50051', 'cmd1')}
                      className="absolute right-3 top-3 p-2.5 rounded-xl bg-white/[0.03] border border-white/5 hover:bg-white/10 text-slate-400 hover:text-white transition-all active:scale-95"
                    >
                      {copied === 'cmd1' ? <span className="text-[9px] text-emerald-400 font-black">COPIED</span> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>

                {/* 2. Isolated Context Command */}
                <div className="space-y-3">
                  <h4 className="text-[11px] font-black text-blue-400 uppercase tracking-[0.2em] flex items-center gap-2">
                    <Network className="w-3 h-3" />
                    Multi-Node Binding
                  </h4>
                  <div className="relative group/cmd">
                    <pre className="p-5 bg-black/60 border border-white/10 rounded-2xl font-mono text-[11px] text-blue-400/90 overflow-x-auto leading-relaxed shadow-inner">
                      <span className="opacity-40"># Run agent-02 on custom port</span>{"\n"}
                      cargo run -p proxy-agent -- --name "Agent-Beta" --admin-port 9092 --listen-port 9096
                    </pre>
                    <button
                      onClick={() => handleCopy('cargo run -p proxy-agent -- --name "Agent-Beta" --admin-port 9092 --listen-port 9096', 'cmd2')}
                      className="absolute right-3 top-3 p-2.5 rounded-xl bg-white/[0.03] border border-white/5 hover:bg-white/10 text-slate-400 hover:text-white transition-all active:scale-95"
                    >
                      {copied === 'cmd2' ? <span className="text-[9px] text-emerald-400 font-black">COPIED</span> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>

                {/* Integration Specs */}
                <div className="grid grid-cols-2 gap-4">
                  <div className="p-4 bg-white/[0.02] border border-white/5 rounded-2xl flex flex-col gap-3 group/info hover:bg-white/[0.04] transition-colors">
                    <div className="p-2 w-fit bg-emerald-500/10 rounded-lg border border-emerald-500/20">
                      <ShieldCheck className="w-4 h-4 text-emerald-400" />
                    </div>
                    <div>
                      <h5 className="text-[10px] font-black text-white uppercase tracking-widest mb-1">Unified Trust</h5>
                      <p className="text-[11px] text-slate-500 leading-relaxed">
                        Orchestrate thousands of nodes using a single root CA certificate for intercepting SSL traffic seamlessly.
                      </p>
                    </div>
                  </div>
                  <div className="p-4 bg-white/[0.02] border border-white/5 rounded-2xl flex flex-col gap-3 group/info hover:bg-white/[0.04] transition-colors">
                    <div className="p-2 w-fit bg-indigo-500/10 rounded-lg border border-indigo-500/20">
                      <Network className="w-4 h-4 text-indigo-400" />
                    </div>
                    <div>
                      <h5 className="text-[10px] font-black text-white uppercase tracking-widest mb-1">Isolated State</h5>
                      <p className="text-[11px] text-slate-500 leading-relaxed">
                        Each node maintains its own telemetry stream and health metrics via the <code className="text-indigo-300 font-mono">--admin-port</code>.
                      </p>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-4 p-4 rounded-2xl bg-amber-500/[0.03] border border-amber-500/10">
                  <div className="p-2 bg-amber-500/10 rounded-lg">
                    <Info className="w-4 h-4 text-amber-500" />
                  </div>
                  <p className="text-[10px] text-slate-400 font-medium leading-relaxed">
                    Ensure your system firewall allows inbound traffic on the orchestrator port (<code className="text-amber-300">50051</code>) to receive agent telemetry.
                  </p>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 gap-3 mb-6 relative">
          <div
            className="bg-white/[0.02] rounded-2xl p-4 border border-white/5 group/online transition-all hover:bg-white/[0.04] hover:border-emerald-500/30 cursor-pointer"
            onClick={() => navigate('/agents')}
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.6)] animate-pulse" />
                <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest">Active</span>
              </div>
              <Wifi className="w-3 h-3 text-emerald-500/50 group-hover/online:text-emerald-500 transition-colors" />
            </div>
            <div className="flex items-baseline gap-1">
              <span className="text-3xl font-mono font-bold text-emerald-400 tabular-nums">{onlineCount}</span>
              <span className="text-[10px] text-emerald-900 font-black uppercase tracking-tighter">Live</span>
            </div>
          </div>

          <div
            className="bg-white/[0.02] rounded-2xl p-4 border border-white/5 group/offline transition-all hover:bg-white/[0.04] hover:border-red-500/30 cursor-pointer"
            onClick={() => navigate('/agents')}
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 rounded-full bg-slate-700" />
                <span className="text-[10px] font-black text-slate-500 uppercase tracking-widest">Signal Lost</span>
              </div>
              <WifiOff className="w-3 h-3 text-slate-700/50 group-hover/offline:text-red-500/50 transition-colors" />
            </div>
            <div className="flex items-baseline gap-1">
              <span className="text-3xl font-mono font-bold text-slate-300 tabular-nums">{offlineCount}</span>
              <span className="text-[10px] text-slate-700 font-black uppercase tracking-tighter">Nodes</span>
            </div>
          </div>
        </div>

        {/* Fleet Health Progress */}
        <div className="space-y-3 pt-3 border-t border-white/5 mt-auto">
          <div className="flex justify-between items-center group/health">
            <div className="flex items-center gap-2">
              <span className={`text-[10px] font-black uppercase tracking-[0.2em] ${statusConfig.color}`}>
                {statusConfig.text}
              </span>
            </div>
            <div className="flex items-baseline gap-1">
              <span className="text-sm font-mono font-black text-white">{onlinePercentage.toFixed(0)}</span>
              <span className="text-[9px] font-bold text-slate-600 uppercase tracking-tighter">% Efficiency</span>
            </div>
          </div>
          <div className="relative h-2 w-full bg-white/[0.02] rounded-full p-[1px] border border-white/[0.02]">
            <div
              className={`h-full ${statusConfig.indicator} rounded-full transition-all duration-1000 ease-out relative group-hover:brightness-125`}
              style={{ width: `${onlinePercentage}%` }}
            >
              {/* Shine effect */}
              <div className="absolute inset-0 bg-gradient-to-r from-white/20 to-transparent rounded-full" />
            </div>
          </div>

          <div className="flex justify-between items-center text-[8px] font-black text-slate-600 uppercase tracking-widest pt-1">
            <div className="flex items-center gap-1">
              <div className="w-1 h-1 rounded-full bg-slate-800" />
              <span>Fleet Sync: Active</span>
            </div>
            <button
              className="flex items-center gap-1 hover:text-indigo-400 transition-colors"
              onClick={() => navigate('/agents')}
            >
              Full Network Report
              <ArrowRight className="w-2 h-2" />
            </button>
          </div>
        </div>

      </CardContent>
    </Card >
  );
};