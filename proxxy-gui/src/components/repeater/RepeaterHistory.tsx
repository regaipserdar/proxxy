import React from 'react';
import { useQuery } from '@apollo/client';
import { GET_REPEATER_HISTORY } from '@/graphql/operations';
import { Clock, ChevronRight, AlertCircle, CheckCircle2, XCircle } from 'lucide-react';
import { format } from 'date-fns';

interface RepeaterHistoryProps {
    tabId: string;
    onSelectExecution: (execution: any) => void;
    onClose: () => void;
}

export const RepeaterHistory: React.FC<RepeaterHistoryProps> = ({ tabId, onSelectExecution, onClose }) => {
    const { data, loading, error } = useQuery(GET_REPEATER_HISTORY, {
        variables: { tabId, limit: 50 },
        fetchPolicy: 'network-only'
    });

    const history = data?.repeaterHistory || [];

    return (
        <div className="flex flex-col h-full bg-[#0D0F13] border-l border-white/10 w-80 animate-in slide-in-from-right duration-200">
            <div className="flex items-center justify-between px-4 py-3 border-b border-white/10 bg-white/[0.02]">
                <div className="flex items-center gap-2">
                    <Clock size={14} className="text-[#9DCDE8]" />
                    <span className="text-[11px] font-bold uppercase tracking-wider text-white/70">Execution History</span>
                </div>
                <button
                    onClick={onClose}
                    className="text-white/40 hover:text-white transition-colors"
                >
                    <ChevronRight size={16} />
                </button>
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar">
                {loading ? (
                    <div className="flex flex-col items-center justify-center h-40 gap-3 opacity-50 text-[10px] font-mono">
                        <div className="w-4 h-4 border-2 border-[#9DCDE8]/30 border-t-[#9DCDE8] rounded-full animate-spin" />
                        <span>Loading History...</span>
                    </div>
                ) : error ? (
                    <div className="p-6 text-center opacity-40">
                        <AlertCircle size={24} className="mx-auto mb-2 text-red-400" />
                        <p className="text-[10px]">Failed to load history</p>
                    </div>
                ) : history.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-40 opacity-20 gap-2 grayscale">
                        <Clock size={32} />
                        <p className="text-[10px] uppercase font-bold tracking-widest">No history yet</p>
                    </div>
                ) : (
                    <div className="divide-y divide-white/5">
                        {history.map((exec: any) => {
                            const date = new Date(parseInt(exec.executedAt) * 1000);
                            const isSuccess = exec.statusCode >= 200 && exec.statusCode < 300;
                            const isError = !isSuccess && exec.statusCode !== 0;

                            return (
                                <button
                                    key={exec.id}
                                    onClick={() => onSelectExecution(exec)}
                                    className="w-full px-4 py-3 text-left hover:bg-white/[0.03] transition-colors group relative overflow-hidden"
                                >
                                    <div className="flex items-start justify-between mb-1">
                                        <div className="flex items-center gap-2">
                                            {isSuccess ? (
                                                <CheckCircle2 size={12} className="text-emerald-400" />
                                            ) : isError ? (
                                                <XCircle size={12} className="text-red-400" />
                                            ) : (
                                                <AlertCircle size={12} className="text-amber-400" />
                                            )}
                                            <span className={`text-[11px] font-bold font-mono ${isSuccess ? 'text-emerald-400' : isError ? 'text-red-400' : 'text-amber-400'
                                                }`}>
                                                {exec.statusCode || 'FAILED'}
                                            </span>
                                        </div>
                                        <span className="text-[9px] text-white/30 font-mono">
                                            {format(date, 'HH:mm:ss')}
                                        </span>
                                    </div>

                                    <div className="text-[10px] text-white/50 font-mono truncate mb-1">
                                        <span className="text-[#9DCDE8]/70 mr-1">{exec.requestData?.method}</span>
                                        {exec.requestData?.url}
                                    </div>

                                    <div className="flex items-center gap-3 text-[9px] text-white/20 font-mono">
                                        <span>{exec.durationMs}ms</span>
                                        <span>{exec.agentId.split('-')[0]}</span>
                                    </div>

                                    {/* Selection indicator bubble */}
                                    <div className="absolute right-2 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <ChevronRight size={12} className="text-[#9DCDE8]" />
                                    </div>
                                </button>
                            );
                        })}
                    </div>
                )}
            </div>
        </div>
    );
};
