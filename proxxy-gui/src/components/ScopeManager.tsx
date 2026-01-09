
import { useState } from 'react';
import {
  Plus, Trash2, Target,
  Settings,
  FolderOpen, Zap
} from 'lucide-react';
import { ScopeRule, Project } from '../types';

export const ScopeManager = () => {
  const [projects] = useState<Project[]>([
    { id: '1', name: 'Identity API v2', status: 'active', ruleCount: 12 },
    { id: '2', name: 'Payment Gateway', status: 'idle', ruleCount: 8 },
    { id: '3', name: 'Mobile App Backend', status: 'idle', ruleCount: 45 },
  ]);

  const [rules] = useState<ScopeRule[]>([
    { id: '1', type: 'include', enabled: true, protocol: 'https', host: 'api.proxxy.dev', port: '443', path: '/v1/*' },
    { id: '2', type: 'exclude', enabled: true, protocol: 'https', host: 'api.proxxy.dev', port: '443', path: '/v1/auth/*' },
    { id: '3', type: 'include', enabled: true, protocol: 'https', host: 'sandbox.dev.local', port: '8443', path: '*' },
    { id: '4', type: 'include', enabled: false, protocol: 'http', host: 'localhost', port: '8080', path: '*' },
  ]);

  const [activeProjectId, setActiveProjectId] = useState('1');
  const [isAdvancedMode, setIsAdvancedMode] = useState(false);
  const [newRuleType, setNewRuleType] = useState<'include' | 'exclude'>('include');

  return (
    <div className="flex h-full bg-[#0A0E14] overflow-hidden select-none">

      {/* --- Projects Sidebar --- */}
      <div className="w-[300px] border-r border-white/10 flex flex-col bg-[#0D0F13]">
        <div className="h-14 flex items-center justify-between px-6 border-b border-white/10 bg-[#111318]">
          <h3 className="text-[10px] font-bold text-white/40 uppercase tracking-[0.2em]">Target Projects</h3>
          <button className="p-1.5 hover:bg-white/5 rounded-md text-[#9DCDE8] transition-all">
            <Plus size={14} />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-2">
          {projects.map(project => (
            <button
              key={project.id}
              onClick={() => setActiveProjectId(project.id)}
              className={`w-full group px-4 py-3 rounded-xl flex items-center gap-4 transition-all border ${activeProjectId === project.id
                  ? 'bg-[#9DCDE8]/10 border-[#9DCDE8]/20 text-[#9DCDE8] shadow-[0_0_20px_rgba(157,205,232,0.1)]'
                  : 'border-transparent text-white/30 hover:bg-white/5 hover:text-white/60'
                }`}
            >
              <div className={`w-2 h-2 rounded-full ${project.status === 'active' ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]' : 'bg-white/10'
                }`} />
              <div className="flex-1 text-left">
                <div className="text-[12px] font-bold truncate">{project.name}</div>
                <div className="text-[9px] opacity-40 uppercase tracking-widest">{project.ruleCount} Rules Attached</div>
              </div>
              <ChevronRight size={12} className={`opacity-20 group-hover:opacity-100 transition-opacity ${activeProjectId === project.id ? 'opacity-100' : ''}`} />
            </button>
          ))}
        </div>

        <div className="p-6 border-t border-white/5">
          <button className="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-white/5 border border-white/10 text-[11px] font-bold text-white/40 hover:bg-white/10 hover:text-white transition-all">
            <FolderOpen size={14} /> Open Local Project
          </button>
        </div>
      </div>

      {/* --- Rules Manager Area --- */}
      <div className="flex-1 flex flex-col min-w-0">

        {/* Rules Header */}
        <div className="h-14 border-b border-white/10 flex items-center justify-between px-8 bg-[#111318]">
          <div className="flex items-center gap-6">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-lg bg-orange-500/10 flex items-center justify-center border border-orange-500/20">
                <Target size={16} className="text-orange-400" />
              </div>
              <h2 className="text-sm font-bold text-white uppercase tracking-wider">
                {projects.find(p => p.id === activeProjectId)?.name}
              </h2>
            </div>

            <div className="h-4 w-px bg-white/10" />

            <div className="flex items-center gap-1 bg-black/40 rounded-lg p-1 border border-white/5">
              <button
                onClick={() => setIsAdvancedMode(false)}
                className={`px-3 py-1 text-[10px] font-bold uppercase rounded-md transition-all ${!isAdvancedMode ? 'bg-[#9DCDE8] text-black shadow-lg' : 'text-white/40 hover:text-white'
                  }`}
              >Simple</button>
              <button
                onClick={() => setIsAdvancedMode(true)}
                className={`px-3 py-1 text-[10px] font-bold uppercase rounded-md transition-all ${isAdvancedMode ? 'bg-[#9DCDE8] text-black shadow-lg' : 'text-white/40 hover:text-white'
                  }`}
              >Regex</button>
            </div>
          </div>

          <div className="flex items-center gap-4">
            <button className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-[11px] font-bold text-emerald-400 hover:bg-emerald-500/20 transition-all">
              <Zap size={14} fill="currentColor" /> Apply to Core
            </button>
            <button className="p-2.5 rounded-lg hover:bg-white/5 text-white/40 transition-colors">
              <Settings size={16} />
            </button>
          </div>
        </div>

        {/* Rule Editor Section */}
        <div className="p-8 space-y-8 flex-1 overflow-y-auto w-full mx-auto">

          <div className="glass-panel rounded-3xl p-8 border-white/5 space-y-6 bg-white/[0.01]">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-bold text-white flex items-center gap-3">
                <Plus size={18} className="text-[#9DCDE8]" />
                Add New Scope Definition
              </h3>
              <div className="flex bg-black/40 p-1 rounded-xl border border-white/5">
                <button
                  onClick={() => setNewRuleType('include')}
                  className={`px-4 py-1.5 rounded-lg text-xs font-bold uppercase tracking-widest outline-none cursor-pointer ${newRuleType === 'include' ? 'bg-emerald-500/20 text-emerald-400' : 'text-white/20'}`}
                >Include</button>
                <button
                  onClick={() => setNewRuleType('exclude')}
                  className={`px-4 py-1.5 rounded-lg text-xs font-bold uppercase tracking-widest outline-none cursor-pointer ${newRuleType === 'exclude' ? 'bg-red-500/20 text-red-400' : 'text-white/20'}`}
                >Exclude</button>
              </div>
            </div>

            <div className="grid grid-cols-4 gap-4">
              <InputGroup label="Protocol" defaultValue="https" />
              <div className="col-span-2">
                <InputGroup label="Host / Domain Match" placeholder="*.example.com" />
              </div>
              <InputGroup label="Port" defaultValue="443" />
              <div className="col-span-4">
                <InputGroup label="Resource Path Pattern" placeholder="/api/v1/*" />
              </div>
            </div>

            <div className="flex items-center justify-end gap-3 pt-4">
              <button className="px-6 py-2.5 rounded-xl text-white/40 hover:text-white transition-colors text-xs font-bold uppercase tracking-widest">Reset Form</button>
              <button className="px-8 py-2.5 rounded-xl bg-white text-black text-xs font-bold hover:bg-[#9DCDE8] transition-all shadow-xl uppercase tracking-widest">
                Add To Scope
              </button>
            </div>
          </div>

          <div className="space-y-4">
            <h3 className="text-[10px] font-bold text-white/20 uppercase tracking-[0.2em] px-2 flex items-center justify-between">
              Active Scope Rules
              <span>{rules.length} Items</span>
            </h3>

            <div className="grid grid-cols-1 gap-3">
              {rules.map(rule => (
                <div
                  key={rule.id}
                  className={`flex items-center gap-6 px-6 py-4 rounded-2xl border transition-all duration-300 group hover:bg-white/[0.02] bg-white/[0.01] ${rule.enabled ? 'border-white/10' : 'border-white/5 grayscale opacity-40'
                    }`}
                >
                  <div className={`px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest border ${rule.type === 'include' ? 'bg-emerald-500/5 text-emerald-500 border-emerald-500/20' : 'bg-red-500/5 text-red-500 border-red-500/20'
                    }`}>
                    {rule.type}
                  </div>

                  <div className="flex-1 flex items-center gap-1 font-mono text-xs">
                    <span className="text-white/30">{rule.protocol}://</span>
                    <span className="text-white font-bold">{rule.host}</span>
                    <span className="text-white/30 truncate max-w-[300px]">: {rule.port}{rule.path}</span>
                  </div>

                  <div className="flex items-center gap-6">
                    <div className={`text-[10px] font-bold font-mono ${rule.enabled ? 'text-emerald-500' : 'text-white/20'}`}>
                      {rule.enabled ? 'ACTIVE' : 'DISABLED'}
                    </div>
                    <div className="flex items-center gap-1 opacity-10 group-hover:opacity-100 transition-opacity">
                      <button className="p-2 hover:bg-white/5 rounded-lg text-white/40 hover:text-white"><Zap size={14} /></button>
                      <button className="p-2 hover:bg-white/5 rounded-lg text-white/40 hover:text-white"><Plus size={14} /></button>
                      <button className="p-2 hover:bg-white/5 rounded-lg text-white/40 hover:text-red-400"><Trash2 size={14} /></button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

        </div>
      </div>
    </div>
  );
};

const InputGroup = ({ label, placeholder, defaultValue }: any) => (
  <div className="space-y-2">
    <label className="text-[10px] font-bold text-white/20 uppercase tracking-widest ml-1">{label}</label>
    <input
      type="text"
      placeholder={placeholder}
      defaultValue={defaultValue}
      className="w-full bg-black/60 border border-white/5 rounded-xl px-4 py-2.5 text-xs text-white/80 focus:outline-none focus:border-[#9DCDE8]/40 font-mono transition-all"
    />
  </div>
);

const ChevronRight = ({ size, className }: { size: number, className: string }) => <svg className={className} width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m9 18 6-6-6-6" /></svg>;
