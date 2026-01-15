import React, { useRef, useEffect } from 'react';
import { Edit2, Server, Globe, ChevronDown, Check, Wifi, History, Send } from 'lucide-react';
import { RepeaterAgent, RepeaterTask } from './types';

interface RepeaterToolbarProps {
    activeTask: RepeaterTask | undefined;
    isEditingName: boolean;
    editingNameValue: string;
    setEditingNameValue: (val: string) => void;
    saveName: () => void;
    startEditingName: () => void;
    agents: RepeaterAgent[];
    selectedAgentId: string | null;
    setSelectedAgentId: (id: string) => void;
    isAgentMenuOpen: boolean;
    setIsAgentMenuOpen: (open: boolean) => void;
    handleSend: () => void;
    isSending: boolean;
    isHistoryOpen: boolean;
    onToggleHistory: () => void;
}

export const RepeaterToolbar: React.FC<RepeaterToolbarProps> = ({
    activeTask,
    isEditingName,
    editingNameValue,
    setEditingNameValue,
    saveName,
    startEditingName,
    agents,
    selectedAgentId,
    setSelectedAgentId,
    isAgentMenuOpen,
    setIsAgentMenuOpen,
    handleSend,
    isSending,
    isHistoryOpen,
    onToggleHistory
}) => {
    const dropdownRef = useRef<HTMLDivElement>(null);
    const selectedAgent = agents.find(a => a.id === selectedAgentId) || agents[0];

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsAgentMenuOpen(false);
            }
        }
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, [setIsAgentMenuOpen]);

    return (
        <div className="h-14 border-b border-white/10 flex items-center justify-between px-6 bg-[#111318] shrink-0">
            <div className="flex items-center gap-6">
                <div className="flex items-center gap-3">
                    <div className="flex flex-col leading-none">
                        <span className="text-[10px] font-black text-[#9DCDE8] uppercase tracking-widest mb-1 italic">Active Task</span>
                        <div className="flex items-center gap-2 group">
                            {isEditingName ? (
                                <input
                                    autoFocus
                                    value={editingNameValue}
                                    onChange={(e) => setEditingNameValue(e.target.value)}
                                    onBlur={saveName}
                                    onKeyDown={(e) => e.key === 'Enter' && saveName()}
                                    className="bg-black/40 border border-cyan-500/50 rounded px-2 py-0.5 text-[12px] font-bold text-white outline-none w-[200px]"
                                />
                            ) : (
                                <>
                                    <span
                                        onClick={startEditingName}
                                        className="text-[12px] font-bold text-white uppercase tracking-wider truncate max-w-[300px] cursor-text hover:text-cyan-400 transition-colors"
                                    >
                                        {activeTask?.name}
                                    </span>
                                    <Edit2 size={10} className="opacity-0 group-hover:opacity-40 text-cyan-400 transition-opacity cursor-pointer" onClick={startEditingName} />
                                </>
                            )}
                        </div>
                    </div>
                </div>

                <div className="h-4 w-px bg-white/10" />

                <div className="relative" ref={dropdownRef}>
                    <button
                        onClick={() => setIsAgentMenuOpen(!isAgentMenuOpen)}
                        className="flex items-center gap-2 px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 transition-colors border border-white/5 hover:border-white/10 group"
                    >
                        {selectedAgent?.type === 'local' ? <Server size={14} className="text-emerald-400" /> : <Globe size={14} className="text-blue-400" />}
                        <div className="flex flex-col items-start leading-none">
                            <span className="text-[10px] text-white/40 font-bold uppercase tracking-wider">Egress Node</span>
                            <span className="text-[12px] font-medium text-white group-hover:text-[#9DCDE8] transition-colors">
                                {selectedAgent?.name || 'Select Agent'}
                            </span>
                        </div>
                        <ChevronDown size={12} className={`text-white/40 ml-2 transition-transform ${isAgentMenuOpen ? 'rotate-180' : ''}`} />
                    </button>

                    {isAgentMenuOpen && (
                        <div className="absolute top-full left-0 mt-2 w-64 bg-[#1A1D24] border border-white/10 rounded-lg shadow-2xl z-50 overflow-hidden ring-1 ring-black/50">
                            <div className="px-3 py-2 bg-black/20 text-[10px] font-bold text-white/40 uppercase tracking-wider border-b border-white/5">
                                Available Agents
                            </div>
                            {agents.map((agent) => (
                                <button
                                    key={agent.id}
                                    onClick={() => { setSelectedAgentId(agent.id); setIsAgentMenuOpen(false); }}
                                    disabled={agent.status.toLowerCase() !== 'online'}
                                    className={`w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors border-b border-white/5 last:border-0
                                      ${selectedAgentId === agent.id ? 'bg-[#9DCDE8]/10' : 'hover:bg-white/5'}
                                      ${agent.status.toLowerCase() !== 'online' ? 'opacity-50 cursor-not-allowed' : ''}
                                    `}
                                >
                                    <div className={`w-2 h-2 rounded-full ${agent.status.toLowerCase() === 'online' ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]' : 'bg-red-500'}`} />
                                    <div className="flex-1">
                                        <div className="text-[12px] font-medium text-white flex items-center gap-2">
                                            {agent.name}
                                            {selectedAgentId === agent.id && <Check size={12} className="text-[#9DCDE8]" />}
                                        </div>
                                        <div className="flex items-center gap-2 mt-0.5">
                                            <span className="text-[10px] text-white/40 uppercase">{agent.hostname}</span>
                                            {agent.status.toLowerCase() === 'online' && (
                                                <span className="flex items-center gap-1 text-[10px] text-emerald-400/80 bg-emerald-400/10 px-1 rounded">
                                                    <Wifi size={8} /> {agent.version}
                                                </span>
                                            )}
                                        </div>
                                    </div>
                                </button>
                            ))}
                        </div>
                    )}
                </div>
            </div>

            <div className="flex items-center gap-3">
                <button
                    onClick={() => onToggleHistory()}
                    className={`flex items-center gap-2 px-4 py-2 rounded-lg text-[11px] font-bold border transition-all ${isHistoryOpen
                        ? 'bg-[#9DCDE8]/20 text-[#9DCDE8] border-[#9DCDE8]/30 shadow-[0_0_15px_rgba(157,205,232,0.1)]'
                        : 'bg-white/5 text-white/60 hover:text-white hover:bg-white/10 border-transparent hover:border-white/10'
                        }`}
                >
                    <History size={14} /> History
                </button>

                <button
                    onClick={handleSend}
                    disabled={isSending || !selectedAgent || selectedAgent.status.toLowerCase() !== 'online'}
                    className="group flex items-center gap-2 px-6 py-2 rounded-lg bg-[#9DCDE8] text-[#0A0E14] text-[11px] font-bold hover:bg-white transition-all shadow-[0_0_20px_rgba(157,205,232,0.1)] hover:shadow-[0_0_20px_rgba(255,255,255,0.3)] disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    {isSending ? (
                        <span className="flex items-center gap-2">Sending...</span>
                    ) : (
                        <>
                            Send Request <Send size={14} className="group-hover:translate-x-0.5 transition-transform" />
                        </>
                    )}
                </button>
            </div>
        </div>
    );
};
