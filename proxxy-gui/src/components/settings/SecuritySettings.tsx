import { useQuery } from '@apollo/client';
import { Shield, Download, Apple, Info, AlertTriangle } from 'lucide-react';
import { GET_CA_CERT } from '@/graphql/operations';
import { StepItem } from './StepItem';

export const SecuritySettings = () => {
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
    );
};
