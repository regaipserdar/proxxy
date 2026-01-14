
import { useState } from 'react';
import { Settings as SettingsIcon, Shield, Database, Globe, User } from 'lucide-react';
import { SettingsNav } from '@/components/settings/SettingsNav';
import { GeneralSettings } from '@/components/settings/GeneralSettings';
import { SecuritySettings } from '@/components/settings/SecuritySettings';

export const SettingsView = () => {
    const [activeTab, setActiveTab] = useState('Security');

    return (
        <div className="p-12 w-full space-y-12 min-h-full">
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
                    {activeTab === 'General' && <GeneralSettings />}
                    {activeTab === 'Security' && <SecuritySettings />}
                    {activeTab === 'Database' && (
                        <div className="p-8 text-white/40 text-center border border-white/5 rounded-3xl bg-white/[0.02]">
                            Database settings placeholder
                        </div>
                    )}
                    {activeTab === 'Network' && (
                        <div className="p-8 text-white/40 text-center border border-white/5 rounded-3xl bg-white/[0.02]">
                            Network settings placeholder
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};
