import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Activity, ExternalLink, Clock } from 'lucide-react';
import { HttpTransaction } from '../../types/graphql';
import { formatDistanceToNow } from 'date-fns';

interface RecentTrafficTableProps {
  traffic: HttpTransaction[];
}

export const RecentTrafficTable: React.FC<RecentTrafficTableProps> = ({ traffic }) => {
  const navigate = useNavigate();
  
  // Show only the most recent 10 transactions
  const recentTraffic = traffic.slice(0, 10);

  const getMethodColor = (method?: string) => {
    switch (method) {
      case 'GET': return 'text-blue-400 bg-blue-500/10 border-blue-500/20';
      case 'POST': return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
      case 'PUT': return 'text-yellow-400 bg-yellow-500/10 border-yellow-500/20';
      case 'DELETE': return 'text-red-400 bg-red-500/10 border-red-500/20';
      case 'PATCH': return 'text-purple-400 bg-purple-500/10 border-purple-500/20';
      default: return 'text-gray-400 bg-gray-500/10 border-gray-500/20';
    }
  };

  const getStatusColor = (status?: number) => {
    if (!status) return 'text-gray-400';
    if (status >= 200 && status < 300) return 'text-emerald-400';
    if (status >= 300 && status < 400) return 'text-blue-400';
    if (status >= 400 && status < 500) return 'text-yellow-400';
    if (status >= 500) return 'text-red-400';
    return 'text-gray-400';
  };

  const formatTimestamp = (timestamp: string | number | undefined) => {
    try {
      if (!timestamp) return 'Unknown';
      const date = typeof timestamp === 'string' ? new Date(timestamp) : new Date(timestamp * 1000);
      return formatDistanceToNow(date, { addSuffix: true });
    } catch {
      return 'Unknown';
    }
  };

  const truncateUrl = (url?: string, maxLength = 40) => {
    if (!url) return 'N/A';
    if (url.length <= maxLength) return url;
    return url.substring(0, maxLength) + '...';
  };

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
            <Activity className="h-5 w-5 text-emerald-400" />
          </div>
          <div>
            <h2 className="text-lg font-bold text-white">Recent Traffic</h2>
            <p className="text-xs text-white/60">Latest HTTP transactions</p>
          </div>
        </div>
        <button
          onClick={() => navigate('/traffic')}
          className="flex items-center gap-2 px-3 py-2 rounded-lg bg-white/5 hover:bg-white/10 transition-all text-sm text-white/80 hover:text-white"
        >
          View All
          <ExternalLink className="h-4 w-4" />
        </button>
      </div>

      {recentTraffic.length === 0 ? (
        <div className="text-center py-12">
          <Activity className="h-12 w-12 text-white/20 mx-auto mb-4" />
          <p className="text-white/40 text-sm">No recent traffic data available</p>
          <p className="text-white/20 text-xs mt-1">Traffic will appear here as it's captured</p>
        </div>
      ) : (
        <div className="space-y-2">
          {recentTraffic.map((transaction, index) => (
            <div
              key={transaction.requestId || index}
              className="flex items-center gap-4 p-3 rounded-lg bg-white/5 hover:bg-white/10 transition-all cursor-pointer group"
              onClick={() => navigate(`/traffic/${transaction.requestId}`)}
            >
              {/* Method Badge */}
              <div className={`px-2 py-1 rounded text-xs font-bold border ${getMethodColor(transaction.method)}`}>
                {transaction.method || 'N/A'}
              </div>

              {/* URL */}
              <div className="flex-1 min-w-0">
                <p className="text-sm text-white group-hover:text-blue-400 transition-colors truncate" title={transaction.url}>
                  {truncateUrl(transaction.url)}
                </p>
              </div>

              {/* Status Code */}
              <div className={`text-sm font-bold ${getStatusColor(transaction.status)}`}>
                {transaction.status || 'N/A'}
              </div>

              {/* Timestamp */}
              <div className="flex items-center gap-1 text-xs text-white/40 min-w-0">
                <Clock className="h-3 w-3" />
                <span className="truncate">
                  {formatTimestamp((transaction as any).timestamp)}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Footer */}
      {recentTraffic.length > 0 && (
        <div className="mt-4 pt-4 border-t border-white/5 flex items-center justify-between">
          <div className="text-xs text-white/40">
            Showing {recentTraffic.length} of {traffic.length} transactions
          </div>
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-emerald-400 rounded-full animate-pulse"></div>
            <span className="text-xs text-white/60">Live updates</span>
          </div>
        </div>
      )}
    </div>
  );
};