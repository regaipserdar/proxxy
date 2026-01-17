import { useState, useMemo, useEffect, useCallback } from 'react';
import { useQuery, useSubscription, useLazyQuery, useApolloClient, useMutation } from '@apollo/client';
import { useNavigate } from 'react-router-dom';
import { RefreshCw, Send, Copy, Layers, Trash2, Target, ShieldOff } from 'lucide-react';
import { toast } from 'sonner';

import {
    GET_HTTP_TRANSACTIONS,
    GET_TRANSACTION_DETAILS,
    TRAFFIC_UPDATES,
    DELETE_REQUESTS_BY_HOST,
    GET_SCOPE_RULES,
    ADD_SCOPE_RULE,
    DELETE_SCOPE_RULE
} from '@/graphql/operations';

import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable';
import { ScopeRule } from '@/types';

// Sub-components
import { TrafficRequest } from '@/components/traffic/types-grapql';
import { TrafficToolbar } from '@/components/traffic/Traffic-grapqlToolbar';
import { TrafficSidebar } from '@/components/traffic/Traffic-grapqlSidebar';
import { RequestInspector } from '@/components/traffic/Reques-grapqltInspector';
import { useRepeaterStore } from '@/store/repeaterStore';
import { formatRequestRaw } from '@/lib/http-utils';

