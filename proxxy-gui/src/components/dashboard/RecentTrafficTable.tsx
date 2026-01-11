import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { ArrowRight, Globe } from 'lucide-react';
import { HttpTransaction } from '../../types/graphql';
import { format } from 'date-fns';
import { Badge } from "@/components/ui/badge";

interface RecentTrafficTableProps {
  traffic: HttpTransaction[];
  isConnected?: boolean;
}

export const RecentTrafficTable: React.FC<RecentTrafficTableProps> = ({
  traffic,
  isConnected = true
}) => {
  const navigate = useNavigate();
  // Dashboard focus: only show last 5-6 items for a compact view
  const recentTraffic = traffic.slice(0, 6);

  const [isTrafficFlowing, setIsTrafficFlowing] = useState(false);
  const [lastTrafficLength, setLastTrafficLength] = useState(0);

  useEffect(() => {
    if (traffic.length > 0 && traffic.length !== lastTrafficLength) {
      setLastTrafficLength(traffic.length);
      const timestamp = (traffic[0] as any).timestamp;
      let lastTxTime: number;

      if (typeof timestamp === 'string') {
        lastTxTime = new Date(timestamp).getTime();
      } else if (typeof timestamp === 'number') {
        lastTxTime = timestamp > 1e12 ? timestamp : timestamp * 1000;
      } else {
        lastTxTime = 0;
      }

      const now = Date.now();
      const isRecent = lastTxTime > 0 && (now - lastTxTime) < 5000;
      setIsTrafficFlowing(isRecent);
      const timer = setTimeout(() => setIsTrafficFlowing(false), 3000);
      return () => clearTimeout(timer);
    } else if (traffic.length === 0) {
      setIsTrafficFlowing(false);
      setLastTrafficLength(0);
    }
  }, [traffic, lastTrafficLength]);

  const getMethodBadge = (method?: string) => {
    const m = method?.toUpperCase() || 'UNKNOWN';
    switch (m) {
      case 'GET': return 'bg-blue-500/10 text-blue-400 border-blue-500/20';
      case 'POST': return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
      case 'PUT': return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
      case 'DELETE': return 'bg-red-500/10 text-red-500 border-red-500/20';
      default: return 'bg-muted text-muted-foreground border-border';
    }
  };

  const getStatusColor = (status?: number) => {
    if (!status) return 'text-muted-foreground';
    if (status >= 200 && status < 300) return 'text-emerald-500';
    if (status >= 300 && status < 400) return 'text-blue-400';
    if (status >= 400 && status < 500) return 'text-yellow-500';
    return 'text-red-500';
  };

  const formatExactTime = (timestamp: string | number | undefined) => {
    try {
      if (!timestamp) return '--:--:--';
      const date = new Date(typeof timestamp === 'number' ? timestamp * 1000 : timestamp);
      return format(date, 'HH:mm:ss');
    } catch {
      return 'Err';
    }
  };

  return (
    <div className="flex flex-col h-full bg-[#111318]">
      <div className="flex-1 p-3 space-y-1.5 overflow-hidden">
        {recentTraffic.length === 0 ? (
          <div className="h-40 flex flex-col items-center justify-center text-muted-foreground/20 gap-3 border border-dashed border-white/5 rounded-xl mx-1 text-center px-4">
            {isConnected ? (
              <>
                <div className="h-5 w-5 border-b-2 border-primary rounded-full animate-spin" />
                <p className="text-[9px] font-black uppercase tracking-[0.2em]">Listening for packets...</p>
              </>
            ) : (
              <>
                <Globe className="h-6 w-6 opacity-30" />
                <p className="text-[9px] font-black uppercase tracking-[0.2em]">Signal Lost...</p>
              </>
            )}
          </div>
        ) : (
          recentTraffic.map((tx, i) => (
            <div
              key={tx.requestId || i}
              onClick={() => navigate(`/proxy/${tx.requestId}`)}
              className="group flex items-center gap-4 p-2.5 rounded-xl bg-white/[0.01] border border-white/5 hover:bg-white/[0.04] hover:border-primary/20 cursor-pointer transition-all animate-in slide-in-from-bottom-2 duration-300"
            >
              <div className="flex items-center gap-2.5 min-w-[100px] shrink-0">
                <span className={`text-[10px] font-black font-mono w-8 text-center ${getStatusColor(tx.status)}`}>
                  {tx.status || '---'}
                </span>
                <Badge variant="outline" className={`h-5 min-w-[45px] justify-center font-black text-[8px] uppercase tracking-tighter rounded-md ${getMethodBadge(tx.method)}`}>
                  {tx.method || '???'}
                </Badge>
              </div>

              <div className="flex-1 min-w-0">
                <p className="text-[11px] font-mono text-slate-400 group-hover:text-slate-200 truncate">
                  {tx.url}
                </p>
              </div>

              <div className="flex items-center gap-2 shrink-0">
                <span className="hidden md:block text-[9px] font-bold text-muted-foreground/40 font-mono tracking-tighter">
                  {formatExactTime((tx as any).timestamp)}
                </span>
                <ArrowRight className="w-3 h-3 text-primary opacity-0 group-hover:opacity-100 transition-all translate-x-2 group-hover:translate-x-0" />
              </div>
            </div>
          ))
        )}
      </div>

      <div className="p-2 px-4 bg-white/[0.01] border-t border-white/5 flex justify-between items-center h-8">
        <div className="flex items-center gap-3">
          <div className={`w-1 h-1 rounded-full ${isTrafficFlowing ? 'bg-emerald-500 shadow-[0_0_5px_rgba(16,185,129,0.5)]' : 'bg-slate-800'}`} />
          <span className="text-[8px] font-black text-muted-foreground/30 uppercase tracking-widest">{isTrafficFlowing ? 'STREAM_ACTIVE' : 'IDLE'}</span>
        </div>
        <span className="text-[8px] font-black text-muted-foreground/30 uppercase tracking-widest">{recentTraffic.length}_PACKETS</span>
      </div>
    </div>
  );
};
