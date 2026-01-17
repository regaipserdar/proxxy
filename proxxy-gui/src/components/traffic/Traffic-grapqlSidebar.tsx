import React, { useMemo } from 'react';
import { Virtuoso } from 'react-virtuoso';
import { ChevronRight, Globe, Loader2 } from 'lucide-react';
import { TrafficRequest, getMethodColor, getStatusColor } from './types-grapql';

interface TrafficSidebarProps {
    groupedRequests: Record<string, TrafficRequest[]>;
    selectedId: string | null;
    setSelectedId: (id: string) => void;
    handleContextMenu: (e: React.MouseEvent, host: string) => void;
    handleRequestContextMenu: (e: React.MouseEvent, request: TrafficRequest) => void;
    loadingDomain: string | null;
    getHostScopeStatus?: (host: string) => 'in-scope' | 'out-of-scope' | 'neutral';
    onHostSelect?: (host: string | null) => void;
}

type FlatItem =
    | { type: 'host'; host: string; count: number; isOpen: boolean }
    | { type: 'request'; request: TrafficRequest; host: string; id: number };

export const TrafficSidebar = ({
    groupedRequests,
    selectedId,
    setSelectedId,
    handleContextMenu,
    handleRequestContextMenu,
    loadingDomain,
    getHostScopeStatus,
    onHostSelect
}: TrafficSidebarProps) => {
    // State for which hosts are expanded
    const [expandedHosts, setExpandedHosts] = React.useState<Record<string, boolean>>({});

    const toggleHost = (host: string) => {
        setExpandedHosts(prev => ({ ...prev, [host]: !prev[host] }));
        onHostSelect?.(host); // Track selected host for keyboard shortcuts
    };

    const flatData = useMemo(() => {
        console.log('[Sidebar] Recalculating flat data...');
        const items: FlatItem[] = [];

        // Convert to array and sort
        const entries = Object.entries(groupedRequests);

        // Sort Hosts: In-Scope/Neutral at top, Out-of-Scope at bottom. Alphabetical tie-break.
        entries.sort(([hostA], [hostB]) => {
            const statusA = getHostScopeStatus?.(hostA) || 'neutral';
            const statusB = getHostScopeStatus?.(hostB) || 'neutral';

            // 0 = Top (In-Scope/Neutral), 1 = Bottom (Out-of-Scope)
            const getWeight = (s: string) => s === 'out-of-scope' ? 1 : 0;

            const weightA = getWeight(statusA);
            const weightB = getWeight(statusB);

            if (weightA !== weightB) {
                return weightA - weightB;
            }
            return hostA.localeCompare(hostB);
        });

        entries.forEach(([host, reqs]) => {
            const isOpen = expandedHosts[host] || false;
            items.push({ type: 'host', host, count: reqs.length, isOpen });

            if (isOpen) {
                // Sort Requests: Newest first (descending timestamp)
                const sortedReqs = [...reqs].sort((a, b) => {
                    const tsA = typeof a.timestamp === 'number' ? a.timestamp : new Date(a.timestamp).getTime();
                    const tsB = typeof b.timestamp === 'number' ? b.timestamp : new Date(b.timestamp).getTime();
                    return tsB - tsA; // Descending
                });

                // Assign IDs based on total count (newest = highest number)
                // Since sortedReqs[0] is the newest, it should get ID = total count
                const total = sortedReqs.length;
                sortedReqs.forEach((req, index) => {
                    items.push({ type: 'request', request: req, host, id: total - index });
                });
            }
        });
        return items;
    }, [groupedRequests, expandedHosts, getHostScopeStatus]);

    return (
        <div className="h-full bg-[#0E1015]/60 flex flex-col overflow-hidden">
            <Virtuoso
                style={{ height: '100%' }}
                data={flatData}
                itemContent={(_index, item) => {
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
                                <Globe size={14} className={(() => {
                                    const status = getHostScopeStatus?.(item.host) || 'neutral';
                                    if (status === 'out-of-scope') return 'text-red-500/70';
                                    if (status === 'in-scope') return 'text-emerald-500/70';
                                    return 'text-cyan-500/70';
                                })()} />
                                <span className={`text-[11px] font-bold uppercase tracking-wider truncate flex-1 ${(() => {
                                    const status = getHostScopeStatus?.(item.host) || 'neutral';
                                    if (status === 'out-of-scope') return 'text-red-400/50 line-through';
                                    if (status === 'in-scope') return 'text-emerald-400';
                                    return 'text-slate-400';
                                })()}`}>
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
                                <span className="text-[9px] font-mono text-slate-600 w-6 text-right shrink-0">
                                    #{item.id}
                                </span>
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
