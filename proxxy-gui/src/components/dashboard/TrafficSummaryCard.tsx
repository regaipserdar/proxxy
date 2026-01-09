import React from 'react';
import { Activity, TrendingUp, AlertCircle } from 'lucide-react';
import { HttpTransaction } from '../../types/graphql';

interface TrafficSummaryCardProps {
  traffic: HttpTransaction[];
}

export const TrafficSummaryCard: React.FC<TrafficSummaryCardProps> = ({ traffic }) => {
  const totalRequests = traffic.length;
  
  // Calculate method breakdown
  const methodCounts = traffic.reduce((acc, transaction) => {
    const method = transaction.method || 'UNKNOWN';
    acc[method] = (acc[method] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  // Calculate status code breakdown
  const statusCounts = traffic.reduce((acc, transaction) => {
    if (transaction.status) {
      const statusClass = Math.floor(transaction.status / 100);
      const key = `${statusClass}xx`;
      acc[key] = (acc[key] || 0) + 1;
    }
    return acc;
  }, {} as Record<string, number>);

  const errorCount = (statusCounts['4xx'] || 0) + (statusCounts['5xx'] || 0);
  const errorRate = totalRequests > 0 ? (errorCount / totalRequests) * 100 : 0;

  const topMethod = Object.entries(methodCounts).sort(([,a], [,b]) => b - a)[0];

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-6 hover:border-white/10 transition-all group">
      <div className="flex items-start justify-between mb-4">
        <div className="p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
          <Activity className="h-6 w-6 text-emerald-400" />
        </div>
        <div className="px-2 py-1 rounded-full bg-white/5 text-xs font-bold text-emerald-400 uppercase tracking-wider">
          Live
        </div>
      </div>

      <div className="space-y-3">
        <div>
          <h3 className="text-xs font-bold text-white/40 uppercase tracking-wider">Recent Traffic</h3>
          <div className="text-2xl font-bold text-white mt-1 font-mono">{totalRequests}</div>
        </div>

        <div className="space-y-2">
          {topMethod && (
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <TrendingUp className="h-4 w-4 text-blue-400" />
                <span className="text-sm text-white/60">Top Method</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs font-mono bg-blue-500/10 text-blue-400 px-2 py-1 rounded">
                  {topMethod[0]}
                </span>
                <span className="text-sm font-bold text-white">{topMethod[1]}</span>
              </div>
            </div>
          )}

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <AlertCircle className="h-4 w-4 text-red-400" />
              <span className="text-sm text-white/60">Error Rate</span>
            </div>
            <span className={`text-sm font-bold ${errorRate > 5 ? 'text-red-400' : errorRate > 1 ? 'text-yellow-400' : 'text-emerald-400'}`}>
              {errorRate.toFixed(1)}%
            </span>
          </div>
        </div>

        {/* Method Distribution */}
        {Object.keys(methodCounts).length > 0 && (
          <div className="mt-4">
            <div className="text-xs text-white/40 mb-2">Method Distribution</div>
            <div className="flex gap-1 h-2 rounded-full overflow-hidden bg-white/5">
              {Object.entries(methodCounts).map(([method, count], index) => {
                const percentage = (count / totalRequests) * 100;
                const colors = ['bg-blue-400', 'bg-emerald-400', 'bg-yellow-400', 'bg-purple-400', 'bg-red-400'];
                return (
                  <div
                    key={method}
                    className={`${colors[index % colors.length]} transition-all duration-500`}
                    style={{ width: `${percentage}%` }}
                    title={`${method}: ${count} (${percentage.toFixed(1)}%)`}
                  />
                );
              })}
            </div>
            <div className="flex flex-wrap gap-2 mt-2">
              {Object.entries(methodCounts).slice(0, 3).map(([method, count], index) => {
                const colors = ['text-blue-400', 'text-emerald-400', 'text-yellow-400'];
                return (
                  <div key={method} className="flex items-center gap-1">
                    <div className={`w-2 h-2 rounded-full ${colors[index % colors.length].replace('text-', 'bg-')}`} />
                    <span className="text-xs text-white/60">{method}</span>
                    <span className="text-xs font-bold text-white">{count}</span>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};