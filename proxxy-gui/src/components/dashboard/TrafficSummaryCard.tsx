import React, { useMemo, useState, useEffect } from 'react';
import { Zap } from 'lucide-react';
import { HttpTransaction } from '../../types/graphql';

interface TrafficSummaryCardProps {
  traffic: HttpTransaction[];
}

export const TrafficSummaryCard: React.FC<TrafficSummaryCardProps> = ({ traffic }) => {
  // Veri akışını izlemek için state
  const [isFlowing, setIsFlowing] = useState(false);
  const [rps, setRps] = useState(0);

  const stats = useMemo(() => {
    const total = traffic.length;
    if (total === 0) return { methods: {}, statusGroups: {}, errorRate: 0, successRate: 0 };

    const methods: Record<string, number> = {};
    const statusGroups = { success: 0, redirect: 0, clientError: 0, serverError: 0 };

    traffic.forEach(tx => {
      // Metod Dağılımı
      const m = tx.method || 'OTHER';
      methods[m] = (methods[m] || 0) + 1;

      // Status Dağılımı
      const s = tx.status || 0;
      if (s >= 200 && s < 300) statusGroups.success++;
      else if (s >= 300 && s < 400) statusGroups.redirect++;
      else if (s >= 400 && s < 500) statusGroups.clientError++;
      else if (s >= 500) statusGroups.serverError++;
    });

    const errorCount = statusGroups.clientError + statusGroups.serverError;

    return {
      methods,
      statusGroups,
      total,
      errorRate: (errorCount / total) * 100,
      successRate: (statusGroups.success / total) * 100
    };
  }, [traffic]);

  // RPS ve Flowing durumunu heapla
  useEffect(() => {
    if (traffic.length > 0) {
      setIsFlowing(true);

      // Basit bir RPS hesabı (son saniyedeki değişim)
      // Gerçek bir RPS için backend'den veri gelmeli ama burada simüle ediyoruz
      const recentCount = traffic.filter(tx => {
        const ts = (tx as any).timestamp;
        const time = typeof ts === 'number' ? (ts > 1e12 ? ts : ts * 1000) : new Date(ts).getTime();
        return (Date.now() - time) < 1000;
      }).length;

      setRps(recentCount);

      const timeout = setTimeout(() => setIsFlowing(false), 3000);
      return () => clearTimeout(timeout);
    }
  }, [traffic]);

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-6 hover:border-white/10 transition-all group relative overflow-hidden">
      {/* Background Glow */}
      <div className={`absolute -top-10 -right-10 w-32 h-32 blur-[50px] rounded-full opacity-10 pointer-events-none transition-colors duration-1000 ${isFlowing ? 'bg-indigo-500' : 'bg-transparent'}`} />

      {/* Header */}
      <div className="flex items-start justify-between mb-6">
        <div className="p-3 rounded-lg bg-indigo-500/10 border border-indigo-500/20">
          <Zap className="h-6 w-6 text-indigo-400" />
        </div>
        <div className={`flex items-center gap-2 px-2.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-widest border transition-all duration-300 ${isFlowing
          ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
          : 'bg-white/5 border-white/5 text-white/30'
          }`}>
          {isFlowing && <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />}
          {isFlowing ? 'Receiving' : 'Idle'}
        </div>
      </div>

      {/* Main Metric */}
      <div className="space-y-4">
        <div>
          <h3 className="text-xs font-bold text-white/40 uppercase tracking-wider">Throughput</h3>
          <div className="flex items-baseline gap-2 mt-1">
            <span className="text-3xl font-bold text-white font-mono tracking-tight">
              {rps.toFixed(1)}
            </span>
            <span className="text-xs text-white/30 font-medium">REQ/SEC</span>
          </div>
        </div>

        {/* Status Breakdown Bar */}
        <div className="space-y-2">
          <div className="flex justify-between text-[10px] uppercase font-bold tracking-wider mb-1">
            <span className="text-white/40">Health Rate</span>
            <span className={stats.successRate > 90 ? 'text-emerald-400' : 'text-yellow-400'}>
              {stats.successRate.toFixed(1)}%
            </span>
          </div>
          <div className="w-full h-1.5 bg-white/5 rounded-full overflow-hidden flex">
            <div className="bg-emerald-500 h-full transition-all duration-500" style={{ width: `${stats.successRate}%` }} />
            <div className="bg-red-500/50 h-full transition-all duration-500" style={{ width: `${stats.errorRate}%` }} />
          </div>
        </div>

        {/* Method Tags */}
        <div className="flex flex-wrap gap-2 pt-2">
          {Object.entries(stats.methods).slice(0, 3).map(([method, count]) => (
            <div key={method} className="bg-white/5 border border-white/5 px-2 py-1 rounded flex items-center gap-2">
              <span className="text-[10px] font-bold text-white/40">{method}</span>
              <span className="text-[10px] font-mono font-bold text-white/80">{count}</span>
            </div>
          ))}
          {Object.keys(stats.methods).length > 3 && (
            <div className="bg-white/5 border border-white/5 px-2 py-1 rounded">
              <span className="text-[10px] font-bold text-white/30">+{Object.keys(stats.methods).length - 3}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};