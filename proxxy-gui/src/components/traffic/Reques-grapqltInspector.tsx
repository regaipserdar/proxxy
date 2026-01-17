import { useState } from 'react';
import { Copy, CheckCircle2, Braces, Clock, HardDrive, Link2, Zap, FileJson } from 'lucide-react';
import { TrafficRequest, getMethodColor, getStatusColor, formatTime, parseHeaders } from './types-grapql';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ScrollArea } from '@/components/ui/scroll-area';
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable';

// ============= UTILITY FUNCTIONS =============

const parseQueryParams = (url: string): Record<string, string> => {
    try {
        const urlObj = new URL(url);
        const params: Record<string, string> = {};
        urlObj.searchParams.forEach((value, key) => {
            params[key] = value;
        });
        return params;
    } catch {
        return {};
    }
};

const decodeJWT = (token: string): any => {
    try {
        const base64Url = token.split('.')[1];
        const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
        const jsonPayload = decodeURIComponent(atob(base64).split('').map(c =>
            '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2)
        ).join(''));
        return JSON.parse(jsonPayload);
    } catch {
        return null;
    }
};

const findAuthTokens = (headers: Record<string, string>, body?: string): Array<{ type: string, value: string, decoded?: any }> => {
    const tokens: Array<{ type: string, value: string, decoded?: any }> = [];

    // Authorization header
    if (headers['authorization'] || headers['Authorization']) {
        const authHeader = headers['authorization'] || headers['Authorization'];
        if (authHeader.startsWith('Bearer ')) {
            const token = authHeader.substring(7);
            tokens.push({
                type: 'Bearer Token (Header)',
                value: token,
                decoded: decodeJWT(token)
            });
        } else if (authHeader.startsWith('Basic ')) {
            tokens.push({
                type: 'Basic Auth (Header)',
                value: authHeader.substring(6)
            });
        }
    }

    // API Key in headers
    ['x-api-key', 'api-key', 'apikey'].forEach(key => {
        if (headers[key] || headers[key.toUpperCase()]) {
            tokens.push({
                type: 'API Key (Header)',
                value: headers[key] || headers[key.toUpperCase()]
            });
        }
    });

    // Session tokens
    ['x-session-id', 'session-id', 'sessionid'].forEach(key => {
        if (headers[key] || headers[key.toUpperCase()]) {
            tokens.push({
                type: 'Session ID (Header)',
                value: headers[key] || headers[key.toUpperCase()]
            });
        }
    });

    // JWT in body
    if (body) {
        try {
            const bodyObj = JSON.parse(body);
            ['token', 'access_token', 'accessToken', 'jwt'].forEach(key => {
                if (bodyObj[key] && typeof bodyObj[key] === 'string') {
                    tokens.push({
                        type: `JWT Token (Body: ${key})`,
                        value: bodyObj[key],
                        decoded: decodeJWT(bodyObj[key])
                    });
                }
            });
        } catch { }
    }

    return tokens;
};

const calculateSize = (content?: string): string => {
    if (!content) return '0 B';
    const bytes = new Blob([content]).size;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};

// ============= SUB-COMPONENTS =============

const StatsBar = ({ request, sequentialId }: { request: TrafficRequest, sequentialId?: number }) => {
    const requestSize = calculateSize(request.requestBody);
    const responseSize = calculateSize(request.responseBody);
    const responseTime = request.responseTime || 0;

    return (
        <div className="grid grid-cols-5 gap-3 px-6 py-3 bg-black/40 border-b border-white/5">
            <div className="flex flex-col gap-1">
                <div className="flex items-center gap-1.5">
                    <div className="w-1 h-1 rounded-full bg-slate-600" />
                    <span className="text-[8px] text-slate-600 font-black uppercase tracking-widest">Request ID</span>
                </div>
                <span className="text-[13px] font-mono font-bold text-cyan-400">#{sequentialId || request.requestId}</span>
            </div>

            <div className="flex flex-col gap-1">
                <div className="flex items-center gap-1.5">
                    <Clock size={9} className="text-slate-600" />
                    <span className="text-[8px] text-slate-600 font-black uppercase tracking-widest">Time</span>
                </div>
                <span className="text-[13px] font-mono font-bold text-emerald-400">
                    {responseTime > 0 ? `${responseTime}ms` : formatTime(request.timestamp)}
                </span>
            </div>

            <div className="flex flex-col gap-1">
                <div className="flex items-center gap-1.5">
                    <HardDrive size={9} className="text-slate-600" />
                    <span className="text-[8px] text-slate-600 font-black uppercase tracking-widest">Request Size</span>
                </div>
                <span className="text-[13px] font-mono font-bold text-slate-400">{requestSize}</span>
            </div>

            <div className="flex flex-col gap-1">
                <div className="flex items-center gap-1.5">
                    <HardDrive size={9} className="text-slate-600" />
                    <span className="text-[8px] text-slate-600 font-black uppercase tracking-widest">Response Size</span>
                </div>
                <span className="text-[13px] font-mono font-bold text-slate-400">{responseSize}</span>
            </div>

            <div className="flex flex-col gap-1">
                <div className="flex items-center gap-1.5">
                    <Zap size={9} className="text-slate-600" />
                    <span className="text-[8px] text-slate-600 font-black uppercase tracking-widest">Protocol</span>
                </div>
                <span className="text-[13px] font-mono font-bold text-slate-400">HTTP/1.1</span>
            </div>
        </div>
    );
};

