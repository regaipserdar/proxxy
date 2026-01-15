
import { ReactNode } from 'react';
import { NavLink } from 'react-router-dom';
import {
  Home, GitMerge, Type as TextIcon, Search, Radio, Filter,
  Edit3, Repeat, Terminal, Settings, ShieldCheck, Target, Send, Server, ListTree
} from 'lucide-react';
import { NodeType } from '../types';

export const LeftSidebar = () => (
  <aside data-tauri-drag-region className="w-[72px] h-full glass-panel border-r border-white/10 flex flex-col items-center py-8 gap-6 z-50">

    <div className="w-10 h-10 bg-white/5 rounded-xl flex items-center justify-center border border-white/10 mb-4 shadow-2xl">
      <span className="text-xl font-bold italic text-[#9DCDE8]">P</span>
    </div>

    <SideNavLink to="/" icon={<Home size={20} />} label="Dashboard" />
    <SideNavLink to="/agents" icon={<Server size={20} />} label="Agents" />
    <SideNavLink to="/designer" icon={<GitMerge size={20} />} label="Workflow Designer" />
    <SideNavLink to="/scope" icon={<Target size={20} />} label="Scope Manager" />
    <SideNavLink to="/traffic-tree" icon={<ListTree size={20} />} label="Traffic Tree" />
    <SideNavLink to="/repeater" icon={<Send size={20} />} label="HTTP Repeater" />
    <SideNavLink to="/intruder" icon={<ShieldCheck size={20} />} label="Intruder" />

    <div className="mt-auto flex flex-col gap-6">
      <SideNavLink to="/settings" icon={<Settings size={18} />} label="Settings" />
    </div>
  </aside>
);

const SideNavLink = ({ to, icon, label, disabled = false }: { to: string, icon: ReactNode, label: string, disabled?: boolean }) => (
  <NavLink
    to={disabled ? '#' : to}
    className={({ isActive }) => `
      w-12 h-12 flex items-center justify-center rounded-2xl transition-all duration-300 relative group
      ${isActive && !disabled ? 'bg-[#9DCDE8]/10 text-[#9DCDE8] ring-1 ring-[#9DCDE8]/30 shadow-[0_0_20px_rgba(157,205,232,0.2)]' : 'text-white/30 hover:text-white/60 hover:bg-white/5'}
      ${disabled ? 'opacity-20 cursor-not-allowed' : ''}
    `}
  >
    {({ isActive }) => (
      <>
        {icon}
        {isActive && !disabled && (
          <div className="absolute -left-1 w-1 h-6 bg-[#9DCDE8] rounded-r-full shadow-[0_0_10px_#9DCDE8]" />
        )}
        <div className="absolute left-[80px] px-3 py-2 bg-[#17181C] border border-white/10 rounded-xl text-[10px] font-bold text-white uppercase tracking-widest opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-[100] shadow-2xl">
          {label}
        </div>
      </>
    )}
  </NavLink>
);

export const RightSidebar = () => (
  <aside className="w-[320px] h-full border-l border-white/10 flex flex-col p-8 gap-8 overflow-y-auto z-50 animate-in slide-in-from-right duration-300 dotted-bg">
    <div className="relative group">
      <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-[#9DCDE8] transition-colors" size={16} />
      <input type="text" placeholder="Search components..." className="w-full bg-black/40 border border-white/10 rounded-xl pl-12 pr-4 py-3.5 text-sm focus:outline-none focus:ring-1 focus:ring-[#9DCDE8]/40 transition-all placeholder:text-white/10 text-white/80 font-mono" />
    </div>
    <div className="flex flex-col gap-8">
      <ToolSection title="Triggers">
        <ToolRow icon={<Radio size={16} />} label="Proxy Listener" type={NodeType.TRIGGER} />
        <ToolRow icon={<TextIcon size={16} />} label="Manual Start" type={NodeType.TRIGGER} />
      </ToolSection>
      <ToolSection title="Logic Filters">
        <ToolRow icon={<Filter size={16} />} label="Regex Matcher" type={NodeType.MATCHER} />
        <ToolRow icon={<ShieldCheck size={16} />} label="Status Guard" type={NodeType.MATCHER} />
        <ToolRow icon={<Repeat size={16} />} label="Payload Repeater" type={NodeType.REPEATER} />
      </ToolSection>
      <ToolSection title="Manipulation">
        <ToolRow icon={<Edit3 size={16} />} label="Header Modifier" type={NodeType.MODIFIER} />
        <ToolRow icon={<Edit3 size={16} />} label="Body Inserter" type={NodeType.MODIFIER} />
        <ToolRow icon={<Terminal size={16} />} label="Logger" type={NodeType.SINK} />
      </ToolSection>
    </div>
  </aside>
);

const ToolSection = ({ title, children }: { title: string, children?: ReactNode }) => (
  <section>
    <h3 className="text-[10px] font-bold text-white/20 uppercase tracking-[0.2em] mb-4">{title}</h3>
    <div className="flex flex-col gap-2">{children}</div>
  </section>
);

const ToolRow = ({ icon, label, type }: { icon: ReactNode, label: string, type: NodeType }) => {
  const onDragStart = (event: React.DragEvent) => {
    event.dataTransfer.setData('application/reactflow-type', type);
    event.dataTransfer.setData('application/reactflow-label', label);
    event.dataTransfer.effectAllowed = 'move';
  };
  return (
    <button draggable onDragStart={onDragStart} className="w-full flex items-center gap-4 px-4 py-3.5 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 hover:border-[#9DCDE8]/20 transition-all group text-left shadow-lg cursor-grab active:cursor-grabbing">
      <div className="text-white/20 group-hover:text-[#9DCDE8] transition-colors">{icon}</div>
      <span className="text-[12px] font-bold text-white/50 group-hover:text-white/90 uppercase tracking-tight">{label}</span>
    </button>
  );
};
