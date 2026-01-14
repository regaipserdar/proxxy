
export const StepItem = ({ number, title, desc }: { number: string, title: string, desc: string }) => (
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