export const TrafficTreePage = () => {
    console.log('[TrafficTree] Rendering page...');
    const client = useApolloClient();

    // States
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [selectedHost, setSelectedHost] = useState<string | null>(null);
    const [filterQuery, setFilterQuery] = useState('');
    const [activeMethodFilter, setActiveMethodFilter] = useState<string | null>(null);
    const [contextMenu, setContextMenu] = useState<{ x: number, y: number, host?: string, request?: TrafficRequest } | null>(null);
    const [loadingDomain, setLoadingDomain] = useState<string | null>(null);
    const [hideConnect, setHideConnect] = useState(false);
    const navigate = useNavigate();
    const addTask = useRepeaterStore(state => state.addTask);
    const [deleteByHost] = useMutation(DELETE_REQUESTS_BY_HOST, {
        onCompleted: () => refetch()
    });

    // Scope Management
    const { data: scopeData, refetch: refetchScope } = useQuery(GET_SCOPE_RULES);
    const [addScopeRule] = useMutation(ADD_SCOPE_RULE);
    const scopeRules: ScopeRule[] = scopeData?.scopeRules || [];

    // Check if host is in scope (Include rules) or out of scope (Exclude rules)
    const getHostScopeStatus = useCallback((host: string): 'in-scope' | 'out-of-scope' | 'neutral' => {
        const excludeRules = scopeRules.filter(r => r.ruleType === 'Exclude' && r.enabled);
        const includeRules = scopeRules.filter(r => r.ruleType === 'Include' && r.enabled);

        // Check exclude first (takes precedence)
        for (const rule of excludeRules) {
            if (rule.isRegex) {
                try {
                    if (new RegExp(rule.pattern).test(host)) return 'out-of-scope';
                } catch { /* invalid regex */ }
            } else {
                // Glob matching (simple)
                const pattern = rule.pattern.replace(/\*/g, '.*');
                if (new RegExp(`^${pattern}$`).test(host)) return 'out-of-scope';
            }
        }

        // If no include rules, everything is neutral (captured)
        if (includeRules.length === 0) return 'neutral';

        // Check include
        for (const rule of includeRules) {
            if (rule.isRegex) {
                try {
                    if (new RegExp(rule.pattern).test(host)) return 'in-scope';
                } catch { /* invalid regex */ }
            } else {
                const pattern = rule.pattern.replace(/\*/g, '.*');
                if (new RegExp(`^${pattern}$`).test(host)) return 'in-scope';
            }
        }

        return 'out-of-scope'; // Include rules exist but host doesn't match any
    }, [scopeRules]);

    // Add host to scope (Include or Exclude)
    const addHostToScope = useCallback(async (host: string, ruleType: 'Include' | 'Exclude') => {
        try {
            await addScopeRule({
                variables: {
                    ruleType,
                    pattern: `*${host}*`,
                    isRegex: false
                }
            });
            toast.success(`${host} added to ${ruleType === 'Include' ? 'Scope' : 'Out-of-Scope'}`);
            refetchScope();
        } catch (err: any) {
            toast.error(`Failed to add rule: ${err.message}`);
        }
        setContextMenu(null);
    }, [addScopeRule, refetchScope]);

    // Remove host from scope (delete all rules matching this host)
    const [deleteScopeRule] = useMutation(DELETE_SCOPE_RULE);
    const removeHostFromScope = useCallback(async (host: string) => {
        try {
            // Find rules that match this host
            const matchingRules = scopeRules.filter(rule => {
                if (rule.isRegex) {
                    try {
                        return new RegExp(rule.pattern).test(host);
                    } catch { return false; }
                }
                const pattern = rule.pattern.replace(/\*/g, '.*');
                return new RegExp(`^${pattern}$`).test(host);
            });

            // Delete all matching rules
            for (const rule of matchingRules) {
                await deleteScopeRule({ variables: { id: rule.id } });
            }

            if (matchingRules.length > 0) {
                toast.success(`Removed ${matchingRules.length} scope rule(s) for ${host}`);
                refetchScope();
            } else {
                toast.info(`No scope rules found for ${host}`);
            }
        } catch (err: any) {
            toast.error(`Failed to remove rule: ${err.message}`);
        }
        setContextMenu(null);
    }, [scopeRules, deleteScopeRule, refetchScope]);

    // Phase 5: Pagination
    const limit = 10000;

    const { data: initialData, refetch, fetchMore, loading } = useQuery(GET_HTTP_TRANSACTIONS, {
        fetchPolicy: 'cache-and-network',
        variables: { limit, offset: 0 }
    });

    useSubscription(TRAFFIC_UPDATES, {
        onData: () => {
            console.log('[TrafficSub] Received update, refetching...');
            refetch();
        }
    });

    const [getDetails, { data: detailsData }] = useLazyQuery(GET_TRANSACTION_DETAILS);

    useEffect(() => {
        if (selectedId) {
            console.log(`[TrafficTree] Fetching details for ${selectedId}`);
            getDetails({ variables: { id: selectedId } });
        }
    }, [selectedId, getDetails]);

    const requests: TrafficRequest[] = initialData?.requests || [];
    const selectedRequest: TrafficRequest | undefined = detailsData?.request;

    const groupedRequests = useMemo(() => {
        console.log('[TrafficTree] Processing groupedRequests filter logic...');
        let filtered = requests;

        if (hideConnect) {
            filtered = filtered.filter(r => r.method !== 'CONNECT');
        }

        if (filterQuery || activeMethodFilter) {
            filtered = requests.filter(r => {
                const query = filterQuery.toLowerCase().trim();

                let matchesText = true;
                if (query) {
                    if (query.startsWith('s:')) {
                        const val = query.replace('s:', '').trim();
                        if (val === 'pend') {
                            matchesText = !r.status || r.status === 0;
                        } else {
                            matchesText = r.status?.toString().includes(val) || false;
                        }
                    } else if (query.startsWith('m:')) {
                        const val = query.replace('m:', '').trim();
                        matchesText = r.method?.toLowerCase().includes(val) || false;
                    } else if (query.startsWith('h:')) {
                        const val = query.replace('h:', '').trim();
                        matchesText = r.url?.toLowerCase().includes(val) || false;
                    } else {
                        matchesText = r.url?.toLowerCase().includes(query) ||
                            r.status?.toString().includes(query) ||
                            r.method?.toLowerCase().includes(query);
                    }
                }

                const matchesMethod = !activeMethodFilter || r.method === activeMethodFilter;
                return matchesText && matchesMethod;
            });
        }

        const groups: Record<string, TrafficRequest[]> = {};
        filtered.forEach(r => {
            let host = 'unknown';
            try {
                if (r.method === 'CONNECT') {
                    // For CONNECT, the URL is usually "host:port"
                    host = r.url.split(':')[0];
                } else if (r.url.includes('://')) {
                    host = new URL(r.url).hostname;
                } else {
                    // Fallback for URLs without protocol (e.g. "example.com/path")
                    host = r.url.split('/')[0].split(':')[0];
                }
            } catch {
                host = 'unknown';
            }

            if (!groups[host]) groups[host] = [];
            groups[host].push(r);
        });
        return groups;
    }, [requests, filterQuery, activeMethodFilter, hideConnect]);

    // Context Menu Handlers
    const handleContextMenu = (e: React.MouseEvent, host: string) => {
        e.preventDefault();
        setContextMenu({ x: e.clientX, y: e.clientY, host });
    };

    const handleRequestContextMenu = (e: React.MouseEvent, request: TrafficRequest) => {
        e.preventDefault();
        setContextMenu({ x: e.clientX, y: e.clientY, request });
    };

    const sendToRepeater = async (req: TrafficRequest) => {
        console.log(`[TrafficTree] Sending request ${req.requestId} to Repeater...`);
        setContextMenu(null);

        // Fetch full details if needed
        let fullReq = req;
        if (!req.requestHeaders) {
            const { data } = await client.query({
                query: GET_TRANSACTION_DETAILS,
                variables: { id: req.requestId },
                fetchPolicy: 'network-only'
            });
            if (data?.request) fullReq = data.request;
        }

        const raw = formatRequestRaw(fullReq);
        let name = 'New Request';

        // Ensure HTTPS is used for security testing targets
        // Most modern web apps use HTTPS, and HTTP requests will fail on HTTPS-only servers
        let targetUrl = fullReq.url;
        try {
            const url = new URL(fullReq.url);
            name = `${fullReq.method} ${url.pathname}`;

            // Convert http:// to https:// for security testing
            if (url.protocol === 'http:') {
                url.protocol = 'https:';
                targetUrl = url.toString();
                console.log(`[TrafficTree] Converted URL from HTTP to HTTPS: ${targetUrl}`);
            }
        } catch {
            name = `${fullReq.method} ${fullReq.url}`;
        }

        // Wait for task to be created before navigating
        // Don't pass agentId - user will select an available agent in Repeater
        await addTask({
            name,
            request: raw,
            targetUrl: targetUrl
        });

        navigate('/repeater');
    };

    const fetchAllForDomain = async (host: string) => {
        const domainReqs = groupedRequests[host] || [];
        console.log(`[TrafficTree] Fetching all for domain: ${host} (${domainReqs.length} items)`);
        setLoadingDomain(host);
        setContextMenu(null);

        for (const req of domainReqs) {
            try {
                await client.query({
                    query: GET_TRANSACTION_DETAILS,
                    variables: { id: req.requestId },
                    fetchPolicy: 'network-only'
                });
            } catch (err) {
                console.error(`Failed to fetch details for ${req.requestId}:`, err);
            }
        }
        setLoadingDomain(null);
    };

    const handleDeleteHostRequests = async (host: string) => {
        if (!window.confirm(`Are you sure you want to delete all requests for ${host}?`)) return;
        console.log(`[TrafficTree] Deleting all requests for domain: ${host}`);
        setContextMenu(null);
        try {
            await deleteByHost({ variables: { host } });
        } catch (err) {
            console.error('Failed to delete requests:', err);
        }
    };

    const loadMore = () => {
        const currentCount = requests.length;
        console.log(`[TrafficTree] Loading more items from DB (offset: ${currentCount})...`);
        fetchMore({
            variables: {
                limit: 10000,
                offset: currentCount
            }
        });
    };

    useEffect(() => {
        const handleClick = () => setContextMenu(null);
        window.addEventListener('click', handleClick);
        return () => window.removeEventListener('click', handleClick);
    }, []);

    // Keyboard shortcuts for scope management
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            // Get host from context menu OR selected host
            const targetHost = contextMenu?.host || selectedHost;
            if (!targetHost) return;

            // Cmd+S or Ctrl+S = Add to Scope (Include)
            if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key.toLowerCase() === 's') {
                e.preventDefault();
                addHostToScope(targetHost, 'Include');
            }

            // Cmd+Shift+E or Ctrl+Shift+E = Add to Out-of-Scope (Exclude)
            if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 'e') {
                e.preventDefault();
                addHostToScope(targetHost, 'Exclude');
            }

            // Cmd+R or Ctrl+R = Remove from Scope
            if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key.toLowerCase() === 'r') {
                e.preventDefault();
                removeHostFromScope(targetHost);
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [contextMenu, selectedHost, addHostToScope, removeHostFromScope]);

    return (
        <div className="h-screen flex flex-col bg-[#0B0D11] text-slate-200 overflow-hidden font-sans">
            {/* Custom Context Menu */}
            {contextMenu && (
                <div
                    className="fixed z-[100] bg-[#161922] border border-white/10 shadow-2xl rounded-md py-1 min-w-[200px] backdrop-blur-xl ring-1 ring-white/5"
                    style={{ top: contextMenu.y, left: contextMenu.x }}
                >
                    {contextMenu.host && (
                        <>
                            {/* Scope Management Options */}
                            <button
                                onClick={() => addHostToScope(contextMenu.host!, 'Include')}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-emerald-400 hover:bg-emerald-500/10 transition-colors text-left"
                            >
                                <Target size={12} />
                                ADD TO SCOPE
                                <span className="ml-auto text-[9px] text-white/30">⌘S</span>
                            </button>
                            <button
                                onClick={() => addHostToScope(contextMenu.host!, 'Exclude')}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-red-400 hover:bg-red-500/10 transition-colors text-left"
                            >
                                <ShieldOff size={12} />
                                ADD TO OUT-OF-SCOPE
                                <span className="ml-auto text-[9px] text-white/30">⌘⇧E</span>
                            </button>
                            <button
                                onClick={() => removeHostFromScope(contextMenu.host!)}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-orange-400 hover:bg-orange-500/10 transition-colors text-left"
                            >
                                <Trash2 size={12} />
                                REMOVE FROM SCOPE
                                <span className="ml-auto text-[9px] text-white/30">⌘R</span>
                            </button>
                            <div className="h-px bg-white/5 my-1" />
                            <button
                                onClick={() => fetchAllForDomain(contextMenu.host!)}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-slate-300 hover:bg-cyan-500/10 hover:text-cyan-400 transition-colors text-left"
                            >
                                <RefreshCw size={12} />
                                FETCH ALL RESPONSES
                            </button>
                            <button
                                onClick={() => {
                                    navigator.clipboard.writeText(contextMenu.host!);
                                    setContextMenu(null);
                                }}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-slate-300 hover:bg-white/5 hover:text-white transition-colors text-left"
                            >
                                <Copy size={12} />
                                COPY DOMAIN
                            </button>
                            <div className="h-px bg-white/5 my-1" />
                            <button
                                onClick={() => handleDeleteHostRequests(contextMenu.host!)}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-red-400 hover:bg-red-500/10 transition-colors text-left"
                            >
                                <Trash2 size={12} />
                                DELETE ALL REQUESTS
                            </button>
                        </>
                    )}

                    {contextMenu.request && (
                        <>
                            <button
                                onClick={() => sendToRepeater(contextMenu.request!)}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-cyan-400 hover:bg-cyan-500/10 transition-colors text-left"
                            >
                                <Send size={12} />
                                SEND TO REPEATER
                            </button>
                            <button
                                onClick={() => {
                                    // TODO: Send to Intruder
                                    setContextMenu(null);
                                }}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-slate-300 hover:bg-white/5 hover:text-white transition-colors text-left opacity-50"
                            >
                                <Layers size={12} />
                                SEND TO INTRUDER
                            </button>
                            <div className="h-px bg-white/5 my-1" />
                            <button
                                onClick={() => {
                                    navigator.clipboard.writeText(contextMenu.request!.url);
                                    setContextMenu(null);
                                }}
                                className="w-full flex items-center gap-3 px-3 py-2 text-[11px] font-bold text-slate-300 hover:bg-white/5 hover:text-white transition-colors text-left"
                            >
                                <Copy size={12} />
                                COPY URL
                            </button>
                        </>
                    )}
                </div>
            )}

            <TrafficToolbar
                filterQuery={filterQuery}
                setFilterQuery={setFilterQuery}
                activeMethodFilter={activeMethodFilter}
                setActiveMethodFilter={setActiveMethodFilter}
                totalItems={requests.length}
                hostCount={Object.keys(groupedRequests).length}
                hideConnect={hideConnect}
                setHideConnect={setHideConnect}
            />

            <ResizablePanelGroup direction="horizontal" className="flex-1 overflow-hidden">
                {/* Left Panel: Sidebar */}
                <ResizablePanel defaultSize={25} minSize={15} className="border-r border-white/5 flex flex-col">
                    <div className="flex-1 overflow-hidden">
                        <TrafficSidebar
                            groupedRequests={groupedRequests}
                            selectedId={selectedId}
                            setSelectedId={setSelectedId}
                            handleContextMenu={handleContextMenu}
                            handleRequestContextMenu={handleRequestContextMenu}
                            loadingDomain={loadingDomain}
                            getHostScopeStatus={getHostScopeStatus}
                            onHostSelect={setSelectedHost}
                        />
                    </div>
                    <button
                        onClick={loadMore}
                        disabled={loading}
                        className="h-10 border-t border-white/5 bg-black/40 hover:bg-white/5 text-[10px] font-black tracking-widest text-slate-500 hover:text-cyan-400 transition-all flex items-center justify-center gap-2 "
                    >
                        {loading && <RefreshCw size={12} className="animate-spin" />}
                        LOAD OLDER REQUESTS FROM DB
                    </button>
                </ResizablePanel>

                <ResizableHandle className="w-[1px] bg-white/10 hover:bg-cyan-500/30 transition-all cursor-col-resize shadow-[0_0_10px_rgba(0,0,0,0.5)]" />

                {/* Right Panel: Inspector */}
                <ResizablePanel defaultSize={75} className="bg-gradient-to-br from-[#0B0D11] to-[#0E1015]">
                    {selectedId ? (
                        <RequestInspector request={selectedRequest} />
                    ) : (
                        <div className="h-full flex flex-col items-center justify-center gap-6">
                            <RefreshCw className="w-16 h-16 text-slate-800/50" />
                            <div className="text-center space-y-3">
                                <h3 className="text-[11px] font-black uppercase tracking-[0.5em] text-slate-500">Awaiting Signal</h3>
                                <p className="text-[10px] text-slate-700 font-mono tracking-wider uppercase">Select a transaction to begin analysis</p>
                            </div>
                        </div>
                    )}
                </ResizablePanel>
            </ResizablePanelGroup>
        </div>
    );
};

export default TrafficTreePage;