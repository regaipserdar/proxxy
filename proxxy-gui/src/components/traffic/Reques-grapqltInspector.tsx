import { useState } from 'react';
import { Copy, CheckCircle2, Braces } from 'lucide-react';
import { TrafficRequest, getMethodColor, getStatusColor, formatTime, parseHeaders } from './types-grapql';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ScrollArea } from '@/components/ui/scroll-area';
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable';

// ============= SUB-COMPONENTS =============

const HeaderTable = ({ headersJson }: { headersJson: string | undefined }) => {
    const [copied, setCopied] = useState(false);
    const headers = parseHeaders(headersJson);

    const copyAll = () => {
        navigator.clipboard.writeText(JSON.stringify(headers, null, 2));
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="rounded-md border border-white/5 overflow-hidden group/table relative bg-black/20">
            <button
                onClick={copyAll}
                className="absolute right-2 top-2 p-1 bg-slate-800/80 hover:bg-slate-700 rounded text-slate-400 opacity-0 group-hover/table:opacity-100 transition-opacity z-10"
                title="Copy All Headers"
            >
                {copied ? <CheckCircle2 size={12} className="text-emerald-400" /> : <Copy size={12} />}
            </button>
            <table className="w-full text-[12px] font-mono border-collapse">
                <thead>
                    <tr className="bg-white/5 text-slate-400 text-left border-b border-white/5">
                        <th className="px-3 py-2 font-semibold w-1/3 text-[10px] uppercase tracking-wider">Key</th>
                        <th className="px-3 py-2 font-semibold text-[10px] uppercase tracking-wider">Value</th>
                    </tr>
                </thead>
                <tbody>
                    {Object.entries(headers).length > 0 ? (
                        Object.entries(headers).map(([key, val]) => (
                            <tr key={key} className="border-b border-white/[0.02] hover:bg-white/[0.02] group/row">
                                <td className="px-3 py-1.5 text-cyan-400 select-all font-bold align-top border-r border-white/5">{key}</td>
                                <td className="px-3 py-1.5 text-slate-300 break-all select-all">{val}</td>
                            </tr>
                        ))
                    ) : (
                        <tr>
                            <td colSpan={2} className="px-3 py-4 text-center text-slate-600 italic">No headers found</td>
                        </tr>
                    )}
                </tbody>
            </table>
        </div>
    );
};

const BodyViewer = ({ content, language = 'json' }: { content: string | undefined, language?: string }) => {
    const [copied, setCopied] = useState(false);
    if (!content) return <div className="text-slate-600 italic text-center py-12 border border-dashed border-white/10 rounded-md bg-black/10">No body content available</div>;

    let displayContent = content;
    let isFormatted = false;
    try {
        if (language === 'json' || (content.trim().startsWith('{') || content.trim().startsWith('['))) {
            displayContent = JSON.stringify(JSON.parse(content), null, 4);
            isFormatted = true;
        }
    } catch { }

    const copyToClipboard = () => {
        navigator.clipboard.writeText(displayContent);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="relative group rounded-md border border-white/5 bg-black/40 overflow-hidden flex flex-col h-full min-h-[400px]">
            <div className="absolute right-4 top-4 flex gap-2 z-20">
                {isFormatted && (
                    <div className="px-2 py-1 bg-emerald-500/10 border border-emerald-500/20 rounded text-[9px] font-black text-emerald-400 uppercase tracking-widest">
                        JSON Pretty
                    </div>
                )}
                <button
                    onClick={copyToClipboard}
                    className="p-1.5 bg-slate-800/90 hover:bg-slate-700 rounded text-slate-400 border border-white/10 shadow-xl backdrop-blur-md transition-all active:scale-95"
                    title="Copy Content"
                >
                    {copied ? <CheckCircle2 size={14} className="text-emerald-400" /> : <Copy size={14} />}
                </button>
            </div>
            <div className="flex-1 overflow-auto p-4 custom-scrollbar">
                <pre className="text-[13px] font-mono leading-relaxed text-slate-300 select-all whitespace-pre-wrap break-all">
                    {displayContent}
                </pre>
            </div>
        </div>
    );
};

