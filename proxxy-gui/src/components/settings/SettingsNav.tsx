
export const SettingsNav = ({ icon, label, active = false, onClick }: { icon: any, label: string, active?: boolean, onClick: () => void }) => (
    <button
        onClick={onClick}
        className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all ${active ? 'bg-[#9DCDE8]/10 text-[#9DCDE8] border border-[#9DCDE8]/20' : 'text-white/40 hover:text-white/60 hover:bg-white/5 border border-transparent'
            }`}>
        {icon}
        <span className="text-sm font-medium">{label}</span>
    </button>
);
