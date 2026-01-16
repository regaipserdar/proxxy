import { useQuery } from '@tanstack/react-query';
import { api } from '../lib/api';
import { TrafficResponse } from '@/types';
import { Clock, Globe, Terminal, ShieldCheck } from 'lucide-react';

export function TrafficTable() {
    const { data: traffic, isLoading } = useQuery<TrafficResponse>({
        queryKey: ['traffic'],
        queryFn: () => api.get('/traffic').then(res => res.data),
        // Veritabanına yük bindirmemek için polling'i 5 saniyeye çektik
        refetchInterval: 5000
    });

    const getMethodStyle = (method: string) => {
        const m = method.toUpperCase();
        if (m === 'GET') return 'text-emerald-400 bg-emerald-400/10 border-emerald-400/20';
        if (m === 'POST') return 'text-blue-400 bg-blue-400/10 border-blue-500/20';
        if (m === 'PUT' || m === 'PATCH') return 'text-orange-400 bg-orange-400/10 border-orange-400/20';
        if (m === 'DELETE') return 'text-red-400 bg-red-400/10 border-red-500/20';
        return 'text-gray-400 bg-gray-400/10 border-gray-400/20';
    };

    const getStatusStyle = (status: number | null) => {
        if (!status) return { color: 'text-gray-500', bg: 'bg-gray-500/20' };
        if (status >= 200 && status < 300) return { color: 'text-emerald-400', bg: 'bg-emerald-400' };
        if (status >= 300 && status < 400) return { color: 'text-blue-400', bg: 'bg-blue-400' };
        if (status >= 400 && status < 500) return { color: 'text-yellow-400', bg: 'bg-yellow-400' };
        return { color: 'text-red-400', bg: 'bg-red-400' };
    };

    const parseUrl = (url: string) => {
        try {
            const parsed = new URL(url);
            return {
                host: parsed.hostname,
                path: parsed.pathname + parsed.search,
                protocol: parsed.protocol.replace(':', '')
            };
        } catch {
            return { host: url, path: '/', protocol: 'http' };
        }
    };

    return (
        <div className="bg-[#0B0C10] border border-white/[0.03] rounded-2xl overflow-hidden shadow-2xl">
            {/* Header Area */}
            <div className="px-6 py-4 border-b border-white/[0.05] flex justify-between items-center bg-white/[0.01]">
                <div className="flex items-center gap-4">
                    <h3 className="text-sm font-bold text-white tracking-wide uppercase opacity-70">Transaction Logs</h3>
                    <div className="h-4 w-px bg-white/10" />
                    <span className="text-[11px] font-mono text-white/30 tracking-widest uppercase">
                        {traffic?.total_count || 0} TOTAL CAPTURED
                    </span>
                </div>
                {isLoading && (
                    <div className="flex items-center gap-2">
                        <div className="w-1 h-1 bg-indigo-400 rounded-full animate-ping" />
                        <span className="text-[10px] text-indigo-400 font-bold uppercase tracking-widest">Polling...</span>
                    </div>
                )}
            </div>

            <div className="overflow-x-auto">
                <table className="w-full text-left">
                    <thead className="bg-white/[0.02] border-b border-white/[0.05]">
                        <tr className="text-[10px] font-bold text-white/20 uppercase tracking-[0.15em]">
                            <th className="px-6 py-4 min-w-[100px]">Method</th>
                            <th className="px-6 py-4">Target Endpoint</th>
                            <th className="px-6 py-4 text-center">Status</th>
                            <th className="px-6 py-4">Source Agent</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-white/[0.03]">
                        {!traffic || traffic.transactions.length === 0 ? (
                            <tr>
                                <td colSpan={4} className="px-6 py-20 text-center">
                                    <div className="flex flex-col items-center gap-3 opacity-20">
                                        <Terminal size={32} />
                                        <p className="text-sm">No traffic intercepted yet</p>
                                    </div>
                                </td>
                            </tr>
                        ) : (
                            traffic.transactions.map((tx: any) => {
                                const { host, path, protocol } = parseUrl(tx.url);
                                const statusStyle = getStatusStyle(tx.status);
                                return (
                                    <tr key={tx.request_id} className="group hover:bg-white/[0.02] transition-colors cursor-pointer border-l-2 border-transparent hover:border-indigo-500/50">
                                        <td className="px-6 py-3.5">
                                            <div className={`inline-flex px-2 py-0.5 rounded text-[10px] font-black border uppercase tracking-wider ${getMethodStyle(tx.method)}`}>
                                                {tx.method}
                                            </div>
                                        </td>
                                        <td className="px-6 py-3.5 max-w-md">
                                            <div className="flex flex-col">
                                                <div className="flex items-center gap-1.5 opacity-40 text-[10px] font-mono mb-0.5">
                                                    <Globe size={10} />
                                                    {protocol}://{host}
                                                </div>
                                                <div className="text-xs text-white/80 font-mono truncate group-hover:text-white transition-colors" title={path}>
                                                    {path}
                                                </div>
                                            </div>
                                        </td>
                                        <td className="px-6 py-3.5">
                                            <div className="flex flex-col items-center justify-center gap-1.5">
                                                <div className={`flex items-center gap-2 px-2 py-0.5 rounded-full bg-white/5 border border-white/5 ${statusStyle.color}`}>
                                                    <div className={`w-1 h-1 rounded-full ${statusStyle.bg}`} />
                                                    <span className="text-[11px] font-bold font-mono">{tx.status || '---'}</span>
                                                </div>
                                            </div>
                                        </td>
                                        <td className="px-6 py-3.5">
                                            <div className="flex items-center gap-2 opacity-50">
                                                <ShieldCheck size={12} className="text-indigo-400" />
                                                <span className="text-[11px] font-mono tracking-tighter uppercase">
                                                    Agent-{tx.agent_id.substring(0, 6)}
                                                </span>
                                            </div>
                                        </td>
                                    </tr>
                                );
                            })
                        )}
                    </tbody>
                </table>
            </div>

            {/* Pagination / Data Info Footer */}
            <div className="px-6 py-3 border-t border-white/[0.05] bg-white/[0.01] flex justify-between items-center">
                <div className="text-[10px] text-white/20 font-bold uppercase tracking-wider">
                    Auto-Refresh Active
                </div>
                <div className="flex items-center gap-4 text-[10px] font-mono text-white/40">
                    <span className="flex items-center gap-1"><Clock size={10} /> Latency: &lt;5ms</span>
                </div>
            </div>
        </div>
    );
}
