export interface TrafficRequest {
    requestId: string;
    method: string;
    url: string;
    status?: number;
    timestamp: string | number;
    agentId?: string;
    requestHeaders?: string;
    responseHeaders?: string;
    requestBody?: string;
    responseBody?: string;
}

export const getMethodColor = (method: string) => {
    switch (method?.toUpperCase()) {
        case 'GET': return 'bg-sky-500/10 text-sky-400 border-sky-500/20';
        case 'POST': return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
        case 'PUT': return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
        case 'DELETE': return 'bg-rose-500/10 text-rose-400 border-rose-500/20';
        default: return 'bg-slate-500/10 text-slate-400 border-slate-500/20';
    }
};

export const getStatusColor = (status: number | undefined) => {
    if (!status) return 'text-slate-500';
    if (status < 300) return 'text-emerald-400';
    if (status < 400) return 'text-sky-400';
    if (status < 500) return 'text-amber-400';
    return 'text-rose-400';
};

export const formatTime = (ts: string | number) => {
    try {
        const date = new Date(typeof ts === 'number' ? ts * 1000 : ts);
        return date.toLocaleTimeString('tr-TR', { hour12: false }) + '.' + date.getMilliseconds().toString().padStart(3, '0');
    } catch { return '--:--:--'; }
};

export const parseHeaders = (headersJson: string | undefined): Record<string, string> => {
    try {
        return JSON.parse(headersJson || '{}');
    } catch {
        return {};
    }
};
