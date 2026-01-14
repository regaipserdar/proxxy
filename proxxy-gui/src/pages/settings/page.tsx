
import { useState } from 'react';
import { useQuery } from '@apollo/client';
import { Settings as SettingsIcon, Shield, Database, Globe, Download, Apple, Info, AlertTriangle, User } from 'lucide-react';
import { GET_CA_CERT } from '@/graphql/operations';

export const SettingsView = () => {
    const [activeTab, setActiveTab] = useState('Security');
    const { data: caData } = useQuery(GET_CA_CERT);

    const downloadCertificate = () => {
        if (!caData?.caCertPem) return;
        const blob = new Blob([caData.caCertPem], { type: 'application/x-x509-ca-cert' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'proxxy-ca.pem';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    };

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

            <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
                <div className="md:col-span-1 space-y-2">
                    <SettingsNav
                        icon={<User size={16} />}
                        label="General"
                        active={activeTab === 'General'}
                        onClick={() => setActiveTab('General')}
                    />
                    <SettingsNav
                        icon={<Shield size={16} />}
                        label="Security"
                        active={activeTab === 'Security'}
                        onClick={() => setActiveTab('Security')}
                    />
                    <SettingsNav
                        icon={<Database size={16} />}
                        label="Database"
                        active={activeTab === 'Database'}
                        onClick={() => setActiveTab('Database')}
                    />
                    <SettingsNav
                        icon={<Globe size={16} />}
                        label="Network"
                        active={activeTab === 'Network'}
                        onClick={() => setActiveTab('Network')}
                    />
                </div>

                <div className="md:col-span-3 space-y-8">
                    {activeTab === 'General' && (
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
                    )}

                    {activeTab === 'Security' && (
                        <div className="space-y-8">
                            <section className="glass-panel rounded-3xl p-8 border-white/5 space-y-6">
                                <div className="flex items-center justify-between">
                                    <div className="space-y-1">
                                        <h3 className="text-lg font-bold text-white flex items-center gap-2">
                                            <Shield size={18} className="text-[#9DCDE8]" />
                                            HTTPS Certificate (CA)
                                        </h3>
                                        <p className="text-sm text-white/40">In order to intercept and inspect HTTPS traffic, you must trust the Orchestrator's Root Certificate Authority.</p>
                                    </div>
                                    <button
                                        onClick={downloadCertificate}
                                        className="flex items-center gap-2 bg-[#9DCDE8]/10 hover:bg-[#9DCDE8]/20 text-[#9DCDE8] border border-[#9DCDE8]/20 px-5 py-2.5 rounded-xl font-bold text-xs transition-all"
                                    >
                                        <Download size={14} /> Download Pem
                                    </button>
                                </div>

                                <div className="bg-amber-500/10 border border-amber-500/20 rounded-2xl p-4 flex gap-4">
                                    <AlertTriangle className="text-amber-500 shrink-0" size={20} />
                                    <div className="space-y-1">
                                        <p className="text-sm font-bold text-amber-500">Security Warning</p>
                                        <p className="text-xs text-amber-500/70 leading-relaxed">Only install this certificate if you trust this Proxxy deployment. This certificate allows the application to decrypt HTTPS traffic on your system.</p>
                                    </div>
                                </div>

                                <div className="space-y-6 pt-4">
                                    <div className="flex items-center gap-2 text-white/80">
                                        <Apple size={16} />
                                        <h4 className="text-sm font-bold uppercase tracking-widest">macOS Installation Steps</h4>
                                    </div>

                                    <div className="grid grid-cols-1 gap-4">
                                        <StepItem
                                            number="1"
                                            title="Download Certificate"
                                            desc="Click the 'Download Pem' button above to save the proxxy-ca.pem file to your Mac."
                                        />
                                        <StepItem
                                            number="2"
                                            title="Add to Keychain"
                                            desc="Double-click the downloaded file or drag it into the 'Keychain Access' app (under 'System' or 'login' keychain)."
                                        />
                                        <StepItem
                                            number="3"
                                            title="Locate 'Proxxy CA'"
                                            desc="Open 'Keychain Access', search for 'Proxxy CA' or 'Orchestrator Root CA' and double-click it."
                                        />
                                        <StepItem
                                            number="4"
                                            title="Trust Certificate"
                                            desc="Expand the 'Trust' section and change 'When using this certificate' to 'Always Trust'. Enter your password when prompted."
                                        />
                                    </div>
                                </div>
                            </section>

                            <section className="glass-panel rounded-3xl p-8 border-white/5 space-y-4">
                                <div className="flex items-center gap-2">
                                    <Info size={16} className="text-blue-400" />
                                    <h3 className="text-[11px] font-bold text-white/40 uppercase tracking-widest">Why is this needed?</h3>
                                </div>
                                <p className="text-xs text-white/40 leading-relaxed">
                                    Modern browsers use SSL Pinning and Certificate Authorities to ensure you are talking directly to the website.
                                    Since Proxxy acts as a "Man-in-the-Middle" to let you debug traffic, your system must recognize Proxxy as a valid
                                    authority that can issue certificates for any domain.
                                </p>
                            </section>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

const StepItem = ({ number, title, desc }: { number: string, title: string, desc: string }) => (
    <div className="flex gap-4 p-4 bg-white/[0.02] border border-white/5 rounded-2xl hover:bg-white/[0.04] transition-all">
        <div className="w-8 h-8 rounded-full bg-[#9DCDE8]/10 border border-[#9DCDE8]/20 flex items-center justify-center shrink-0">
            <span className="text-xs font-bold text-[#9DCDE8]">{number}</span>
        </div>
        <div className="space-y-1">
            <h5 className="text-sm font-bold text-white/90">{title}</h5>
            <p className="text-xs text-white/40 leading-relaxed">{desc}</p>
        </div>
    </div>
);

const SettingsNav = ({ icon, label, active = false, onClick }: { icon: any, label: string, active?: boolean, onClick: () => void }) => (
    <button
        onClick={onClick}
        className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all ${active ? 'bg-[#9DCDE8]/10 text-[#9DCDE8] border border-[#9DCDE8]/20' : 'text-white/40 hover:text-white/60 hover:bg-white/5 border border-transparent'
            }`}>
        {icon}
        <span className="text-sm font-medium">{label}</span>
    </button>
);
