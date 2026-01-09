import React from 'react';
import { Server, Wifi, WifiOff } from 'lucide-react';
import { Agent } from '../../types/graphql';

interface AgentStatusCardProps {
  agents: Agent[];
}

export const AgentStatusCard: React.FC<AgentStatusCardProps> = ({ agents }) => {
  const onlineCount = agents.filter(agent => agent.status === 'Online').length;
  const offlineCount = agents.length - onlineCount;
  const totalCount = agents.length;

  const getStatusColor = () => {
    if (totalCount === 0) return 'text-gray-400';
    if (onlineCount === totalCount) return 'text-emerald-400';
    if (onlineCount > totalCount / 2) return 'text-yellow-400';
    return 'text-red-400';
  };

  const getStatusText = () => {
    if (totalCount === 0) return 'No Agents';
    if (onlineCount === totalCount) return 'All Online';
    if (onlineCount === 0) return 'All Offline';
    return 'Partial';
  };

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-6 hover:border-white/10 transition-all group">
      <div className="flex items-start justify-between mb-4">
        <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
          <Server className="h-6 w-6 text-blue-400" />
        </div>
        <div className={`px-2 py-1 rounded-full bg-white/5 text-xs font-bold ${getStatusColor()} uppercase tracking-wider`}>
          {getStatusText()}
        </div>
      </div>

      <div className="space-y-3">
        <div>
          <h3 className="text-xs font-bold text-white/40 uppercase tracking-wider">Proxy Agents</h3>
          <div className="text-2xl font-bold text-white mt-1 font-mono">{totalCount}</div>
        </div>

        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Wifi className="h-4 w-4 text-emerald-400" />
              <span className="text-sm text-white/60">Online</span>
            </div>
            <span className="text-sm font-bold text-emerald-400">{onlineCount}</span>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <WifiOff className="h-4 w-4 text-red-400" />
              <span className="text-sm text-white/60">Offline</span>
            </div>
            <span className="text-sm font-bold text-red-400">{offlineCount}</span>
          </div>
        </div>

        {/* Status Bar */}
        {totalCount > 0 && (
          <div className="mt-4">
            <div className="w-full h-2 bg-white/5 rounded-full overflow-hidden">
              <div 
                className="h-full bg-gradient-to-r from-emerald-400 to-emerald-500 transition-all duration-500"
                style={{ width: `${(onlineCount / totalCount) * 100}%` }}
              />
            </div>
            <div className="flex justify-between text-xs text-white/40 mt-1">
              <span>0%</span>
              <span>100%</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};