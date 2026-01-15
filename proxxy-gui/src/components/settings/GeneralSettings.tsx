
export const GeneralSettings = () => {
    return (
        <section className="glass-panel rounded-3xl p-8 border-white/5 space-y-6">
            <h3 className="text-lg font-bold text-white">General Configuration</h3>
            <div className="space-y-4">
                <div className="space-y-2">
                    <label className="text-[11px] font-bold text-white/30 uppercase tracking-widest ml-1">Orchestrator Name</label>
                    <input
                        className="w-full bg-black/40 border border-white/5 rounded-xl px-4 py-3 text-sm text-white focus:outline-none focus:border-[#9DCDE8]/50 transition-colors"
                        defaultValue="proxxy-core-01"
                    />
                </div>
                <div className="space-y-2">
                    <label className="text-[11px] font-bold text-white/30 uppercase tracking-widest ml-1">Admin Port</label>
                    <input
                        className="w-full bg-black/40 border border-white/5 rounded-xl px-4 py-3 text-sm text-white focus:outline-none focus:border-[#9DCDE8]/50 transition-colors"
                        defaultValue="9090"
                    />
                </div>
            </div>
            <div className="pt-6">
                <button className="bg-[#9DCDE8] text-black px-8 py-3 rounded-xl font-bold text-sm shadow-[0_0_20px_rgba(157,205,232,0.2)] hover:scale-[1.02] active:scale-95 transition-all">
                    Save Changes
                </button>
            </div>
        </section>
    );
};
