import { LucideIcon } from 'lucide-react';

interface MetricCardProps {
    title: string;
    value: string | number;
    sub?: string;
    icon: LucideIcon;
    color: 'emerald' | 'blue' | 'yellow' | 'red';
}

export function MetricCard({ title, value, sub, icon: Icon, color }: MetricCardProps) {
    return (
        <div className="bg-[#16191F] border border-white/5 p-4 rounded-xl flex items-center justify-between">
            <div>
                <h3 className="text-gray-400 text-xs font-bold uppercase tracking-wider">{title}</h3>
                <div className="text-2xl font-bold text-white mt-1">{value}</div>
                {sub && <div className="text-xs text-gray-500 mt-1">{sub}</div>}
            </div>
            <div className={`p-3 rounded-lg bg-${color}-500/20`}>
                <Icon size={24} className={`text-${color}-400`} />
            </div>
        </div>
    );
}
