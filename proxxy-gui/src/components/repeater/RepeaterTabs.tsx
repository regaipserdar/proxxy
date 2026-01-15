import React from 'react';
import { Terminal, X, Plus } from 'lucide-react';
import { RepeaterTask } from './types';

interface RepeaterTabsProps {
    tasks: RepeaterTask[];
    activeTaskId: string | null;
    setActiveTaskId: (id: string) => void;
    removeTask: (id: string) => void;
    handleNewTab: () => void;
    startEditingName: () => void;
}

export const RepeaterTabs: React.FC<RepeaterTabsProps> = ({
    tasks,
    activeTaskId,
    setActiveTaskId,
    removeTask,
    handleNewTab,
    startEditingName
}) => {
    return (
        <div className="h-10 bg-[#0E1015] border-b border-white/5 flex items-center px-4 gap-1 overflow-x-auto no-scrollbar shrink-0">
            {tasks.map(task => (
                <div
                    key={task.id}
                    onClick={() => setActiveTaskId(task.id)}
                    onDoubleClick={() => {
                        setActiveTaskId(task.id);
                        setTimeout(startEditingName, 0);
                    }}
                    className={`group h-8 flex items-center gap-3 px-3 rounded-t-lg transition-all cursor-pointer min-w-[120px] max-w-[200px] border-t-2 ${activeTaskId === task.id
                        ? 'bg-[#161922] border-cyan-500 text-cyan-50 shadow-[0_-10px_20px_rgba(0,0,0,0.3)]'
                        : 'bg-transparent border-transparent text-slate-500 hover:text-slate-300'
                        }`}
                >
                    <Terminal size={10} className={activeTaskId === task.id ? 'text-cyan-400' : 'text-slate-600'} />
                    <span className="text-[10px] font-bold truncate flex-1 tracking-tight">{task.name}</span>
                    <button
                        onClick={(e) => { e.stopPropagation(); removeTask(task.id); }}
                        className="opacity-0 group-hover:opacity-100 p-0.5 hover:bg-white/10 rounded transition-all"
                    >
                        <X size={10} />
                    </button>
                </div>
            ))}
            <button
                onClick={handleNewTab}
                className="p-1.5 hover:bg-white/5 rounded-md text-slate-600 hover:text-cyan-400 transition-all ml-2"
                title="New Tab"
            >
                <Plus size={14} />
            </button>
        </div>
    );
};
