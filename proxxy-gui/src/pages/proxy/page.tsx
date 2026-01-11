import { useState, useMemo, useEffect, useRef } from 'react';
import { useQuery, useSubscription, useLazyQuery } from '@apollo/client';
import {
  Search, Trash2, Copy, Database, X,
  Pause, Play, Loader2, Clock, Globe
} from 'lucide-react';
import {
  GET_HTTP_TRANSACTIONS,
  TRAFFIC_UPDATES,
  GET_TRANSACTION_DETAILS
} from '@/graphql/operations';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ScrollArea } from '@/components/ui/scroll-area';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable';
import { format } from 'date-fns';

// --- Helper Functions ---
const formatTime = (ts: string | number) => {
  try {
    const date = new Date(typeof ts === 'number' ? ts * 1000 : ts);
    return format(date, 'HH:mm:ss.SSS');
  } catch { return '--:--:--'; }
};

const getMethodColor = (method: string) => {
  switch (method?.toUpperCase()) {
    case 'GET': return 'text-blue-400 bg-blue-500/10 border-blue-500/20';
    case 'POST': return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
    case 'PUT': return 'text-orange-400 bg-orange-500/10 border-orange-500/20';
    case 'DELETE': return 'text-red-400 bg-red-500/10 border-red-500/20';
    default: return 'text-slate-400 bg-slate-500/10 border-slate-500/20';
  }
};

const getStatusColor = (status: number) => {
  if (status < 300) return 'text-emerald-400';
  if (status < 400) return 'text-blue-400';
  if (status < 500) return 'text-amber-400';
  return 'text-red-500';
};

// --- Components ---

// Code Viewer Component
const CodeViewer = ({ content }: { content: string }) => {
  return (
    <ScrollArea className="h-full w-full">
      <div className="relative group">
        <div className="p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap break-all text-slate-300 selection:bg-blue-500/30">
          {content}
        </div>
        <Button
          variant="ghost"
          size="sm"
          className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity bg-black/50 hover:bg-blue-500 text-white h-8 w-8 p-0"
          onClick={() => navigator.clipboard.writeText(content)}
        >
          <Copy className="w-4 h-4" />
        </Button>
      </div>
    </ScrollArea>
  );
};

// Request Inspector Component
const RequestInspector = ({ data, loading, baseInfo }: { data: any, loading: boolean, baseInfo: any }) => {
  if (loading) return <div className="h-full flex items-center justify-center"><Loader2 className="animate-spin text-blue-500" /></div>;
  if (!baseInfo) return null;

  const requestContent = data?.requestHeaders ? `${data.requestHeaders}\n\n${data.requestBody || ''}` : 'Loading details...';
  const responseContent = data?.responseHeaders ? `${data.responseHeaders}\n\n${data.responseBody || ''}` : 'Loading details...';

  return (
    <div className="h-full flex flex-col bg-[#0E1015]">
      {/* Detail Header */}
      <div className="p-4 border-b border-white/5 bg-[#111318]">
        <div className="flex items-center gap-3 mb-2">
          <Badge variant="outline" className={`${getMethodColor(baseInfo.method)} text-xs border-0`}>{baseInfo.method}</Badge>
          <span className="font-mono text-sm text-slate-300 truncate flex-1" title={baseInfo.url}>{baseInfo.url}</span>
        </div>
        <div className="flex items-center gap-4 text-xs text-slate-500 font-mono">
          <div className="flex items-center gap-1"><Clock className="w-3 h-3" /> {formatTime(baseInfo.timestamp)}</div>
          <div className="flex items-center gap-1"><Globe className="w-3 h-3" /> {new URL(baseInfo.url).hostname}</div>
          <div className={`font-bold ${getStatusColor(baseInfo.status)}`}>{baseInfo.status}</div>
        </div>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="request" className="flex-1 flex flex-col overflow-hidden">
        <div className="px-4 border-b border-white/5 bg-[#0B0D11]">
          <TabsList className="h-10 bg-transparent gap-4 p-0">
            <TabsTrigger
              value="request"
              className="data-[state=active]:bg-transparent data-[state=active]:border-b-2 data-[state=active]:border-blue-500 data-[state=active]:text-blue-400 rounded-none px-0 text-xs font-bold uppercase tracking-wider text-slate-500 shadow-none border-b-2 border-transparent"
            >
              Request
            </TabsTrigger>
            <TabsTrigger
              value="response"
              className="data-[state=active]:bg-transparent data-[state=active]:border-b-2 data-[state=active]:border-emerald-500 data-[state=active]:text-emerald-400 rounded-none px-0 text-xs font-bold uppercase tracking-wider text-slate-500 shadow-none border-b-2 border-transparent"
            >
              Response
            </TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="request" className="flex-1 m-0 overflow-hidden relative group data-[state=active]:flex flex-col">
          <CodeViewer content={requestContent} />
        </TabsContent>

        <TabsContent value="response" className="flex-1 m-0 overflow-hidden relative group data-[state=active]:flex flex-col">
          <CodeViewer content={responseContent} />
        </TabsContent>
      </Tabs>
    </div>
  );
};

