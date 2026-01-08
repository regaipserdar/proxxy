import { useQuery } from '@tanstack/react-query';
import { api } from '../lib/api';
import { TrafficResponse } from '../types';

export function TrafficTable() {
    const { data: traffic, isLoading } = useQuery<TrafficResponse>({
        queryKey: ['traffic'],
        queryFn: () => api.get('/traffic').then(res => res.data),
        refetchInterval: 3000 // Refresh every 3 seconds
    });

    const getMethodColor = (method: string) => {
        switch (method.toUpperCase()) {
            case 'GET': return 'text-emerald-400';
            case 'POST': return 'text-blue-400';
            case 'PUT': return 'text-yellow-400';
            case 'DELETE': return 'text-red-400';
            case 'PATCH': return 'text-purple-400';
            default: return 'text-gray-400';
        }
    };

    const getStatusColor = (status: number | null) => {
        if (!status) return 'text-gray-400';
        if (status >= 200 && status < 300) return 'text-emerald-400';
        if (status >= 300 && status < 400) return 'text-blue-400';
        if (status >= 400 && status < 500) return 'text-yellow-400';
        if (status >= 500) return 'text-red-400';
        return 'text-gray-400';
    };

    const parseUrl = (url: string) => {
        try {
            const parsed = new URL(url);
            return {
                host: parsed.hostname,
                path: parsed.pathname + parsed.search
            };
        } catch {
            return { host: url, path: '/' };
        }
    };

    return (
        <div className="bg-[#111318] border border-white/5 rounded-xl overflow-hidden">
            <div className="px-4 py-3 border-b border-white/5 flex justify-between items-center">
                <h3 className="text-xs font-bold text-white/60">Recent Traffic</h3>
                <div className="flex items-center gap-3">
                    <span className="text-[10px] text-white/20">
                        {traffic?.total_count || 0} requests
                    </span>
                    {isLoading && (
                        <span className="text-[10px] text-[#9DCDE8] animate-pulse">Loading...</span>
                    )}
                </div>
            </div>
            <table className="w-full text-left text-[12px]">
                <thead className="text-white/30 font-mono border-b border-white/5">
                    <tr>
                        <th className="px-4 py-2 font-normal">Method</th>
                        <th className="px-4 py-2 font-normal">Host</th>
                        <th className="px-4 py-2 font-normal">Path</th>
                        <th className="px-4 py-2 font-normal">Status</th>
                        <th className="px-4 py-2 font-normal">Agent</th>
                    </tr>
                </thead>
                <tbody className="text-white/70">
                    {!traffic || traffic.transactions.length === 0 ? (
                        <tr>
                            <td colSpan={5} className="px-4 py-8 text-center text-white/20 text-sm">
                                {isLoading ? 'Loading traffic...' : 'No traffic captured yet. Configure your browser to use the proxy.'}
                            </td>
                        </tr>
                    ) : (
                        traffic.transactions.map((tx) => {
                            const { host, path } = parseUrl(tx.url);
                            return (
                                <tr key={tx.request_id} className="border-b border-white/5 hover:bg-white/5 cursor-pointer">
                                    <td className={`px-4 py-2 font-bold ${getMethodColor(tx.method)}`}>
                                        {tx.method}
                                    </td>
                                    <td className="px-4 py-2 opacity-60 font-mono text-[11px]">
                                        {host}
                                    </td>
                                    <td className="px-4 py-2 opacity-60 truncate max-w-xs" title={path}>
                                        {path}
                                    </td>
                                    <td className={`px-4 py-2 ${getStatusColor(tx.status)}`}>
                                        {tx.status || '-'}
                                    </td>
                                    <td className="px-4 py-2 opacity-40 text-[10px] font-mono">
                                        {tx.agent_id.substring(0, 8)}
                                    </td>
                                </tr>
                            );
                        })
                    )}
                </tbody>
            </table>
        </div>
    );
}
