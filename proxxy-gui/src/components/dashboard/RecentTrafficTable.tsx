import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Activity, Clock, ArrowRight } from 'lucide-react';
import { HttpTransaction } from '../../types/graphql';
import { format } from 'date-fns';

interface RecentTrafficTableProps {
  traffic: HttpTransaction[];
  isConnected?: boolean;
}

export const RecentTrafficTable: React.FC<RecentTrafficTableProps> = ({
  traffic,
  isConnected = true
}) => {
  const navigate = useNavigate();
  const recentTraffic = traffic.slice(0, 10);

  // "Live" göstergesi için son işlem zamanını takip et
  const [isTrafficFlowing, setIsTrafficFlowing] = useState(false);
  const [lastTrafficLength, setLastTrafficLength] = useState(0);

  useEffect(() => {
    if (traffic.length > 0 && traffic.length !== lastTrafficLength) {
      // Yeni traffic geldiğinde
      setLastTrafficLength(traffic.length);

      // Timestamp'i doğru parse et
      const timestamp = (traffic[0] as any).timestamp;
      let lastTxTime: number;

      if (typeof timestamp === 'string') {
        // ISO string formatı
        lastTxTime = new Date(timestamp).getTime();
      } else if (typeof timestamp === 'number') {
        // Unix timestamp (saniye veya milisaniye)
        lastTxTime = timestamp > 1e12 ? timestamp : timestamp * 1000;
      } else {
        lastTxTime = 0;
      }

      const now = Date.now();
      const isRecent = lastTxTime > 0 && (now - lastTxTime) < 5000; // 5 saniye içinde

      setIsTrafficFlowing(isRecent);

      // 3 saniye sonra idle'a çek
      const timer = setTimeout(() => setIsTrafficFlowing(false), 3000);
      return () => clearTimeout(timer);
    } else if (traffic.length === 0) {
      setIsTrafficFlowing(false);
      setLastTrafficLength(0);
    }
  }, [traffic, lastTrafficLength]);

  const getMethodStyle = (method?: string) => {
    switch (method?.toUpperCase()) {
      case 'GET': return 'text-blue-400 bg-blue-500/10 border-blue-500/20';
      case 'POST': return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
      case 'PUT': return 'text-orange-400 bg-orange-500/10 border-orange-500/20';
      case 'DELETE': return 'text-red-400 bg-red-500/10 border-red-500/20';
      default: return 'text-gray-400 bg-gray-500/10 border-gray-500/20';
    }
  };

  const getStatusColor = (status?: number) => {
    if (!status) return 'text-gray-500';
    if (status >= 200 && status < 300) return 'text-emerald-400';
    if (status >= 300 && status < 400) return 'text-blue-400';
    if (status >= 400 && status < 500) return 'text-yellow-400';
    return 'text-red-500';
  };

  const formatExactTime = (timestamp: string | number | undefined) => {
    try {
      if (!timestamp) return '--:--:--';
      // Backend'den saniye geliyorsa 1000 ile çarp, milisaniye geliyorsa çarpma
      const date = new Date(typeof timestamp === 'number' ? timestamp * 1000 : timestamp);
      return format(date, 'HH:mm:ss.SSS');
    } catch {
      return 'Invalid Date';
    }
  };

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b border-white/5 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-indigo-500/10 border border-indigo-500/20">
            <Activity className="h-4 w-4 text-indigo-400" />
          </div>
          <div>
            <h2 className="text-sm font-bold text-white tracking-wide">Live Traffic</h2>
            <div className="flex items-center gap-2 mt-0.5">
              {/* Gerçek Durum Göstergesi */}
              <div className={`w-1.5 h-1.5 rounded-full ${isConnected ? (isTrafficFlowing ? 'bg-emerald-400 animate-pulse' : 'bg-gray-500') : 'bg-red-500'}`} />
              <p className="text-[10px] font-mono text-white/40 uppercase">
                {!isConnected ? 'DISCONNECTED' : (isTrafficFlowing ? 'RECEIVING DATA' : 'IDLE')}
              </p>
            </div>
          </div>
        </div>
        <button
          onClick={() => navigate('/traffic')}
          className="group flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 transition-all text-xs font-medium text-white/60 hover:text-white"
        >
          View All
          <ArrowRight className="h-3 w-3 group-hover:translate-x-0.5 transition-transform" />
        </button>
      </div>

      {/* List Content */}
      <div className="flex-1 overflow-hidden p-2 space-y-1">
        {recentTraffic.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center text-white/20 py-8">
            <Activity className="h-8 w-8 mb-2 opacity-20" />
            <p className="text-xs">Waiting for requests...</p>
          </div>
        ) : (
          recentTraffic.map((tx, i) => (
            <div
              key={tx.requestId || i}
              onClick={() => navigate(`/traffic/${tx.requestId}`)}
              className="group flex items-center gap-3 p-2 rounded hover:bg-white/5 cursor-pointer transition-colors border border-transparent hover:border-white/5"
            >
              {/* Time - Monospace ve Gri */}
              <div className="hidden sm:flex items-center gap-1.5 text-white/30 min-w-[90px]">
                <Clock className="h-3 w-3" />
                <span className="text-[10px] font-mono tracking-tighter">
                  {formatExactTime((tx as any).timestamp)}
                </span>
              </div>

              {/* Method */}
              <div className={`px-1.5 py-0.5 rounded text-[10px] font-bold border min-w-[45px] text-center ${getMethodStyle(tx.method)}`}>
                {tx.method || '???'}
              </div>

              {/* URL - Truncate */}
              <div className="flex-1 min-w-0">
                <p className="text-xs text-white/80 group-hover:text-white truncate font-mono">
                  {tx.url}
                </p>
              </div>

              {/* Status */}
              <div className={`text-xs font-bold font-mono text-right min-w-[35px] ${getStatusColor(tx.status)}`}>
                {tx.status || '---'}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Footer Stats */}
      <div className="px-4 py-3 border-t border-white/5 bg-white/[0.02] flex justify-between items-center text-[10px] text-white/30 font-mono">
        <span>BUFFER: {recentTraffic.length} items</span>
        <span>LATENCY: &lt;1ms</span>
      </div>
    </div>
  );
};