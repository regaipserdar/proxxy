import React, { useMemo, useState } from 'react';
import { Virtuoso } from 'react-virtuoso';
import { ChevronRight, Globe, Loader2, Folder, FileCode } from 'lucide-react';
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
    | { type: 'host'; host: string; count: number; isOpen: boolean; level: number }
    | { type: 'folder'; host: string; path: string; count: number; isOpen: boolean; level: number; latestTs: number }
    | {
        type: 'request';
        request: TrafficRequest;
        host: string;
        level: number;
        displayId: string | number;
        methodColor: string;
        statusColor: string;
        fileName: string;
    };

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
    const [expandedHosts, setExpandedHosts] = useState<Record<string, boolean>>({});
    const [expandedFolders, setExpandedFolders] = useState<Record<string, boolean>>({});

    const toggleHost = (host: string) => {
        setExpandedHosts(prev => ({ ...prev, [host]: !prev[host] }));
        onHostSelect?.(host);
    };

    const toggleFolder = (key: string) => {
        setExpandedFolders(prev => ({ ...prev, [key]: !prev[key] }));
    };

    // Path'i klasör yapısına ayır
    const parsePathSegments = (url: string): { segments: string[], query: string, fileName: string } => {
        try {
            const u = new URL(url);
            const pathname = u.pathname;
            const query = u.search;

            // Path'i segment'lere ayır ve boşları temizle
            const segments = pathname.split('/').filter(s => s.length > 0);

            // Son segment dosya adı olabilir (extension içeriyorsa veya query varsa)
            let fileName = '';
            const lastSegment = segments[segments.length - 1];

            if (lastSegment && (lastSegment.includes('.') || query)) {
                fileName = lastSegment;
                segments.pop(); // Son segment'i klasör listesinden çıkar
            }

            return { segments, query, fileName };
        } catch {
            return { segments: [], query: '', fileName: url };
        }
    };

    const flatData = useMemo(() => {
        const items: FlatItem[] = [];
        const hostEntries = Object.entries(groupedRequests);

        // Host sıralaması
        hostEntries.sort(([hostA], [hostB]) => {
            const statusA = getHostScopeStatus?.(hostA) || 'neutral';
            const statusB = getHostScopeStatus?.(hostB) || 'neutral';
            const getWeight = (s: string) => s === 'out-of-scope' ? 1 : 0;

            if (getWeight(statusA) !== getWeight(statusB)) {
                return getWeight(statusA) - getWeight(statusB);
            }
            return hostA.localeCompare(hostB);
        });

        hostEntries.forEach(([host, reqs]) => {
            const isHostOpen = expandedHosts[host] || false;
            items.push({ type: 'host', host, count: reqs.length, isOpen: isHostOpen, level: 0 });

            if (isHostOpen) {
                // Tree yapısını oluştur
                interface TreeNode {
                    requests: TrafficRequest[];
                    children: Record<string, TreeNode>;
                    latestTs: number;
                }

                const tree: TreeNode = { requests: [], children: {}, latestTs: 0 };

                // İstekleri tree'ye yerleştir
                reqs.forEach(req => {
                    const { segments, query, fileName } = parsePathSegments(req.url);
                    const ts = typeof req.timestamp === 'number' ? req.timestamp : new Date(req.timestamp).getTime();

                    let currentNode = tree;

                    // Her segment için klasör oluştur
                    segments.forEach(segment => {
                        if (!currentNode.children[segment]) {
                            currentNode.children[segment] = { requests: [], children: {}, latestTs: 0 };
                        }
                        currentNode = currentNode.children[segment];
                        currentNode.latestTs = Math.max(currentNode.latestTs, ts);
                    });

                    // İsteği son node'a ekle
                    currentNode.requests.push(req);
                    currentNode.latestTs = Math.max(currentNode.latestTs, ts);
                });

                // Tree'yi recursive olarak flat listeye çevir
                const buildFlatList = (node: TreeNode, pathPrefix: string, level: number) => {
                    // Önce klasörleri ekle (timestamp'e göre sıralı)
                    const folderEntries = Object.entries(node.children).sort((a, b) => b[1].latestTs - a[1].latestTs);

                    folderEntries.forEach(([folderName, childNode]) => {
                        const currentPath = pathPrefix ? `${pathPrefix}/${folderName}` : folderName;
                        const folderKey = `${host}|${currentPath}`;
                        const isFolderOpen = expandedFolders[folderKey] || false;
                        const totalRequests = countRequests(childNode);

                        items.push({
                            type: 'folder',
                            host,
                            path: currentPath,
                            count: totalRequests,
                            isOpen: isFolderOpen,
                            level,
                            latestTs: childNode.latestTs
                        });

                        if (isFolderOpen) {
                            buildFlatList(childNode, currentPath, level + 1);
                        }
                    });

                    // Sonra bu seviyedeki istekleri ekle
                    if (node.requests.length > 0) {
                        const sortedReqs = [...node.requests].sort((a, b) => {
                            const tsA = typeof a.timestamp === 'number' ? a.timestamp : new Date(a.timestamp).getTime();
                            const tsB = typeof b.timestamp === 'number' ? b.timestamp : new Date(b.timestamp).getTime();
                            if (tsB !== tsA) return tsB - tsA;
                            const idA = (a as any).id || a.requestId;
                            const idB = (b as any).id || b.requestId;
                            return idB > idA ? 1 : -1;
                        });

                        sortedReqs.forEach(req => {
                            const displayId = (req as any).id || req.requestId;
                            const methodColor = getMethodColor(req.method).split(' ')[1];
                            const statusColor = getStatusColor(req.status);
                            const { query, fileName } = parsePathSegments(req.url);

                            const displayName = fileName || (query ? `index${query}` : 'index');

                            items.push({
                                type: 'request',
                                request: req,
                                host,
                                level,
                                displayId,
                                methodColor,
                                statusColor,
                                fileName: displayName
                            });
                        });
                    }
                };

                // Yardımcı fonksiyon: Toplam istek sayısı
                const countRequests = (node: TreeNode): number => {
                    let count = node.requests.length;
                    Object.values(node.children).forEach(child => {
                        count += countRequests(child);
                    });
                    return count;
                };

                buildFlatList(tree, '', 1);
            }
        });

        return items;
    }, [groupedRequests, expandedHosts, expandedFolders, getHostScopeStatus]);

    return (
        <div className="h-full bg-[#0E1015]/60 flex flex-col overflow-hidden">
            <Virtuoso
                style={{ height: '100%' }}
                data={flatData}
                itemContent={(_index, item) => {
                    const marginLeft = `${item.level * 16}px`;

                    if (item.type === 'host') {
                        return (
                            <div
                                className="group flex items-center gap-2 p-1.5 mx-1 mt-1 hover:bg-white/5 rounded-md cursor-pointer select-none transition-all active:scale-[0.98] border border-transparent hover:border-white/5"
                                onClick={() => toggleHost(item.host)}
                                onContextMenu={(e) => handleContextMenu(e, item.host)}
                            >
                                <ChevronRight
                                    size={14}
                                    className={`text-slate-500 transition-transform duration-200 ${item.isOpen ? 'rotate-90' : ''}`}
                                />
                                <Globe size={14} className={(() => {
                                    const status = getHostScopeStatus?.(item.host) || 'neutral';
                                    if (status === 'out-of-scope') return 'text-red-500/70';
                                    if (status === 'in-scope') return 'text-emerald-500/70';
                                    return 'text-cyan-500/70';
                                })()} />
                                <span className={`text-[12px] font-bold text-slate-300 truncate flex-1 ${(() => {
                                    const status = getHostScopeStatus?.(item.host) || 'neutral';
                                    if (status === 'out-of-scope') return 'text-slate-500 line-through decoration-red-500/30';
                                    return '';
                                })()}`}>
                                    {item.host}
                                </span>
                                {loadingDomain === item.host ? (
                                    <Loader2 size={12} className="animate-spin text-cyan-500" />
                                ) : (
                                    <span className="text-[10px] font-mono text-slate-500 bg-black/20 px-1.5 rounded-full border border-white/5">{item.count}</span>
                                )}
                            </div>
                        );
                    }

                    if (item.type === 'folder') {
                        const folderKey = `${item.host}|${item.path}`;
                        const folderName = item.path.split('/').pop() || item.path;

                        return (
                            <div
                                style={{ marginLeft }}
                                className="group flex items-center gap-2 p-1 mx-1 hover:bg-white/5 rounded cursor-pointer select-none transition-colors border-l border-white/5 hover:border-cyan-500/30 pl-2"
                                onClick={() => toggleFolder(folderKey)}
                            >
                                <ChevronRight
                                    size={12}
                                    className={`text-slate-600 transition-transform duration-150 ${item.isOpen ? 'rotate-90' : ''}`}
                                />
                                <Folder size={12} className={`${item.isOpen ? 'text-amber-500/80' : 'text-slate-500'}`} />
                                <span className="text-[11px] font-mono text-slate-400 truncate flex-1">
                                    {folderName}
                                </span>
                                <span className="text-[9px] text-slate-600">{item.count}</span>
                            </div>
                        );
                    }

                    // Request
                    const req = item.request;
                    const isSelected = selectedId === req.requestId;

                    return (
                        <div
                            style={{ marginLeft }}
                            onClick={() => setSelectedId(req.requestId)}
                            onContextMenu={(e) => handleRequestContextMenu(e, req)}
                            className={`flex items-center gap-2 p-1.5 mx-1 mb-0.5 rounded cursor-pointer transition-all border-l-2 ${isSelected
                                ? 'bg-cyan-900/10 border-cyan-500 text-cyan-50'
                                : 'border-transparent hover:bg-white/5 text-slate-500 hover:text-slate-300 hover:border-slate-700'
                                }`}
                        >
                            <FileCode size={11} className="text-slate-600 shrink-0" />
                            <span className="text-[9px] font-mono opacity-50 w-8 text-right shrink-0 truncate">
                                #{item.displayId}
                            </span>
                            <span className={`text-[9px] font-black w-8 shrink-0 tracking-tighter uppercase ${item.methodColor}`}>
                                {req.method}
                            </span>
                            <span className="text-[10px] font-mono truncate flex-1 opacity-80">
                                {item.fileName}
                            </span>
                            <span className={`text-[9px] font-bold tracking-tighter w-8 shrink-0 text-right ${item.statusColor}`}>
                                {req.status || '...'}
                            </span>
                        </div>
                    );
                }}
            />
        </div>
    );
};