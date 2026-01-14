import React, { useMemo } from 'react';
import { Virtuoso } from 'react-virtuoso';
import { ChevronRight, Globe, Loader2 } from 'lucide-react';
import { TrafficRequest, getMethodColor, getStatusColor } from './types-grapql';
import { ScrollArea } from '@/components/ui/scroll-area';

interface TrafficSidebarProps {
    groupedRequests: Record<string, TrafficRequest[]>;
    selectedId: string | null;
    setSelectedId: (id: string) => void;
    handleContextMenu: (e: React.MouseEvent, host: string) => void;
    handleRequestContextMenu: (e: React.MouseEvent, request: TrafficRequest) => void;
    loadingDomain: string | null;
}

type FlatItem =
    | { type: 'host'; host: string; count: number; isOpen: boolean }
    | { type: 'request'; request: TrafficRequest; host: string };

export const TrafficSidebar = ({
    groupedRequests,
    selectedId,
    setSelectedId,
    handleContextMenu,
    handleRequestContextMenu,
    loadingDomain
}: TrafficSidebarProps) => {
    // State for which hosts are expanded
    const [expandedHosts, setExpandedHosts] = React.useState<Record<string, boolean>>({});

    const toggleHost = (host: string) => {
        setExpandedHosts(prev => ({ ...prev, [host]: !prev[host] }));
    };

    const flatData = useMemo(() => {
        console.log('[Sidebar] Recalculating flat data...');
        const items: FlatItem[] = [];
        Object.entries(groupedRequests).forEach(([host, reqs]) => {
            const isOpen = expandedHosts[host] || false;
            items.push({ type: 'host', host, count: reqs.length, isOpen });
            if (isOpen) {
                reqs.forEach(req => {
                    items.push({ type: 'request', request: req, host });
                });
            }
        });
        return items;
    }, [groupedRequests, expandedHosts]);

    return (
        <div className="h-full bg-[#0E1015]/60 flex flex-col overflow-hidden">
            <Virtuoso
                style={{ height: '100%' }}
                data={flatData}
                itemContent={(index, item) => {
                    if (item.type === 'host') {
                        return (
                            <div
                                className="group flex items-center gap-2 p-2 mx-1 hover:bg-white/5 rounded-md cursor-pointer select-none transition-all active:scale-[0.98]"
                                onClick={() => toggleHost(item.host)}
                                onContextMenu={(e) => handleContextMenu(e, item.host)}
                            >
                                <ChevronRight
                                    size={12}
                                    className={`text-slate-600 transition-transform ${item.isOpen ? 'rotate-90' : ''}`}
                                />
                                <Globe size={14} className="text-cyan-500/70" />
                                <span className="text-[11px] font-bold uppercase tracking-wider truncate flex-1 text-slate-400">
                                    {item.host}
                                </span>
                                {loadingDomain === item.host ? (
                                    <Loader2 size={12} className="animate-spin text-cyan-500" />
                                ) : (
                                    <span className="text-[10px] font-mono text-slate-600 bg-white/5 px-1.5 rounded">{item.count}</span>
                                )}
                            </div>
                        );
                    } else {
                        const req = item.request;
                        const isSelected = selectedId === req.requestId;
                        return (
                            <div
                                onClick={() => setSelectedId(req.requestId)}
                                onContextMenu={(e) => handleRequestContextMenu(e, req)}
                                className={`flex items-center gap-3 p-2 mx-1 ml-6 mb-0.5 rounded-md cursor-pointer transition-all border ${isSelected
                                    ? 'bg-cyan-500/10 border-cyan-500/30 text-cyan-50 shadow-[0_0_15px_rgba(6,182,212,0.05)]'
                                    : 'border-transparent hover:bg-white/5 text-slate-500 hover:text-slate-300'
                                    }`}
                            >
                                <span className={`text-[9px] font-black w-8 shrink-0 tracking-tighter ${getMethodColor(req.method).split(' ')[1]}`}>
                                    {req.method}
                                </span>
                                <span className="text-[11px] font-mono truncate flex-1 leading-none tracking-tight">
                                    {(() => {
                                        try {
                                            const u = new URL(req.url);
                                            return u.pathname + u.search;
                                        } catch { return req.url; }
                                    })()}
                                </span>
                                <span className={`text-[10px] font-black tracking-tighter w-10 shrink-0 text-right ${getStatusColor(req.status)}`}>
                                    {req.status || 'PEND'}
                                </span>
                            </div>
                        );
                    }
                }}
            />
        </div>
    );
};
