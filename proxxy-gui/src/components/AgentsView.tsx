import { useState } from 'react';
import { useQuery } from '@apollo/client';
import { Search, Server } from 'lucide-react';
import { GET_AGENTS } from '../graphql/operations';
import { Agent } from '../types/graphql';
import { AgentCard } from './AgentCard';

export const AgentsView = () => {
    const [searchTerm, setSearchTerm] = useState('');
    const [filterStatus, setFilterStatus] = useState<'All' | 'Online' | 'Offline'>('All');

    const { data, loading, error } = useQuery<{ agents: Agent[] }>(GET_AGENTS, {
        pollInterval: 5000,
        fetchPolicy: 'cache-and-network',
    });

    const filteredAgents = data?.agents.filter((agent: Agent) => {
        const matchesSearch = (agent.name || '').toLowerCase().includes(searchTerm.toLowerCase()) ||
            agent.id.toLowerCase().includes(searchTerm.toLowerCase()) ||
            agent.hostname.toLowerCase().includes(searchTerm.toLowerCase());
        const matchesStatus = filterStatus === 'All' || agent.status === filterStatus;
        return matchesSearch && matchesStatus;
    }) || [];

    const onlineCount = data?.agents.filter((a: Agent) => a.status === 'Online').length || 0;
    const offlineCount = data?.agents.filter((a: Agent) => a.status === 'Offline').length || 0;

    if (error) {
        return (
            <div className="p-10 flex flex-col items-center justify-center text-red-400">
                <Server size={48} className="mb-4 opacity-50" />
                <h3 className="text-lg font-bold mb-2">Error Loading Agents</h3>
                <p className="text-sm opacity-80">{error.message}</p>
            </div>
        );
    }

    return (
        <div className="p-8 h-full flex flex-col gap-6 w-full max-w-[1600px] mx-auto animate-in fade-in duration-500">
            <div className="flex justify-between items-center">
                <div>
                    <h1 className="text-2xl font-bold text-white mb-2">Agents</h1>
                    <p className="text-white/40">Manage and monitor connected proxy agents</p>
                </div>
                {/* Status Summary Pills */}
                <div className="flex gap-3">
                    <div className="flex items-center gap-2 px-3 py-1.5 bg-[#111318] rounded-lg border border-white/10 shadow-sm">
                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)] animate-pulse" />
                        <span className="text-xs font-bold text-white/90">{onlineCount} Online</span>
                    </div>
                    <div className="flex items-center gap-2 px-3 py-1.5 bg-[#111318] rounded-lg border border-white/10 shadow-sm">
                        <div className="w-1.5 h-1.5 rounded-full bg-red-400" />
                        <span className="text-xs font-bold text-white/90">{offlineCount} Offline</span>
                    </div>
                </div>
            </div>

            <div className="flex flex-col md:flex-row gap-4 mb-2">
                {/* Search Bar */}
                <div className="relative flex-1 group">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-[#9DCDE8] transition-colors" size={16} />
                    <input
                        type="text"
                        placeholder="Search agents by ID, name or hostname..."
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                        className="w-full bg-[#111318] border border-white/10 rounded-xl pl-10 pr-4 py-3 text-sm focus:outline-none focus:border-[#9DCDE8]/30 focus:ring-1 focus:ring-[#9DCDE8]/20 transition-all text-white placeholder:text-white/20 font-mono"
                    />
                </div>

                {/* Filter Toggles */}
                <div className="flex bg-[#111318] border border-white/10 rounded-xl p-1 shrink-0">
                    {(['All', 'Online', 'Offline'] as const).map(status => (
                        <button
                            key={status}
                            onClick={() => setFilterStatus(status)}
                            className={`px-4 py-2 rounded-lg text-xs font-bold transition-all ${filterStatus === status
                                ? 'bg-white/10 text-white shadow-sm ring-1 ring-white/5'
                                : 'text-white/40 hover:text-white/80 hover:bg-white/5'
                                }`}
                        >
                            {status}
                        </button>
                    ))}
                </div>
            </div>

            {loading && !data && (
                <div className="flex-1 flex items-center justify-center">
                    <div className="flex flex-col items-center gap-4">
                        <div className="w-8 h-8 border-2 border-[#9DCDE8] border-t-transparent rounded-full animate-spin" />
                        <span className="text-xs text-white/30 font-mono animate-pulse">SYNCING AGENTS...</span>
                    </div>
                </div>
            )}

            {!loading && (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                    {filteredAgents.map((agent: Agent) => (
                        <AgentCard key={agent.id} agent={agent} />
                    ))}
                    {filteredAgents.length === 0 && (
                        <div className="col-span-full flex flex-col items-center justify-center py-24 text-white/20 border-2 border-dashed border-white/5 rounded-2xl bg-white/[0.02]">
                            <Server size={48} className="mb-4 opacity-50" />
                            <p className="font-mono text-sm">No agents found matching your criteria</p>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};