const JsonTreeView = ({ data, label = "root" }: { data: any, label?: string }) => {
    const [collapsed, setCollapsed] = useState(true);
    const isObject = typeof data === 'object' && data !== null;

    if (!isObject) {
        return (
            <div className="flex items-center gap-2 py-0.5 ml-4 border-l border-white/5 pl-2">
                <span className="text-slate-500 text-[11px] font-mono">{label}:</span>
                <span className="text-emerald-400 text-[11px] font-mono">{JSON.stringify(data)}</span>
            </div>
        );
    }

    return (
        <div className="ml-4">
            <div
                className="flex items-center gap-1.5 py-0.5 cursor-pointer hover:text-cyan-400 transition-colors group"
                onClick={() => setCollapsed(!collapsed)}
            >
                {collapsed ? <Braces size={10} className="text-slate-600" /> : <Braces size={10} className="text-amber-500 opacity-70" />}
                <span className="text-[11px] font-bold font-mono text-slate-300 group-hover:text-cyan-400">{label}</span>
                <span className="text-[9px] text-slate-600 font-mono">({Object.keys(data).length})</span>
            </div>
            {!collapsed && (
                <div className="border-l border-white/5 ml-1 flex flex-col pt-0.5">
                    {Object.entries(data).map(([key, val]) => (
                        <JsonTreeView key={key} data={val} label={key} />
                    ))}
                </div>
            )}
        </div>
    );
};

// ============= MAIN INSPECTOR =============

interface RequestInspectorProps {
    request: TrafficRequest | undefined;
}

