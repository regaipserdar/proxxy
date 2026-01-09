import { useState, useMemo, useRef, useCallback, useEffect } from 'react';
import {
  Search, Trash2, Download,
  Copy, Database, Code, X
} from 'lucide-react';
import { useRequests } from '@/hooks/useRequests';

export const ProxyView = () => {
  const { requests, clearRequests } = useRequests();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'request' | 'response'>('request');
  const [filterQuery, setFilterQuery] = useState('');
  const [contentSearch, setContentSearch] = useState('');

  // Resizable Panel Logic
  const [panelWidth, setPanelWidth] = useState(550);
  const isResizing = useRef(false);

  const startResizing = useCallback(() => {
    isResizing.current = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }, []);

  const stopResizing = useCallback(() => {
    isResizing.current = false;
    document.body.style.cursor = 'default';
    document.body.style.userSelect = 'auto';
  }, []);

  const onResize = useCallback((e: MouseEvent) => {
    if (!isResizing.current) return;
    const newWidth = window.innerWidth - e.clientX;
    if (newWidth > 300 && newWidth < 1200) {
      setPanelWidth(newWidth);
    }
  }, []);

  useEffect(() => {
    window.addEventListener('mousemove', onResize);
    window.addEventListener('mouseup', stopResizing);
    return () => {
      window.removeEventListener('mousemove', onResize);
      window.removeEventListener('mouseup', stopResizing);
    };
  }, [onResize, stopResizing]);

  const selectedRequest = useMemo(() =>
    requests.find(r => r.id === selectedId), [requests, selectedId]
  );

  // Simulated GQL Filtering for the main table
  const filteredRequests = useMemo(() => {
    if (!filterQuery) return requests;

    const terms = filterQuery.toLowerCase().split(' ');
    return requests.filter(r => {
      return terms.every(term => {
        if (term.includes(':')) {
          const [key, val] = term.split(':');
          if (key === 'method') return r.method.toLowerCase() === val;
          if (key === 'status') return r.status.toString() === val;
          if (key === 'host') return r.host.toLowerCase().includes(val);
          if (key === 'path') return r.path.toLowerCase().includes(val);
        }
        return r.path.toLowerCase().includes(term) || r.host.toLowerCase().includes(term) || r.method.toLowerCase().includes(term);
      });
    });
  }, [requests, filterQuery]);

  const currentContent = useMemo(() => {
    if (!selectedRequest) return '';
    return activeTab === 'request' ? selectedRequest.rawRequest : selectedRequest.rawResponse;
  }, [selectedRequest, activeTab]);

  return (
    <div className="flex h-full w-full bg-[#0A0E14] overflow-hidden select-none">
      {/* Table Side */}
      <div className="flex-1 flex flex-col min-w-0">
        <div className="h-14 border-b border-white/10 flex items-center justify-between px-4 bg-[#111318]">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 px-3 py-1.5 bg-[#9DCDE8]/5 border border-[#9DCDE8]/10 rounded-lg">
              <Database size={12} className="text-[#9DCDE8]" />
              <span className="text-[10px] font-bold text-[#9DCDE8] uppercase tracking-[0.15em]">Intercept Active</span>
            </div>
            <div className="relative group">
              <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-[#9DCDE8] transition-colors" />
              <input
                type="text"
                placeholder="GraphQL Query: method:POST status:200..."
                value={filterQuery}
                onChange={(e) => setFilterQuery(e.target.value)}
                className="bg-black/60 border border-white/10 rounded-lg pl-9 pr-12 py-1.5 text-xs text-white/80 focus:outline-none focus:border-[#9DCDE8]/40 w-[350px] transition-all font-mono"
              />
              <div className="absolute right-3 top-1/2 -translate-y-1/2 opacity-20 hover:opacity-100 transition-opacity">
                <Code size={12} className="text-[#9DCDE8]" />
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button onClick={() => {
              const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(requests, null, 2));
              const downloadAnchorNode = document.createElement('a');
              downloadAnchorNode.setAttribute("href", dataStr);
              downloadAnchorNode.setAttribute("download", "traffic_export.json");
              document.body.appendChild(downloadAnchorNode);
              downloadAnchorNode.click();
              downloadAnchorNode.remove();
            }} className="p-2 hover:bg-white/5 rounded-lg text-white/20 hover:text-white transition-colors" title="Export to JSON">
              <Download size={16} />
            </button>
            <button onClick={clearRequests} className="p-2 hover:bg-white/5 rounded-lg text-white/20 hover:text-white transition-colors" title="Clear all logs">
              <Trash2 size={16} />
            </button>
          </div>
        </div>

        <div className="flex-1 relative overflow-hidden">
          <VirtualTable
            data={filteredRequests}
            selectedId={selectedId}
            onSelect={setSelectedId}
          />
        </div>
      </div>

      {/* Resize Handle */}
      <div
        onMouseDown={startResizing}
        className="w-1.5 bg-white/5 hover:bg-[#9DCDE8]/40 cursor-col-resize transition-all flex items-center justify-center group relative z-30"
      >
        <div className="w-[1px] h-12 bg-white/10 group-hover:bg-[#9DCDE8]/50" />
      </div>

      {/* Inspection Panel */}
      <div style={{ width: panelWidth }} className="flex flex-col bg-[#0D0F13] shadow-[-20px_0_40px_rgba(0,0,0,0.6)] z-20">
        <div className="h-14 border-b border-white/10 flex items-center px-4 bg-[#111318] gap-4">
          <div className="flex gap-1">
            <TabButton active={activeTab === 'request'} onClick={() => setActiveTab('request')}>Request</TabButton>
            <TabButton active={activeTab === 'response'} onClick={() => setActiveTab('response')}>Response</TabButton>
          </div>

          {/* In-Panel Search */}
          <div className="flex-1 max-w-[200px] ml-4 relative group">
            <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-[#9DCDE8] transition-colors" />
            <input
              type="text"
              placeholder={`Search in ${activeTab}...`}
              value={contentSearch}
              onChange={(e) => setContentSearch(e.target.value)}
              className="w-full bg-black/40 border border-white/5 rounded-md pl-8 pr-2 py-1 text-[10px] text-white/60 focus:outline-none focus:border-[#9DCDE8]/30 font-mono transition-all"
            />
            {contentSearch && (
              <button
                onClick={() => setContentSearch('')}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-white/20 hover:text-white"
              >
                <X size={10} />
              </button>
            )}
          </div>

          <div className="ml-auto flex gap-1">
            <button className="p-2 rounded-lg text-white/20 hover:text-white hover:bg-white/5" title="Copy Raw"><Copy size={14} /></button>
            <button className="p-2 rounded-lg text-white/20 hover:text-white hover:bg-white/5" title="Send to Repeater"><RepeatIcon size={14} /></button>
          </div>
        </div>

        <div className="flex-1 overflow-auto bg-[#080A0E] relative">
          {selectedRequest ? (
            <div className="h-full flex flex-col">
              <div className="p-4 border-b border-white/5 bg-white/[0.02]">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-3">
                    <span className={`px-2 py-0.5 rounded text-[10px] font-bold ${getMethodColor(selectedRequest.method)}`}>
                      {selectedRequest.method}
                    </span>
                    <span className="text-white/40 text-[10px] font-mono tracking-tighter truncate max-w-[250px]">{selectedRequest.host}</span>
                  </div>
                  <span className="text-[10px] font-mono opacity-20">{selectedRequest.timestamp}</span>
                </div>
                <h3 className="text-xs font-mono text-white/90 break-all">{selectedRequest.path}</h3>
              </div>
              <div className="flex-1 overflow-auto">
                <CodeViewer
                  content={currentContent}
                  search={contentSearch}
                />
              </div>
            </div>
          ) : (
            <div className="h-full flex flex-col items-center justify-center opacity-10 gap-6">
              <div className="w-20 h-20 rounded-3xl border-2 border-dashed border-white flex items-center justify-center animate-pulse">
                <Search size={40} />
              </div>
              <p className="text-xs font-bold uppercase tracking-[0.3em]">Query Analysis Pending</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

const RepeatIcon = ({ size }: { size: number }) => <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m17 2 4 4-4 4" /><path d="M3 11v-1a4 4 0 0 1 4-4h14" /><path d="m7 22-4-4 4-4" /><path d="M21 13v1a4 4 0 0 1-4 4H3" /></svg>;

const TabButton = ({ children, active, onClick }: any) => (
  <button
    onClick={onClick}
    className={`px-4 py-1.5 rounded-lg text-[10px] font-bold uppercase tracking-[0.1em] transition-all border ${active ? 'bg-[#9DCDE8]/10 text-[#9DCDE8] border-[#9DCDE8]/20 shadow-[0_0_15px_rgba(157,205,232,0.1)]' : 'text-white/20 border-transparent hover:text-white/40'
      }`}
  >
    {children}
  </button>
);

const VirtualTable = ({ data, selectedId, onSelect }: any) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const rowHeight = 36;
  const viewportHeight = 800;

  const onScroll = (e: any) => setScrollTop(e.currentTarget.scrollTop);

  const startIndex = Math.max(0, Math.floor(scrollTop / rowHeight) - 5);
  const endIndex = Math.min(data.length, Math.floor((scrollTop + viewportHeight) / rowHeight) + 5);
  const visibleRows = data.slice(startIndex, endIndex);

  return (
    <div ref={containerRef} onScroll={onScroll} className="h-full overflow-auto relative scroll-smooth bg-[#0A0E14]">
      <div className="sticky top-0 z-20 bg-[#111318]/95 backdrop-blur-md border-b border-white/5 flex text-[9px] font-bold uppercase tracking-[0.15em] text-white/20 h-9 items-center px-2">
        <div className="w-16 px-2">Index</div>
        <div className="w-20 px-2">Verb</div>
        <div className="w-48 px-2">Authority</div>
        <div className="flex-1 px-2">Resource Path</div>
        <div className="w-24 px-2">Status</div>
        <div className="w-28 px-2 text-right">Elapsed</div>
      </div>

      <div style={{ height: data.length * rowHeight }} className="relative">
        {visibleRows.map((item: any, idx: number) => (
          <div
            key={item.id}
            onClick={() => onSelect(item.id)}
            style={{
              position: 'absolute',
              top: (startIndex + idx) * rowHeight,
              height: rowHeight,
              width: '100%'
            }}
            className={`flex items-center text-[11px] border-b border-white/[0.02] hover:bg-white/[0.04] cursor-pointer group transition-all ${selectedId === item.id ? 'bg-[#9DCDE8]/5 text-white' : 'text-white/50'
              }`}
          >
            <div className={`absolute left-0 w-1 h-full transition-all ${selectedId === item.id ? 'bg-[#9DCDE8]' : 'bg-transparent group-hover:bg-white/10'}`} />
            <div className="w-16 px-4 font-mono text-[9px] opacity-20">{startIndex + idx + 1}</div>
            <div className="w-20 px-2">
              <span className={`px-1.5 py-0.5 rounded-[3px] text-[8px] font-bold ${getMethodColor(item.method)}`}>
                {item.method}
              </span>
            </div>
            <div className={`w-48 px-2 truncate transition-colors ${selectedId === item.id ? 'text-[#9DCDE8]' : 'text-white/60'}`}>{item.host}</div>
            <div className="flex-1 px-2 truncate opacity-40 font-mono tracking-tight">{item.path}</div>
            <div className="w-24 px-2 font-bold font-mono">
              <span className={item.status < 300 ? 'text-emerald-500' : item.status < 500 ? 'text-amber-500' : 'text-red-500'}>
                {item.status}
              </span>
            </div>
            <div className="w-28 px-2 text-right text-[10px] font-mono opacity-20">24ms</div>
          </div>
        ))}
      </div>
    </div>
  );
};

const CodeViewer = ({ content, search }: { content: string, search: string }) => {
  const highlightSearch = (text: string) => {
    if (!search || !text) return <span>{text}</span>;
    const parts = text.split(new RegExp(`(${search})`, 'gi'));
    return (
      <span>
        {parts.map((part, i) =>
          part.toLowerCase() === search.toLowerCase() ? (
            <mark key={i} className="bg-[#9DCDE8] text-black rounded-sm px-0.5">{part}</mark>
          ) : (
            part
          )
        )}
      </span>
    );
  };

  return (
    <pre className="p-6 font-mono leading-relaxed select-text text-[12px] whitespace-pre-wrap break-all">
      {content.split('\n').map((line, i) => {
        const isHeader = line.includes(': ');
        return (
          <div key={i} className="flex gap-6 group hover:bg-white/[0.02] -mx-6 px-6 transition-colors">
            <span className="w-8 text-right opacity-10 select-none text-[10px] font-mono shrink-0">{i + 1}</span>
            <span className={isHeader ? 'text-[#9DCDE8]/80' : 'text-white/60'}>
              {highlightSearch(line)}
            </span>
          </div>
        );
      })}
    </pre>
  );
};

const getMethodColor = (method: string) => {
  switch (method) {
    case 'GET': return 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/10';
    case 'POST': return 'bg-[#9DCDE8]/10 text-[#9DCDE8] border border-[#9DCDE8]/10';
    case 'PUT': return 'bg-amber-500/10 text-amber-400 border border-amber-500/10';
    case 'DELETE': return 'bg-red-500/10 text-red-400 border border-red-500/10';
    default: return 'bg-white/5 text-white/40 border border-white/5';
  }
};
