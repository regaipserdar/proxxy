import React from 'react';
import { Terminal, Plus } from 'lucide-react';

interface EmptyRepeaterProps {
    handleNewTab: () => void;
}

export const EmptyRepeater: React.FC<EmptyRepeaterProps> = ({ handleNewTab }) => {
    return (
        <div className="flex flex-col h-full bg-[#0A0E14] items-center justify-center gap-6">
            <div className="w-20 h-20 rounded-3xl bg-white/5 flex items-center justify-center border border-white/10 shadow-2xl">
                <Terminal size={40} className="text-slate-600" />
            </div>
            <div className="text-center space-y-4">
                <h2 className="text-lg font-black uppercase tracking-[0.4em] text-slate-400">Terminal Empty</h2>
                <p className="text-sm text-slate-600 font-mono tracking-wider max-w-md mx-auto">
                    No active repeater sessions. Capture traffic or create a manual request to begin.
                </p>
                <button
                    onClick={handleNewTab}
                    className="px-8 py-3 bg-cyan-500/10 border border-cyan-500/20 rounded-xl text-cyan-400 text-xs font-bold uppercase tracking-widest hover:bg-cyan-500/20 transition-all flex items-center gap-2 mx-auto"
                >
                    <Plus size={16} /> New Session
                </button>
            </div>
        </div>
    );
};