export const RequestInspector = ({ request }: RequestInspectorProps) => {
    if (!request) return null;

    const copyTrigger = (text: string) => {
        navigator.clipboard.writeText(text);
    };

    return (
        <div className="h-full flex flex-col overflow-hidden animate-in fade-in slide-in-from-right-4 duration-500">
            {/* Header Info */}
            <div className="px-6 py-4 bg-black/30 border-b border-white/5 backdrop-blur-md flex items-center justify-between">
                <div className="flex items-center gap-6 flex-1 min-w-0">
                    <div className="flex items-baseline gap-3">
                        <Badge className={`${getMethodColor(request.method)} h-5 px-2 text-[10px] font-black border uppercase tracking-widest`}>
                            {request.method}
                        </Badge>
                        <div
                            className="group relative flex items-center gap-2 cursor-pointer max-w-2xl"
                            onClick={() => copyTrigger(request.url)}
                        >
                            <h2 className="text-[12px] font-mono font-medium text-slate-300 truncate hover:text-cyan-400 transition-colors">
                                {request.url}
                            </h2>
                            <Copy size={12} className="text-slate-600 opacity-0 group-hover:opacity-100 transition-opacity" />
                        </div>
                    </div>

                    <div className="h-4 w-[1px] bg-white/10" />

                    <div className="flex items-center gap-8">
                        <div className="flex flex-col cursor-pointer group/stat" onClick={() => copyTrigger(request.status?.toString() || '')}>
                            <span className="text-[9px] text-slate-600 font-black uppercase tracking-widest mb-0.5">Status</span>
                            <div className="flex items-center gap-2">
                                <span className={`text-[11px] font-black font-mono leading-none ${getStatusColor(request.status)}`}>
                                    {request.status || 'PENDING'}
                                </span>
                                <Copy size={10} className="text-slate-700 opacity-0 group-hover/stat:opacity-100 transition-opacity" />
                            </div>
                        </div>
                        <div className="flex flex-col">
                            <span className="text-[9px] text-slate-600 font-black uppercase tracking-widest mb-0.5">Time</span>
                            <span className="text-[11px] font-mono text-slate-500 leading-none">{formatTime(request.timestamp)}</span>
                        </div>
                        <div className="flex flex-col">
                            <span className="text-[9px] text-slate-600 font-black uppercase tracking-widest mb-0.5">Agent</span>
                            <span className="text-[11px] font-mono text-slate-500 leading-none">{request.agentId || 'N/A'}</span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Main Content (Split View: Request & Response) */}
            <div className="flex-1 overflow-hidden bg-[#0B0D11]">
                <ResizablePanelGroup direction="horizontal">
                    {/* REQUEST PANEL */}
                    <ResizablePanel defaultSize={50} minSize={30} className="flex flex-col border-r border-white/5">
                        <div className="flex-1 flex flex-col overflow-hidden bg-[#0E1015]/30">
                            <Tabs defaultValue="headers" className="flex-1 flex flex-col overflow-hidden">
                                <div className="px-5 py-2.5 bg-[#0E1015] border-b border-white/5 flex items-center justify-between">
                                    <div className="flex items-center gap-3">
                                        <div className="w-1.5 h-1.5 rounded-full bg-cyan-500 shadow-[0_0_8px_rgba(6,182,212,0.5)]" />
                                        <span className="text-[10px] font-black tracking-[0.2em] text-cyan-400 uppercase">REQUEST</span>
                                    </div>
                                    <TabsList className="h-7 bg-black/40 p-0.5 gap-1 rounded-md border border-white/5">
                                        <TabsTrigger value="headers" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400 text-slate-500 uppercase transition-all">HEADERS</TabsTrigger>
                                        <TabsTrigger value="body" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400 text-slate-500 uppercase transition-all">BODY</TabsTrigger>
                                        <TabsTrigger value="raw" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400 text-slate-500 uppercase transition-all">RAW</TabsTrigger>
                                    </TabsList>
                                </div>

                                <div className="flex-1 overflow-hidden">
                                    <TabsContent value="headers" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <HeaderTable headersJson={request.requestHeaders} />
                                        </ScrollArea>
                                    </TabsContent>
                                    <TabsContent value="body" className="mt-0 h-full p-4 outline-none">
                                        <BodyViewer content={request.requestBody} />
                                    </TabsContent>
                                    <TabsContent value="raw" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <div className="p-4 bg-black/40 rounded-lg border border-white/5 font-mono text-[11px] text-slate-400 shadow-inner group/raw relative">
                                                <button
                                                    onClick={() => copyTrigger(`${request.method} ${request.url} HTTP/1.1\n${Object.entries(parseHeaders(request.requestHeaders)).map(([k, v]) => `${k}: ${v}`).join('\n')}\n\n${request.requestBody || ''}`)}
                                                    className="absolute right-3 top-3 p-1.5 bg-white/5 hover:bg-white/10 rounded-md opacity-0 group-hover/raw:opacity-100 transition-all"
                                                >
                                                    <Copy size={12} />
                                                </button>
                                                <div className="space-y-0.5">
                                                    <div className="pb-2 border-b border-white/5 mb-2 flex items-center justify-between">
                                                        <div>
                                                            <span className="text-cyan-400 font-bold">{request.method}</span> <span className="text-slate-300 truncate inline-block max-w-[200px] align-bottom">{request.url}</span> <span className="text-slate-500">HTTP/1.1</span>
                                                        </div>
                                                    </div>
                                                    {Object.entries(parseHeaders(request.requestHeaders)).map(([k, v]) => (
                                                        <div key={k} className="flex gap-2">
                                                            <span className="text-emerald-400/80 shrink-0 font-bold">{k}:</span>
                                                            <span className="text-slate-400 break-all">{v}</span>
                                                        </div>
                                                    ))}
                                                    {request.requestBody && (
                                                        <div className="mt-4 pt-4 border-t border-white/5 text-slate-300 break-all whitespace-pre-wrap font-sans text-[12px]">
                                                            {request.requestBody}
                                                        </div>
                                                    )}
                                                </div>
                                            </div>
                                        </ScrollArea>
                                    </TabsContent>
                                </div>
                            </Tabs>
                        </div>
                    </ResizablePanel>

                    <ResizableHandle className="w-[1px] bg-white/5 hover:bg-cyan-500/30 transition-all cursor-col-resize shadow-[0_0_10px_rgba(0,0,0,0.5)]" />

                    {/* RESPONSE PANEL */}
                    <ResizablePanel defaultSize={50} minSize={30} className="flex flex-col">
                        <div className="flex-1 flex flex-col overflow-hidden bg-[#0E1015]/30">
                            <Tabs defaultValue="headers" className="flex-1 flex flex-col overflow-hidden">
                                <div className="px-5 py-2.5 bg-[#0E1015] border-b border-white/5 flex items-center justify-between">
                                    <div className="flex items-center gap-3">
                                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                                        <span className="text-[10px] font-black tracking-[0.2em] text-emerald-400 uppercase">RESPONSE</span>
                                    </div>
                                    <TabsList className="h-7 bg-black/40 p-0.5 gap-1 rounded-md border border-white/5">
                                        <TabsTrigger value="headers" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400 text-slate-500 uppercase transition-all">HEADERS</TabsTrigger>
                                        <TabsTrigger value="body" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400 text-slate-500 uppercase transition-all">BODY</TabsTrigger>
                                        <TabsTrigger value="tree" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400 text-slate-500 uppercase transition-all">TREE</TabsTrigger>
                                        <TabsTrigger value="raw" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400 text-slate-500 uppercase transition-all">RAW</TabsTrigger>
                                    </TabsList>
                                </div>

                                <div className="flex-1 overflow-hidden">
                                    <TabsContent value="headers" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <HeaderTable headersJson={request.responseHeaders} />
                                        </ScrollArea>
                                    </TabsContent>
                                    <TabsContent value="body" className="mt-0 h-full p-4 outline-none">
                                        <BodyViewer content={request.responseBody} />
                                    </TabsContent>
                                    <TabsContent value="tree" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <div className="p-4 border border-white/5 rounded-md bg-black/20 ring-1 ring-white/[0.02]">
                                                <JsonTreeView data={(() => { try { return JSON.parse(request.responseBody || '{}'); } catch { return { error: "Invalid JSON" }; } })()} />
                                            </div>
                                        </ScrollArea>
                                    </TabsContent>
                                    <TabsContent value="raw" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <div className="p-4 bg-black/40 rounded-lg border border-white/5 font-mono text-[11px] text-slate-400 shadow-inner group/raw relative">
                                                <button
                                                    onClick={() => copyTrigger(`HTTP/1.1 ${request.status}\n${Object.entries(parseHeaders(request.responseHeaders)).map(([k, v]) => `${k}: ${v}`).join('\n')}\n\n${request.responseBody || ''}`)}
                                                    className="absolute right-3 top-3 p-1.5 bg-white/5 hover:bg-white/10 rounded-md opacity-0 group-hover/raw:opacity-100 transition-all"
                                                >
                                                    <Copy size={12} />
                                                </button>
                                                <div className="space-y-0.5">
                                                    <div className="pb-2 border-b border-white/5 mb-2 flex items-center justify-between">
                                                        <div>
                                                            <span className="text-slate-500">HTTP/1.1</span> <span className={`font-bold ${getStatusColor(request.status)}`}>{request.status || 'PENDING'}</span>
                                                        </div>
                                                    </div>
                                                    {Object.entries(parseHeaders(request.responseHeaders)).map(([k, v]) => (
                                                        <div key={k} className="flex gap-2">
                                                            <span className="text-emerald-400/80 shrink-0 font-bold">{k}:</span>
                                                            <span className="text-slate-400 break-all">{v}</span>
                                                        </div>
                                                    ))}
                                                    {request.responseBody && (
                                                        <div className="mt-4 pt-4 border-t border-white/5 text-slate-300 break-all whitespace-pre-wrap font-sans text-[12px]">
                                                            {request.responseBody}
                                                        </div>
                                                    )}
                                                </div>
                                            </div>
                                        </ScrollArea>
                                    </TabsContent>
                                </div>
                            </Tabs>
                        </div>
                    </ResizablePanel>
                </ResizablePanelGroup>
            </div>
        </div>
    );
};
