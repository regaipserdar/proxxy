
import { Play } from 'lucide-react';

interface DesignerHeaderProps {
    onRun: () => void;
}

export const DesignerHeader = ({ onRun }: DesignerHeaderProps) => {
    return (
        <header className="absolute top-6 left-6 right-[340px] flex items-center justify-between z-40 pointer-events-none">
            <div className="flex items-center gap-3 bg-[#17181C]/80 backdrop-blur-xl border border-white/10 rounded-xl px-4 py-2 pointer-events-auto">
                <span className="text-xs font-medium text-white/40 tracking-wide uppercase">Workspace: Default</span>
            </div>
            <div className="flex items-center gap-4 pointer-events-auto">
                <button onClick={onRun} className="p-3 bg-black/40 backdrop-blur-2xl border border-white/10 rounded-xl text-[#9DCDE8] hover:bg-white/5 transition-all shadow-2xl">
                    <Play size={18} fill="currentColor" />
                </button>
                <button className="bg-[#9DCDE8] text-black px-6 py-2.5 rounded-xl font-bold text-sm shadow-[0_0_30px_rgba(157,205,232,0.4)] transition-transform active:scale-95">
                    Export Flow
                </button>
            </div>
        </header>
    );
};