const QueryParamsTable = ({ url }: { url: string }) => {
    const params = parseQueryParams(url);
    const [copied, setCopied] = useState<string | null>(null);

    if (Object.keys(params).length === 0) {
        return (
            <div className="text-slate-600 italic text-center py-8 border border-dashed border-white/10 rounded-md bg-black/10">
                No query parameters
            </div>
        );
    }

    const copyParam = (value: string, key: string) => {
        navigator.clipboard.writeText(value);
        setCopied(key);
        setTimeout(() => setCopied(null), 2000);
    };

    return (
        <div className="rounded-md border border-white/5 overflow-hidden bg-black/20">
            <table className="w-full text-[12px] font-mono border-collapse">
                <thead>
                    <tr className="bg-white/5 text-slate-400 text-left border-b border-white/5">
                        <th className="px-3 py-2 font-semibold w-1/3 text-[10px] uppercase tracking-wider">Parameter</th>
                        <th className="px-3 py-2 font-semibold text-[10px] uppercase tracking-wider">Value</th>
                    </tr>
                </thead>
                <tbody>
                    {Object.entries(params).map(([key, val]) => (
                        <tr key={key} className="border-b border-white/[0.02] hover:bg-white/[0.02] group/row">
                            <td className="px-3 py-1.5 text-amber-400 select-all font-bold align-top border-r border-white/5">{key}</td>
                            <td className="px-3 py-1.5 text-slate-300 break-all select-all group/cell relative">
                                {val}
                                <button
                                    onClick={() => copyParam(val, key)}
                                    className="absolute right-2 top-1.5 p-1 bg-slate-800/80 hover:bg-slate-700 rounded opacity-0 group-hover/cell:opacity-100 transition-opacity"
                                >
                                    {copied === key ? <CheckCircle2 size={10} className="text-emerald-400" /> : <Copy size={10} />}
                                </button>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
};

const AuthTokensViewer = ({ request }: { request: TrafficRequest }) => {
    const reqHeaders = parseHeaders(request.requestHeaders);
    const resHeaders = parseHeaders(request.responseHeaders);
    const reqTokens = findAuthTokens(reqHeaders, request.requestBody);
    const resTokens = findAuthTokens(resHeaders, request.responseBody);

    const [copied, setCopied] = useState<string | null>(null);

    const copyToken = (value: string, id: string) => {
        navigator.clipboard.writeText(value);
        setCopied(id);
        setTimeout(() => setCopied(null), 2000);
    };

    if (reqTokens.length === 0 && resTokens.length === 0) {
        return (
            <div className="text-slate-600 italic text-center py-8 border border-dashed border-white/10 rounded-md bg-black/10">
                No authentication tokens found
            </div>
        );
    }

    return (
        <div className="space-y-4">
            {reqTokens.length > 0 && (
                <div>
                    <h3 className="text-[10px] font-black text-cyan-400 uppercase tracking-widest mb-2 flex items-center gap-2">
                        <Link2 size={12} />
                        Request Tokens
                    </h3>
                    <div className="space-y-2">
                        {reqTokens.map((token, idx) => (
                            <div key={idx} className="p-3 bg-black/40 border border-white/5 rounded-md group/token relative">
                                <button
                                    onClick={() => copyToken(token.value, `req-${idx}`)}
                                    className="absolute right-2 top-2 p-1.5 bg-slate-800/80 hover:bg-slate-700 rounded opacity-0 group-hover/token:opacity-100 transition-opacity"
                                >
                                    {copied === `req-${idx}` ? <CheckCircle2 size={12} className="text-emerald-400" /> : <Copy size={12} />}
                                </button>
                                <div className="text-[9px] text-slate-500 uppercase tracking-wider font-bold mb-1">{token.type}</div>
                                <div className="text-[11px] font-mono text-slate-300 break-all mb-2">{token.value}</div>
                                {token.decoded && (
                                    <div className="mt-2 pt-2 border-t border-white/5">
                                        <div className="text-[9px] text-emerald-500 uppercase tracking-wider font-bold mb-1 flex items-center gap-1">
                                            <FileJson size={10} />
                                            Decoded JWT
                                        </div>
                                        <pre className="text-[10px] font-mono text-slate-400 bg-black/60 p-2 rounded overflow-x-auto">
                                            {JSON.stringify(token.decoded, null, 2)}
                                        </pre>
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {resTokens.length > 0 && (
                <div>
                    <h3 className="text-[10px] font-black text-emerald-400 uppercase tracking-widest mb-2 flex items-center gap-2">
                        <Link2 size={12} />
                        Response Tokens
                    </h3>
                    <div className="space-y-2">
                        {resTokens.map((token, idx) => (
                            <div key={idx} className="p-3 bg-black/40 border border-white/5 rounded-md group/token relative">
                                <button
                                    onClick={() => copyToken(token.value, `res-${idx}`)}
                                    className="absolute right-2 top-2 p-1.5 bg-slate-800/80 hover:bg-slate-700 rounded opacity-0 group-hover/token:opacity-100 transition-opacity"
                                >
                                    {copied === `res-${idx}` ? <CheckCircle2 size={12} className="text-emerald-400" /> : <Copy size={12} />}
                                </button>
                                <div className="text-[9px] text-slate-500 uppercase tracking-wider font-bold mb-1">{token.type}</div>
                                <div className="text-[11px] font-mono text-slate-300 break-all mb-2">{token.value}</div>
                                {token.decoded && (
                                    <div className="mt-2 pt-2 border-t border-white/5">
                                        <div className="text-[9px] text-emerald-500 uppercase tracking-wider font-bold mb-1 flex items-center gap-1">
                                            <FileJson size={10} />
                                            Decoded JWT
                                        </div>
                                        <pre className="text-[10px] font-mono text-slate-400 bg-black/60 p-2 rounded overflow-x-auto">
                                            {JSON.stringify(token.decoded, null, 2)}
                                        </pre>
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
};

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
    sequentialId?: number;
};

const RawHttpViewer = ({ type, method, url, status, headers, body }: { type: 'request' | 'response', method?: string, url?: string, status?: number, headers?: string, body?: string }) => {
    const parsedHeaders = parseHeaders(headers);
    const [copied, setCopied] = useState(false);

    const handleCopy = () => {
        let text = "";
        if (type === 'request') {
            text = `${method} ${url} HTTP/1.1\n`;
        } else {
            text = `HTTP/1.1 ${status}\n`;
        }
        Object.entries(parsedHeaders).forEach(([k, v]) => {
            text += `${k}: ${v}\n`;
        });
        text += `\n${body || ''}`;

        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="p-4 bg-black/40 rounded-lg border border-white/5 font-mono text-[11px] text-slate-400 shadow-inner group/raw relative min-h-full">
            <button
                onClick={handleCopy}
                className="absolute right-3 top-3 p-1.5 bg-white/5 hover:bg-white/10 rounded-md opacity-0 group-hover/raw:opacity-100 transition-all z-10"
            >
                {copied ? <CheckCircle2 size={12} className="text-emerald-400" /> : <Copy size={12} />}
            </button>
            <div className="space-y-0.5">
                <div className="pb-2 border-b border-white/5 mb-2 flex items-center justify-between">
                    <div className="break-all whitespace-pre-wrap">
                        {type === 'request' ? (
                            <>
                                <span className="text-cyan-400 font-bold">{method}</span> <span className="text-slate-300">{url}</span> <span className="text-slate-500">HTTP/1.1</span>
                            </>
                        ) : (
                            <>
                                <span className="text-slate-500">HTTP/1.1</span> <span className={`font-bold ${getStatusColor(status)}`}>{status || 'PENDING'}</span>
                            </>
                        )}
                    </div>
                </div>
                {Object.entries(parsedHeaders).map(([k, v]) => (
                    <div key={k} className="flex gap-2">
                        <span className="text-emerald-400/80 shrink-0 font-bold">{k}:</span>
                        <span className="text-slate-400 break-all">{v}</span>
                    </div>
                ))}
                {body && (
                    <div className="mt-4 pt-4 border-t border-white/5 text-slate-300 break-all whitespace-pre-wrap font-sans text-[12px]">
                        {body}
                    </div>
                )}
            </div>
        </div>
    );
};

// ============= MAIN INSPECTOR =============

interface RequestInspectorProps {
    request: TrafficRequest | undefined;
    sequentialId?: number;
}

export const RequestInspector = ({ request, sequentialId }: RequestInspectorProps) => {
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

            {/* Stats Bar */}
            <StatsBar request={request} sequentialId={sequentialId} />

            {/* Main Content (Split View: Request & Response) */}
            <div className="flex-1 overflow-hidden bg-[#0B0D11]">
                <ResizablePanelGroup direction="horizontal">
                    {/* REQUEST PANEL */}
                    <ResizablePanel defaultSize={50} minSize={30} className="flex flex-col border-r border-white/5">
                        <div className="flex-1 flex flex-col overflow-hidden bg-[#0E1015]/30">
                            <Tabs defaultValue="raw" className="flex-1 flex flex-col overflow-hidden">
                                <div className="px-5 py-2.5 bg-[#0E1015] border-b border-white/5 flex items-center justify-between">
                                    <div className="flex items-center gap-3">
                                        <div className="w-1.5 h-1.5 rounded-full bg-cyan-500 shadow-[0_0_8px_rgba(6,182,212,0.5)]" />
                                        <span className="text-[10px] font-black tracking-[0.2em] text-cyan-400 uppercase">REQUEST</span>
                                    </div>
                                    <TabsList className="h-7 bg-black/40 p-0.5 gap-1 rounded-md border border-white/5">
                                        <TabsTrigger value="raw" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400 text-slate-500 uppercase transition-all">RAW</TabsTrigger>
                                        <TabsTrigger value="headers" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400 text-slate-500 uppercase transition-all">HEADERS</TabsTrigger>
                                        <TabsTrigger value="params" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400 text-slate-500 uppercase transition-all">PARAMS</TabsTrigger>
                                        <TabsTrigger value="body" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400 text-slate-500 uppercase transition-all">BODY</TabsTrigger>
                                        <TabsTrigger value="auth" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400 text-slate-500 uppercase transition-all">AUTH</TabsTrigger>
                                    </TabsList>
                                </div>

                                <div className="flex-1 overflow-hidden">
                                    <TabsContent value="raw" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <RawHttpViewer type="request" method={request.method} url={request.url} headers={request.requestHeaders} body={request.requestBody} />
                                        </ScrollArea>
                                    </TabsContent>
                                    <TabsContent value="headers" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <HeaderTable headersJson={request.requestHeaders} />
                                        </ScrollArea>
                                    </TabsContent>
                                    <TabsContent value="params" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <QueryParamsTable url={request.url} />
                                        </ScrollArea>
                                    </TabsContent>
                                    <TabsContent value="body" className="mt-0 h-full p-4 outline-none">
                                        <BodyViewer content={request.requestBody} />
                                    </TabsContent>
                                    <TabsContent value="auth" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <AuthTokensViewer request={request} />
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
                            <Tabs defaultValue="raw" className="flex-1 flex flex-col overflow-hidden">
                                <div className="px-5 py-2.5 bg-[#0E1015] border-b border-white/5 flex items-center justify-between">
                                    <div className="flex items-center gap-3">
                                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                                        <span className="text-[10px] font-black tracking-[0.2em] text-emerald-400 uppercase">RESPONSE</span>
                                    </div>
                                    <TabsList className="h-7 bg-black/40 p-0.5 gap-1 rounded-md border border-white/5">
                                        <TabsTrigger value="raw" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400 text-slate-500 uppercase transition-all">RAW</TabsTrigger>
                                        <TabsTrigger value="headers" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400 text-slate-500 uppercase transition-all">HEADERS</TabsTrigger>
                                        <TabsTrigger value="body" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400 text-slate-500 uppercase transition-all">BODY</TabsTrigger>
                                        <TabsTrigger value="tree" className="h-full px-3 rounded-sm text-[8px] font-bold tracking-wider data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400 text-slate-500 uppercase transition-all">TREE</TabsTrigger>
                                    </TabsList>
                                </div>

                                <div className="flex-1 overflow-hidden">
                                    <TabsContent value="raw" className="mt-0 h-full p-4 outline-none">
                                        <ScrollArea className="h-full">
                                            <RawHttpViewer type="response" status={request.status} headers={request.responseHeaders} body={request.responseBody} />
                                        </ScrollArea>
                                    </TabsContent>
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
                                </div>
                            </Tabs>
                        </div>
                    </ResizablePanel>
                </ResizablePanelGroup>
            </div>
        </div>
    );
};