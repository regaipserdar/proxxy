
import { Settings as SettingsIcon, Shield, Server, Bell, User, Database, Globe } from 'lucide-react';

export const SettingsView = () => {
    return (
        <div className="p-12 w-full space-y-12 dotted-bg min-h-full">
            <header className="space-y-1">
                <div className="flex items-center gap-3">
                    <div className="p-2 bg-[#9DCDE8]/10 rounded-lg border border-[#9DCDE8]/20">
                        <SettingsIcon size={20} className="text-[#9DCDE8]" />
                    </div>
                    <h1 className="text-3xl font-bold text-white tracking-tight">System Settings</h1>
                </div>
                <p className="text-white/40 text-sm pl-11">Configure your Orchestrator and Agent nodes</p>
            </header>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
                <div className="md:col-span-1 space-y-2">
                    <SettingsNav icon={<User size={16} />} label="General" active />
                    <SettingsNav icon={<Shield size={16} />} label="Security" />
                    <SettingsNav icon={<Database size={16} />} label="Database" />
                    <SettingsNav icon={<Globe size={16} />} label="Network" />
                    <SettingsNav icon={<Bell size={16} />} label="Notifications" />
                    <SettingsNav icon={<Server size={16} />} label="Agent Defaults" />
                </div>

                <div className="md:col-span-2 space-y-8">
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

                            <div className="flex items-center justify-between p-4 bg-white/5 rounded-2xl border border-white/5">
                                <div>
                                    <p className="text-sm font-bold text-white">Debug Logging</p>
                                    <p className="text-xs text-white/40">Enable verbose output for troubleshooting</p>
                                </div>
                                <div className="w-12 h-6 bg-[#9DCDE8] rounded-full relative cursor-pointer">
                                    <div className="absolute right-1 top-1 w-4 h-4 bg-black rounded-full shadow-sm" />
                                </div>
                            </div>
                        </div>

                        <div className="pt-6">
                            <button className="bg-[#9DCDE8] text-black px-8 py-3 rounded-xl font-bold text-sm shadow-[0_0_20px_rgba(157,205,232,0.2)] hover:scale-[1.02] active:scale-95 transition-all">
                                Save Changes
                            </button>
                        </div>
                    </section>

                    <section className="glass-panel rounded-3xl p-8 border-white/5 space-y-6 border-dashed opacity-50">
                        <h3 className="text-lg font-bold text-white">Advanced Metadata</h3>
                        <p className="text-sm text-white/40">Additional configuration parameters will appear here as features are enabled.</p>
                    </section>
                </div>
            </div>
        </div>
    );
};

const SettingsNav = ({ icon, label, active = false }: { icon: any, label: string, active?: boolean }) => (
    <button className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all ${active ? 'bg-[#9DCDE8]/10 text-[#9DCDE8] border border-[#9DCDE8]/20' : 'text-white/40 hover:text-white/60 hover:bg-white/5 border border-transparent'
        }`}>
        {icon}
        <span className="text-sm font-medium">{label}</span>
    </button>
);
