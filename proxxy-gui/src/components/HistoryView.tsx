
import { TrafficTable } from './TrafficTable';
import { History, Search, Download, Trash2 } from 'lucide-react';

export const HistoryView = () => {
    return (
        <div className="p-12 w-full space-y-12 dotted-bg min-h-full">
            <header className="flex items-center justify-between">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <div className="p-2 bg-purple-500/10 rounded-lg border border-purple-500/20">
                            <History size={20} className="text-purple-400" />
                        </div>
                        <h1 className="text-3xl font-bold text-white tracking-tight">Traffic History</h1>
                    </div>
                    <p className="text-white/40 text-sm pl-11">Review and analyze intercepted HTTP transactions</p>
                </div>

                <div className="flex items-center gap-4">
                    <div className="relative group">
                        <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-purple-400 transition-colors" size={16} />
                        <input type="text" placeholder="Search logs..." className="bg-black/40 border border-white/10 rounded-xl pl-12 pr-4 py-3 text-sm focus:outline-none focus:ring-1 focus:ring-purple-500/40 transition-all placeholder:text-white/10 text-white/80 w-64" />
                    </div>
                    <button className="p-3 bg-white/5 border border-white/10 rounded-xl text-white/60 hover:text-white transition-all"><Download size={18} /></button>
                    <button className="p-3 bg-red-400/5 border border-red-400/10 rounded-xl text-red-400/60 hover:text-red-400 transition-all"><Trash2 size={18} /></button>
                </div>
            </header>

            <div className="glass-panel rounded-3xl overflow-hidden border-white/5 shadow-2xl bg-black/20">
                <TrafficTable />
            </div>
        </div>
    );
};