// Main Page Component
export const ProxyPage = () => {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [filterQuery, setFilterQuery] = useState('');
  const [isLive, setIsLive] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  // 1. List Data (Initial Load)
  const { data: initialData, loading, refetch } = useQuery(GET_HTTP_TRANSACTIONS, {
    fetchPolicy: 'cache-and-network',
    variables: { limit: 100 }
  });

  // 2. Live Data Stream (Subscription)
  useSubscription(TRAFFIC_UPDATES, {
    onData: () => {
      if (!isLive) return;
      // Scroll to bottom logic can be added here if needed
    }
  });

  // 3. Detail Data (Lazy Load)
  const [getDetails, { data: detailsData, loading: detailsLoading }] = useLazyQuery(GET_TRANSACTION_DETAILS);

  useEffect(() => {
    if (selectedId) {
      getDetails({ variables: { requestId: selectedId } });
    }
  }, [selectedId, getDetails]);

  const requests = initialData?.requests || [];
  const selectedRequest = detailsData?.request;

  const filteredRequests = useMemo(() => {
    if (!filterQuery) return requests;
    const terms = filterQuery.toLowerCase().split(' ');

    return requests.filter((r: any) => {
      return terms.every(term => {
        if (term.includes(':')) {
          const [key, val] = term.split(':');
          if (key === 'method') return r.method?.toLowerCase() === val;
          if (key === 'status') return r.status?.toString() === val;
          if (key === 'url') return r.url?.toLowerCase().includes(val);
        }
        return (
          r.url?.toLowerCase().includes(term) ||
          r.method?.toLowerCase().includes(term) ||
          r.requestId.includes(term)
        );
      });
    });
  }, [requests, filterQuery]);

  const handleClear = () => {
    refetch();
  };

  return (
    <div className="h-full w-full bg-[#0B0D11] text-slate-200 font-sans flex flex-col overflow-hidden">

      {/* HEADER TOOLBAR */}
      <div className="h-14 border-b border-white/5 bg-[#111318] flex items-center justify-between px-4 shrink-0">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2 text-emerald-400">
            <Database className="w-5 h-5" />
            <h1 className="font-bold tracking-tight">Proxy History</h1>
          </div>

          <div className="h-6 w-[1px] bg-white/10 mx-2" />

          <div className="relative group">
            <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 group-focus-within:text-emerald-400 transition-colors" />
            <input
              type="text"
              placeholder="Filter (method:POST status:200...)"
              value={filterQuery}
              onChange={(e) => setFilterQuery(e.target.value)}
              className="bg-black/20 border border-white/10 rounded-lg pl-9 pr-4 py-1.5 text-xs w-64 focus:outline-none focus:border-emerald-500/50 transition-all font-mono"
            />
            {filterQuery && (
              <button onClick={() => setFilterQuery('')} className="absolute right-2 top-1/2 -translate-y-1/2 text-slate-500 hover:text-white">
                <X className="w-3 h-3" />
              </button>
            )}
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsLive(!isLive)}
            className={`h-8 gap-2 border ${isLive ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-400' : 'border-white/10 bg-white/5 text-slate-400'}`}
          >
            {isLive ? <Pause className="w-3.5 h-3.5" /> : <Play className="w-3.5 h-3.5" />}
            <span className="text-xs font-bold">{isLive ? 'LIVE' : 'PAUSED'}</span>
          </Button>

          <Button variant="ghost" size="sm" onClick={handleClear} className="h-8 w-8 p-0 hover:bg-red-500/10 hover:text-red-400">
            <Trash2 className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* MAIN CONTENT SPLIT VIEW */}
      <div className="flex-1 overflow-hidden">
        {/* FIX: Use 'horizontal' not 'direction="horizontal"' if using newer version, 
            or 'direction="horizontal"' for older. 
            Using 'direction="horizontal"' as it is standard for vertical split (panels side by side) 
        */}
        <ResizablePanelGroup orientation="horizontal">

          {/* LEFT PANEL: REQUEST LIST */}
          <ResizablePanel defaultSize={40} minSize={25} className="bg-[#0E1015]">
            <div className="h-full flex flex-col">
              {/* Table Header */}
              <div className="grid grid-cols-[60px_80px_1fr_60px] gap-2 px-4 py-2 text-[10px] font-bold text-slate-500 uppercase tracking-wider border-b border-white/5 bg-[#111318]">
                <span>ID</span>
                <span>Method</span>
                <span>URL</span>
                <span className="text-right">Status</span>
              </div>

              {/* Scrollable List */}
              <div className="flex-1 overflow-y-auto custom-scrollbar" ref={scrollRef}>
                {loading ? (
                  <div className="flex justify-center py-10"><Loader2 className="animate-spin text-slate-600" /></div>
                ) : filteredRequests.length === 0 ? (
                  <div className="flex flex-col items-center justify-center h-full text-slate-600 gap-2">
                    <Search className="w-8 h-8 opacity-20" />
                    <span className="text-xs">No traffic found</span>
                  </div>
                ) : (
                  filteredRequests.map((req: any, i: number) => (
                    <div
                      key={req.requestId || i}
                      onClick={() => setSelectedId(req.requestId)}
                      className={`grid grid-cols-[60px_80px_1fr_60px] gap-2 px-4 py-2 border-b border-white/[0.02] cursor-pointer transition-colors text-xs font-mono hover:bg-white/[0.02] ${selectedId === req.requestId ? 'bg-blue-500/10 border-l-2 border-l-blue-500' : 'border-l-2 border-l-transparent'
                        }`}
                    >
                      <span className="text-slate-500 truncate">#{req.requestId.slice(-4)}</span>
                      <span className={`px-1.5 py-0.5 rounded w-fit text-[10px] font-bold border ${getMethodColor(req.method)}`}>
                        {req.method}
                      </span>
                      <span className="text-slate-300 truncate" title={req.url}>{req.url}</span>
                      <span className={`text-right font-bold ${getStatusColor(req.status)}`}>
                        {req.status || '...'}
                      </span>
                    </div>
                  ))
                )}
              </div>

              {/* Footer Status */}
              <div className="px-3 py-1 bg-[#0B0D11] border-t border-white/5 text-[10px] text-slate-500 flex justify-between font-mono">
                <span>{filteredRequests.length} Requests</span>
                <span>{formatTime(Date.now())}</span>
              </div>
            </div>
          </ResizablePanel>

          <ResizableHandle withHandle className="bg-white/5 hover:bg-blue-500/50 transition-colors w-1" />

          {/* RIGHT PANEL: INSPECTOR */}
          <ResizablePanel defaultSize={60}>
            {selectedId ? (
              <RequestInspector
                data={selectedRequest}
                loading={detailsLoading}
                baseInfo={requests.find((r: any) => r.requestId === selectedId)}
              />
            ) : (
              <div className="h-full flex flex-col items-center justify-center text-slate-600 bg-[#0B0D11]">
                <Database className="w-12 h-12 opacity-20 mb-4" />
                <p className="text-sm font-medium">Select a request to inspect details</p>
              </div>
            )}
          </ResizablePanel>

        </ResizablePanelGroup>
      </div>
    </div>
  );
};