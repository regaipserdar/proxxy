import React, { useMemo } from 'react';
import { Server, Wifi, WifiOff, Activity } from 'lucide-react';
import { Agent } from '../../types/graphql';

interface AgentStatusCardProps {
  agents: Agent[];
}

export const AgentStatusCard: React.FC<AgentStatusCardProps> = ({ agents }) => {
  // Performans: Her render'da tekrar hesaplamamak için useMemo kullanıyoruz
  const { onlineCount, offlineCount, totalCount, onlinePercentage } = useMemo(() => {
    const total = agents.length;
    // Status kontrolünü case-insensitive yapıyoruz (online/Online/ONLINE)
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

  // Durum Rengi ve Metni
  const statusConfig = useMemo(() => {
    if (totalCount === 0) return { color: 'text-gray-500', bg: 'bg-gray-500/10', border: 'border-gray-500/20', text: 'No Agents' };
    if (onlineCount === totalCount) return { color: 'text-emerald-400', bg: 'bg-emerald-500/10', border: 'border-emerald-500/20', text: 'Healthy' };
    if (onlineCount > totalCount * 0.8) return { color: 'text-yellow-400', bg: 'bg-yellow-500/10', border: 'border-yellow-500/20', text: 'Degraded' };
    return { color: 'text-red-400', bg: 'bg-red-500/10', border: 'border-red-500/20', text: 'Critical' };
  }, [totalCount, onlineCount]);

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-6 hover:border-white/10 transition-all group relative overflow-hidden">

      {/* Arka plan için hafif gradient efekti */}
      <div className={`absolute top-0 right-0 w-32 h-32 ${statusConfig.bg} blur-[60px] rounded-full -mr-10 -mt-10 opacity-20 pointer-events-none transition-colors duration-500`} />

      {/* Header */}
      <div className="flex items-start justify-between mb-6 relative">
        <div className={`p-3 rounded-lg ${statusConfig.bg} ${statusConfig.border} border transition-colors duration-300`}>
          <Server className={`h-6 w-6 ${statusConfig.color}`} />
        </div>

        <div className="flex flex-col items-end">
          <div className={`flex items-center gap-2 px-2.5 py-1 rounded-full bg-white/5 border border-white/5 text-xs font-bold uppercase tracking-wider ${statusConfig.color}`}>
            {/* Canlılık hissi veren nokta */}
            {totalCount > 0 && (
              <span className="relative flex h-2 w-2">
                <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${statusConfig.color.replace('text-', 'bg-')}`}></span>
                <span className={`relative inline-flex rounded-full h-2 w-2 ${statusConfig.color.replace('text-', 'bg-')}`}></span>
              </span>
            )}
            {statusConfig.text}
          </div>
        </div>
      </div>

      {/* Main Stats */}
      <div className="space-y-4 relative">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <h3 className="text-xs font-bold text-white/40 uppercase tracking-wider">Total Agents</h3>
            {totalCount > 0 && <Activity className="h-3 w-3 text-white/20" />}
          </div>
          <div className="text-3xl font-bold text-white font-mono tracking-tight">
            {totalCount}
          </div>
        </div>

        {/* Details Row */}
        <div className="grid grid-cols-2 gap-3">
          <div className="bg-white/[0.02] rounded-lg p-2.5 border border-white/5 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Wifi className="h-4 w-4 text-emerald-400" />
              <span className="text-xs text-white/60">Online</span>
            </div>
            <span className="text-sm font-bold text-emerald-400 font-mono">{onlineCount}</span>
          </div>

          <div className="bg-white/[0.02] rounded-lg p-2.5 border border-white/5 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <WifiOff className="h-4 w-4 text-red-400" />
              <span className="text-xs text-white/60">Offline</span>
            </div>
            <span className="text-sm font-bold text-red-400 font-mono">{offlineCount}</span>
          </div>
        </div>

        {/* Stacked Progress Bar */}
        {totalCount > 0 && (
          <div className="mt-2">
            <div className="w-full h-1.5 bg-white/5 rounded-full overflow-hidden flex">
              {/* Online Part */}
              <div
                className="h-full bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.5)] transition-all duration-500 ease-out"
                style={{ width: `${onlinePercentage}%` }}
              />
              {/* Offline Part (Otomatik olarak kalanı doldurur ama görsel olarak kırmızı ekleyebiliriz) */}
              <div
                className="h-full bg-red-500/50 transition-all duration-500 ease-out"
                style={{ width: `${100 - onlinePercentage}%` }}
              />
            </div>
            <div className="flex justify-between text-[10px] text-white/30 mt-1.5 font-mono">
              <span>Availability</span>
              <span>{onlinePercentage.toFixed(1)}%</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};