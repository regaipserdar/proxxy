import React from 'react';
import { Search, Terminal } from 'lucide-react';
import { RepeaterTask } from './types';

interface RepeaterInspectorProps {
    activeTask: RepeaterTask | undefined;
    updateTask: (id: string, updates: Partial<RepeaterTask>) => void;
    reqSearch: string;
    setReqSearch: (val: string) => void;
    resSearch: string;
    setResSearch: (val: string) => void;
}

export const RepeaterInspector: React.FC<RepeaterInspectorProps> = ({
    activeTask,
    updateTask,
    reqSearch,
    setReqSearch,
    resSearch,
    setResSearch
}) => {
    return (
        <div className="flex-1 grid grid-cols-2 gap-px bg-white/5 overflow-hidden">
            {/* Request Side */}
            <div className="flex flex-col bg-[#0D0F13]">
                <div className="px-4 py-2 border-b border-white/5 flex items-center justify-between bg-white/[0.02]">
                    <div className="flex items-center gap-3 flex-1">
                        <span className="text-[10px] font-bold uppercase tracking-widest text-[#9DCDE8]">Request</span>
                        <div className="relative group max-w-[200px]">
                            <Search size={10} className="absolute left-2 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-[#9DCDE8]" />
                            <input
                                type="text"
                                placeholder="Find in request..."
                                value={reqSearch}
                                onChange={(e) => setReqSearch(e.target.value)}
                                className="w-full bg-black/20 border border-white/5 rounded px-6 py-1 text-[10px] text-white/60 focus:outline-none focus:border-[#9DCDE8]/30 focus:text-white font-mono transition-colors"
                            />
                        </div>
                    </div>
                </div>
                <textarea
                    value={activeTask?.request || ''}
                    onChange={(e) => activeTask && updateTask(activeTask.id, { request: e.target.value })}
                    className="flex-1 bg-transparent p-6 font-mono text-sm text-white/80 outline-none resize-none leading-relaxed placeholder:text-white/10"
                    spellCheck={false}
                />
            </div>

            {/* Response Side */}
            <div className="flex flex-col bg-[#080A0E]">
                <div className="px-4 py-2 border-b border-white/5 flex items-center justify-between bg-white/[0.02]">
                    <div className="flex items-center gap-3 flex-1">
                        <span className="text-[10px] font-bold uppercase tracking-widest text-emerald-400">Response</span>
                        <div className="relative group max-w-[200px]">
                            <Search size={10} className="absolute left-2 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-emerald-400" />
                            <input
                                type="text"
                                placeholder="Find in response..."
                                value={resSearch}
                                onChange={(e) => setResSearch(e.target.value)}
                                className="w-full bg-black/20 border border-white/5 rounded px-6 py-1 text-[10px] text-white/60 focus:outline-none focus:border-emerald-500/30 focus:text-white font-mono transition-colors"
                            />
                        </div>
                    </div>
                    {activeTask?.response && (
                        <div className="flex gap-3 items-center text-[10px] font-mono">
                            <span className="text-emerald-400 bg-emerald-400/10 px-2 py-0.5 rounded border border-emerald-400/20">200 OK</span>
                            <span className="text-white/40">245ms</span>
                            <span className="text-white/40">{Math.round(activeTask.response.length / 1024 * 10) / 10} KB</span>
                        </div>
                    )}
                </div>
                {activeTask?.response ? (
                    <div className="flex-1 overflow-auto">
                        <pre className="p-6 font-mono text-sm leading-relaxed text-white/80">
                            {activeTask.response.split('\n').map((line, i) => {
                                const isMatch = resSearch && line.toLowerCase().includes(resSearch.toLowerCase());
                                return (
                                    <div key={i} className={`px-1 rounded-sm ${isMatch ? 'bg-[#9DCDE8]/20 text-[#9DCDE8] font-bold' : ''}`}>
                                        {line}
                                    </div>
                                );
                            })}
                        </pre>
                    </div>
                ) : (
                    <div className="flex-1 flex flex-col items-center justify-center opacity-20 gap-4 select-none">
                        <div className="w-16 h-16 rounded-2xl bg-white/10 flex items-center justify-center">
                            <Terminal size={32} />
                        </div>
                        <div className="text-center">
                            <p className="text-xs font-bold uppercase tracking-widest mb-1">Waiting for Response</p>
                            <p className="text-[10px] text-white/50">Select an agent and hit send to inspect</p>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};
