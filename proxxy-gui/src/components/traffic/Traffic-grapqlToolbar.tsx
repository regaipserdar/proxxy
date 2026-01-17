import {
    Search, Layout, SlidersHorizontal,
    CheckCircle2, Terminal
} from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";
import { Separator } from '@/components/ui/separator';

interface TrafficToolbarProps {
    filterQuery: string;
    setFilterQuery: (query: string) => void;
    activeMethodFilter: string | null;
    setActiveMethodFilter: (method: string | null) => void;
    totalItems: number;
    hostCount: number;
    hideConnect: boolean;
    setHideConnect: (hide: boolean) => void;
}

export const TrafficToolbar = ({
    filterQuery,
    setFilterQuery,
    activeMethodFilter,
    setActiveMethodFilter,
    totalItems,
    hostCount,
    hideConnect,
    setHideConnect
}: TrafficToolbarProps) => {
    return (
        <header className="h-14 border-b border-white/5 flex items-center px-4 justify-between shrink-0 bg-[#0E1015]/95 backdrop-blur-md z-30 shadow-2xl">
            {/* LOGO & METHOD FILTERS */}
            <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                    <div className="p-1.5 bg-cyan-500/10 rounded-lg border border-cyan-500/20 shadow-[0_0_15px_rgba(6,182,212,0.15)]">
                        <Layout className="w-4 h-4 text-cyan-400" />
                    </div>
                    <h1 className="text-[11px] font-black uppercase tracking-[0.3em] text-cyan-50 hidden sm:block bg-clip-text text-transparent bg-gradient-to-r from-white to-slate-400">
                        Traffic Tree
                    </h1>
                    <p>v 1.0.0 </p>
                </div>
                <div className="h-6 w-[1px] bg-white/10 mx-2" />

                <div className="flex gap-1.5">
                    {['GET', 'POST', 'PUT', 'DELETE'].map(m => (
                        <button
                            key={m}
                            onClick={() => setActiveMethodFilter(activeMethodFilter === m ? null : m)}
                            className={`text-[9px] font-black px-2.5 py-1 rounded border transition-all active:scale-95 ${activeMethodFilter === m
                                ? 'bg-cyan-500/20 border-cyan-500/50 text-cyan-400 shadow-[0_0_10px_rgba(6,182,212,0.1)]'
                                : 'bg-white/5 border-white/5 text-slate-500 hover:border-white/20 hover:text-slate-300'
                                }`}
                        >
                            {m}
                        </button>
                    ))}
                </div>
            </div>

            {/* SEARCH & FILTERS */}
            <div className="flex items-center gap-3">
                <div className="relative group">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-500 group-focus-within:text-cyan-400 transition-colors" />
                    <Input
                        placeholder="Search traffic (s:200)..."
                        className="h-9 w-64 md:w-80 lg:w-96 pl-10 pr-10 text-[11px] bg-black/40 border-white/10 hover:border-white/20 focus-visible:ring-cyan-500/30 font-mono placeholder:text-slate-600 transition-all focus:w-[450px]"
                        value={filterQuery}
                        onChange={(e) => setFilterQuery(e.target.value)}
                    />

                    {/* Filter Dialog Trigger */}
                    <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
                        <Dialog>
                            <DialogTrigger asChild>
                                <button
                                    className={`p-1.5 rounded-md transition-all flex items-center gap-2 border ${hideConnect
                                        ? 'bg-amber-500/10 border-amber-500/30 text-amber-500 shadow-[0_0_10px_rgba(245,158,11,0.1)]'
                                        : 'hover:bg-white/10 text-slate-500 hover:text-cyan-400 border-transparent'
                                        }`}
                                    title="Advanced Filters"
                                >
                                    <SlidersHorizontal size={14} />
                                    {hideConnect && <div className="w-1.5 h-1.5 rounded-full bg-amber-500 animate-pulse" />}
                                </button>
                            </DialogTrigger>
                            <DialogContent className="bg-[#0A0B0F] border-white/[0.08] text-slate-200 max-w-2xl w-full p-0 overflow-hidden shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
                                <DialogHeader className="px-5 py-4 bg-gradient-to-b from-slate-900/50 to-transparent border-b border-white/[0.06]">
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-3">
                                            <div className="p-1.5 bg-gradient-to-br from-cyan-500/20 to-blue-500/20 rounded-lg border border-cyan-500/30 shadow-lg">
                                                <SlidersHorizontal className="w-4 h-4 text-cyan-400" />
                                            </div>
                                            <DialogTitle className="text-xs font-semibold text-slate-200">
                                                Advanced Traffic Filters
                                            </DialogTitle>
                                        </div>
                                        <Badge variant="outline" className="text-[9px] border-cyan-500/20 text-cyan-400/80 bg-cyan-500/5 px-2 py-0.5 h-5 font-medium">
                                            Pro
                                        </Badge>
                                    </div>
                                </DialogHeader>

                                <div className="p-5 space-y-5 max-h-[70vh] overflow-y-auto">
                                    {/* Filter Presets - Burp Suite Style */}
                                    <div className="space-y-2.5">
                                        <div className="flex items-center justify-between mb-3">
                                            <h4 className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider">
                                                Filter Presets
                                            </h4>
                                            <button
                                                className="text-[9px] text-cyan-400 hover:text-cyan-300 font-medium transition-colors"
                                                onClick={() => setFilterQuery('')}
                                            >
                                                Reset All
                                            </button>
                                        </div>

                                        <div className="grid grid-cols-2 gap-2">
                                            {[
                                                { label: 'Show only errors', value: 's:4xx,5xx', active: false },
                                                { label: 'Hide images', value: '!image', active: false },
                                                { label: 'API calls only', value: 'h:api', active: false },
                                                { label: 'Slow requests', value: 't:>1000', active: false },
                                            ].map((preset, idx) => (
                                                <button
                                                    key={idx}
                                                    className="flex items-center gap-2 px-3 py-2 bg-slate-900/40 hover:bg-slate-800/60 border border-white/[0.06] hover:border-white/10 rounded-lg transition-all text-left group"
                                                    onClick={() => setFilterQuery(preset.value)}
                                                >
                                                    <div className={`w-1.5 h-1.5 rounded-full ${preset.active ? 'bg-cyan-400' : 'bg-slate-600'} group-hover:bg-cyan-400 transition-colors`} />
                                                    <span className="text-[10px] text-slate-300 group-hover:text-slate-100 font-medium transition-colors">
                                                        {preset.label}
                                                    </span>
                                                </button>
                                            ))}
                                        </div>
                                    </div>

                                    <Separator className="bg-white/[0.06]" />

                                    {/* Visibility Controls - Chrome DevTools Style */}
                                    <div className="space-y-2.5">
                                        <h4 className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-3">
                                            Display Options
                                        </h4>

                                        <div className="space-y-2">
                                            <div
                                                onClick={() => setHideConnect(!hideConnect)}
                                                className="flex items-center justify-between p-3 bg-slate-900/40 hover:bg-slate-800/60 border border-white/[0.06] hover:border-white/10 rounded-lg transition-all cursor-pointer group"
                                            >
                                                <div className="flex items-center gap-3">
                                                    <div className={`w-4 h-4 rounded border-2 flex items-center justify-center transition-all ${hideConnect
                                                        ? 'bg-cyan-500 border-cyan-500'
                                                        : 'border-slate-600 group-hover:border-slate-500'
                                                        }`}>
                                                        {hideConnect && <CheckCircle2 className="w-3 h-3 text-white" strokeWidth={3} />}
                                                    </div>
                                                    <div>
                                                        <p className="text-[11px] font-medium text-slate-200 group-hover:text-slate-100 transition-colors">
                                                            Hide CONNECT tunnels
                                                        </p>
                                                        <p className="text-[10px] text-slate-500 mt-0.5">
                                                            Filter out SSL/TLS handshakes and tunnel establishment
                                                        </p>
                                                    </div>
                                                </div>
                                                <Badge variant="outline" className="text-[8px] border-slate-700 text-slate-500 bg-slate-900/50 px-1.5 py-0 h-4 font-mono">
                                                    CONNECT
                                                </Badge>
                                            </div>

                                            {/* Additional Options */}
                                            {[
                                                { label: 'Hide OPTIONS requests', desc: 'Filter CORS preflight checks', tag: 'OPTIONS' },
                                                { label: 'Show only modified', desc: 'Display intercepted & modified traffic', tag: 'MOD' }
                                            ].map((option, idx) => (
                                                <div
                                                    key={idx}
                                                    className="flex items-center justify-between p-3 bg-slate-900/40 hover:bg-slate-800/60 border border-white/[0.06] hover:border-white/10 rounded-lg transition-all cursor-pointer group opacity-60"
                                                >
                                                    <div className="flex items-center gap-3">
                                                        <div className="w-4 h-4 rounded border-2 border-slate-600 group-hover:border-slate-500 transition-all" />
                                                        <div>
                                                            <p className="text-[11px] font-medium text-slate-200 group-hover:text-slate-100 transition-colors">
                                                                {option.label}
                                                            </p>
                                                            <p className="text-[10px] text-slate-500 mt-0.5">
                                                                {option.desc}
                                                            </p>
                                                        </div>
                                                    </div>
                                                    <Badge variant="outline" className="text-[8px] border-slate-700 text-slate-500 bg-slate-900/50 px-1.5 py-0 h-4 font-mono">
                                                        {option.tag}
                                                    </Badge>
                                                </div>
                                            ))}
                                        </div>
                                    </div>

                                    <Separator className="bg-white/[0.06]" />

                                    {/* Search Syntax Reference - Postman Style */}
                                    <div className="space-y-2.5">
                                        <h4 className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-3">
                                            Search Operators
                                        </h4>

                                        <div className="bg-slate-900/60 border border-white/[0.06] rounded-lg overflow-hidden">
                                            <div className="divide-y divide-white/[0.06]">
                                                {[
                                                    { operator: 's:', example: 's:200', desc: 'Status code (200, 404, 5xx)', color: 'text-emerald-400' },
                                                    { operator: 'm:', example: 'm:POST', desc: 'HTTP method (GET, POST, PUT)', color: 'text-blue-400' },
                                                    { operator: 'h:', example: 'h:api.example', desc: 'Host or domain filter', color: 'text-amber-400' },
                                                    { operator: 't:', example: 't:>500', desc: 'Response time in ms', color: 'text-purple-400' },
                                                    { operator: '!', example: '!image', desc: 'Exclude content type', color: 'text-red-400' }
                                                ].map((syntax, idx) => (
                                                    <div
                                                        key={idx}
                                                        className="flex items-center justify-between p-3 hover:bg-slate-800/40 transition-colors group cursor-pointer"
                                                        onClick={() => setFilterQuery(syntax.example)}
                                                    >
                                                        <div className="flex items-center gap-3 flex-1">
                                                            <code className={`text-xs font-mono font-semibold ${syntax.color} bg-black/30 px-2 py-1 rounded border border-white/[0.08] min-w-[60px] text-center`}>
                                                                {syntax.operator}
                                                            </code>
                                                            <div className="flex-1">
                                                                <p className="text-[10px] text-slate-300 font-medium">{syntax.desc}</p>
                                                            </div>
                                                        </div>
                                                        <code className="text-[9px] font-mono text-slate-500 group-hover:text-slate-400 bg-black/20 px-2 py-0.5 rounded border border-white/[0.06] transition-colors">
                                                            {syntax.example}
                                                        </code>
                                                    </div>
                                                ))}
                                            </div>
                                        </div>

                                        {/* Quick Tips */}
                                        <div className="flex items-start gap-2 p-3 bg-blue-500/5 border border-blue-500/10 rounded-lg mt-3">
                                            <Terminal className="w-3.5 h-3.5 text-blue-400 mt-0.5 shrink-0" />
                                            <div className="text-[10px] text-slate-400 leading-relaxed">
                                                <span className="text-blue-400 font-semibold">Pro tip:</span> Combine multiple operators with spaces. Example: <code className="text-blue-300 bg-black/30 px-1.5 py-0.5 rounded font-mono text-[9px]">s:200 m:POST h:api</code>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div className="px-5 py-3 bg-gradient-to-r from-slate-900/60 to-slate-900/40 border-t border-white/[0.06] flex justify-between items-center">
                                    <div className="flex items-center gap-1.5">
                                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                        <span className="text-[10px] text-slate-500 font-medium">
                                            {totalItems.toLocaleString()} requests captured
                                        </span>
                                    </div>
                                    <button className="text-[10px] text-cyan-400 hover:text-cyan-300 font-medium transition-colors flex items-center gap-1">
                                        Export filters
                                        <span className="text-slate-600">→</span>
                                    </button>
                                </div>
                            </DialogContent>
                        </Dialog>
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    <Badge variant="outline" className="h-6 border-white/10 text-[10px] text-slate-500 font-mono bg-black/20">
                        {totalItems} REQ
                    </Badge>
                    <Badge variant="outline" className="h-6 border-cyan-500/20 text-[10px] text-cyan-500/70 font-mono bg-cyan-950/10">
                        {hostCount} HOSTS
                    </Badge>
                </div>
            </div>
        </header>
    );
};