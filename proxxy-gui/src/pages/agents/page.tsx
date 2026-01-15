import { useState } from 'react';
import { useQuery } from '@apollo/client';
import { Search, Server, Plus, LayoutGrid, List, SlidersHorizontal, ArrowUpDown } from 'lucide-react';
import { GET_AGENTS } from '@/graphql/operations';
import { Agent } from '@/types/graphql';
import { AgentCard } from '@/components/agents/AgentCard';
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";

export const AgentsView = () => {
    const [searchTerm, setSearchTerm] = useState('');
    const [filterStatus, setFilterStatus] = useState<string>('all');
    const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');

    const { data, loading, error } = useQuery<{ agents: Agent[] }>(GET_AGENTS, {
        pollInterval: 5000,
        fetchPolicy: 'cache-and-network',
    });

    const filteredAgents = data?.agents.filter((agent: Agent) => {
        const name = agent.name || '';
        const id = agent.id || '';
        const host = agent.hostname || '';
        const search = searchTerm.toLowerCase();

        const matchesSearch = name.toLowerCase().includes(search) ||
            id.toLowerCase().includes(search) ||
            host.toLowerCase().includes(search);

        const matchesStatus = filterStatus === 'all' ||
            (agent.status?.toLowerCase() === filterStatus.toLowerCase());

        return matchesSearch && matchesStatus;
    }) || [];

    const stats = {
        total: data?.agents.length || 0,
        online: data?.agents.filter((a: Agent) => a.status?.toLowerCase() === 'online').length || 0,
        offline: (data?.agents.length || 0) - (data?.agents.filter((a: Agent) => a.status?.toLowerCase() === 'online').length || 0),
    };

    if (error) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center p-20 animate-in fade-in zoom-in duration-500">
                <div className="p-6 rounded-3xl bg-red-500/10 border border-red-500/20 mb-6 group">
                    <Server size={48} className="text-red-400 group-hover:shake transition-transform" />
                </div>
                <h3 className="text-2xl font-black text-white uppercase tracking-tighter mb-2">Sync Interrupted</h3>
                <p className="text-slate-500 font-medium text-center max-w-md">{error.message}</p>
                <Button variant="outline" className="mt-8 border-white/10 hover:bg-white/5" onClick={() => window.location.reload()}>
                    Re-establish Connection
                </Button>
            </div>
        );
    }

    return (
        <div className="p-8 h-full flex flex-col gap-8 w-full max-w-[1600px] mx-auto animate-in fade-in slide-in-from-bottom-4 duration-700">
            {/* Header Section */}
            <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-6 pb-2">
                <div className="space-y-1.5">
                    <div className="flex items-center gap-2">
                        <Badge className="bg-indigo-500/10 text-indigo-400 border-indigo-500/20 px-2 py-0 text-[10px] uppercase font-black tracking-widest">Network Status</Badge>
                        <div className="w-1 h-1 rounded-full bg-slate-700" />
                        <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{stats.online} Active Nodes</span>
                    </div>
                    <h1 className="text-4xl font-black text-white uppercase tracking-tighter flex items-center gap-4">
                        Agents
                        <span className="text-slate-800 tabular-nums">/{stats.total}</span>
                    </h1>
                    <p className="text-xs font-medium text-slate-500 uppercase tracking-widest opacity-70">Node management and telemetry orchestration</p>
                </div>

                <div className="flex items-center gap-3">
                    <Tabs value={viewMode} onValueChange={(v) => setViewMode(v as any)} className="bg-[#111318] border border-white/5 p-1 rounded-xl hidden sm:flex">
                        <TabsList className="bg-transparent gap-1">
                            <TabsTrigger value="grid" className="data-[state=active]:bg-white/5 data-[state=active]:text-white rounded-lg h-8 w-10 p-0 transition-all">
                                <LayoutGrid size={15} />
                            </TabsTrigger>
                            <TabsTrigger value="list" className="data-[state=active]:bg-white/5 data-[state=active]:text-white rounded-lg h-8 w-10 p-0 transition-all">
                                <List size={15} />
                            </TabsTrigger>
                        </TabsList>
                    </Tabs>

                    <Dialog>
                        <DialogTrigger asChild>
                            <Button className="bg-indigo-600 hover:bg-indigo-500 text-white font-black uppercase tracking-widest h-11 px-6 rounded-xl shadow-lg shadow-indigo-500/20 gap-2 overflow-hidden group">
                                <Plus size={18} className="transition-transform group-hover:rotate-90" />
                                <span className="hidden sm:inline">Add Node</span>
                            </Button>
                        </DialogTrigger>
                        <DialogContent className="bg-[#0B0D11] border-white/10 text-white sm:max-w-md">
                            <DialogHeader>
                                <DialogTitle className="text-xl font-black uppercase tracking-tighter">Add New Agent</DialogTitle>
                            </DialogHeader>
                            <div className="py-6 space-y-4">
                                <p className="text-sm text-slate-400 leading-relaxed font-medium">Use the CLI to connect a new proxy agent. Refer to the dashboard guide for the full connection string.</p>
                                <div className="p-4 bg-black/40 border border-white/10 rounded-2xl font-mono text-xs text-indigo-400 break-all select-all">
                                    cargo run -p proxy-agent -- --name "Agent-Alpha" --orchestrator-url http://127.0.0.1:50051
                                </div>
                            </div>
                        </DialogContent>
                    </Dialog>
                </div>
            </div>

            {/* Filters & Search Section */}
            <div className="flex flex-col xl:flex-row gap-4">
                <div className="relative flex-1 group">
                    <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-600 group-focus-within:text-indigo-400 transition-colors" size={18} />
                    <Input
                        placeholder="Search nodes by ID, friendly name or hostname..."
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                        className="bg-[#111318] border-white/5 h-12 pl-12 rounded-2xl text-sm font-medium transition-all focus-visible:ring-indigo-500/20 focus-visible:border-indigo-500/40 placeholder:text-slate-600 placeholder:uppercase placeholder:tracking-tighter placeholder:font-black"
                    />
                </div>

                <div className="flex items-center gap-3 overflow-x-auto pb-2 sm:pb-0">
                    <Tabs value={filterStatus} onValueChange={setFilterStatus} className="bg-[#111318] border border-white/5 p-1 rounded-xl">
                        <TabsList className="bg-transparent gap-1">
                            <TabsTrigger value="all" className="data-[state=active]:bg-indigo-500/10 data-[state=active]:text-indigo-400 rounded-lg h-9 px-4 text-xs font-black uppercase tracking-widest transition-all">
                                All Nodes
                                <Badge variant="secondary" className="ml-2 h-4 px-1 bg-white/5 text-[9px]">{stats.total}</Badge>
                            </TabsTrigger>
                            <TabsTrigger value="online" className="data-[state=active]:bg-emerald-500/10 data-[state=active]:text-emerald-400 rounded-lg h-9 px-4 text-xs font-black uppercase tracking-widest transition-all">
                                Online
                                <Badge variant="secondary" className="ml-2 h-4 px-1 bg-white/5 text-[9px]">{stats.online}</Badge>
                            </TabsTrigger>
                            <TabsTrigger value="offline" className="data-[state=active]:bg-red-500/10 data-[state=active]:text-red-400 rounded-lg h-9 px-4 text-xs font-black uppercase tracking-widest transition-all">
                                Offline
                                <Badge variant="secondary" className="ml-2 h-4 px-1 bg-white/5 text-[9px]">{stats.offline}</Badge>
                            </TabsTrigger>
                        </TabsList>
                    </Tabs>

                    <Button variant="outline" className="h-11 border-white/5 bg-[#111318] hover:bg-white/5 rounded-xl gap-2 font-black uppercase tracking-widest text-[10px]">
                        <ArrowUpDown size={14} className="text-slate-500" />
                        Sort By
                    </Button>
                </div>
            </div>

            {/* Content Area */}
            {loading && !data ? (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                    {[1, 2, 3, 4, 5, 6, 7, 8].map(i => (
                        <Card key={i} className="bg-[#111318] border-white/5 h-48 overflow-hidden">
                            <div className="p-5 space-y-4">
                                <div className="flex justify-between">
                                    <div className="flex gap-3">
                                        <Skeleton className="h-10 w-10 rounded-xl bg-white/5" />
                                        <div className="space-y-2">
                                            <Skeleton className="h-4 w-24 bg-white/5" />
                                            <Skeleton className="h-3 w-32 bg-white/5" />
                                        </div>
                                    </div>
                                    <Skeleton className="h-6 w-16 rounded-full bg-white/5" />
                                </div>
                                <div className="grid grid-cols-2 gap-2 pt-4">
                                    <Skeleton className="h-12 rounded-xl bg-white/5" />
                                    <Skeleton className="h-12 rounded-xl bg-white/5" />
                                </div>
                            </div>
                        </Card>
                    ))}
                </div>
            ) : (
                <>
                    {filteredAgents.length > 0 ? (
                        <div className={viewMode === 'grid'
                            ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6"
                            : "flex flex-col gap-3"
                        }>
                            {filteredAgents.map((agent: Agent) => (
                                <AgentCard key={agent.id} agent={agent} />
                            ))}
                        </div>
                    ) : (
                        <div className="flex-1 flex flex-col items-center justify-center py-32 rounded-[2.5rem] bg-indigo-500/[0.01] border-2 border-dashed border-white/5 animate-in fade-in zoom-in duration-700">
                            <div className="p-8 rounded-full bg-white/[0.02] border border-white/5 mb-8">
                                <Search size={64} className="text-slate-700" />
                            </div>
                            <h3 className="text-3xl font-black text-white uppercase tracking-tighter mb-3">No Nodes Found</h3>
                            <p className="text-slate-500 font-medium text-center max-w-sm px-6 uppercase tracking-tighter text-xs">
                                Adjust your search parameters or check the status filters to locate specific nodes.
                            </p>
                            <Button variant="link" className="text-indigo-400 font-black uppercase tracking-widest mt-6" onClick={() => { setSearchTerm(''); setFilterStatus('all'); }}>
                                Clear All Search Parameters
                            </Button>
                        </div>
                    )}
                </>
            )}

            {/* Action Footer */}
            {!loading && stats.total > 0 && (
                <div className="flex justify-center pt-8">
                    <div className="flex items-center gap-6 px-8 py-3 bg-[#111318] border border-white/5 rounded-2xl shadow-2xl">
                        <div className="flex items-center gap-3">
                            <div className="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.5)]" />
                            <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest leading-none">Global Network Active</span>
                        </div>
                        <div className="hidden sm:flex items-center gap-2 border-l border-white/10 pl-6 cursor-help">
                            <SlidersHorizontal size={14} className="text-slate-600" />
                            <span className="text-[10px] font-black text-slate-500 uppercase tracking-widest leading-none">Network Config v2.4</span>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
