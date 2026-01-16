
import { ProxxyNode, NodeType } from '@/types';
import { Settings, X, Globe, Tag, Terminal } from 'lucide-react';

export const PropertySidebar = ({ node, onUpdate, onClose }: { node: ProxxyNode, onUpdate: (id: string, updates: any) => void, onClose: () => void, key?: string }) => {
  const config = node.data.config;

  const handleChange = (key: string, value: any) => {
    onUpdate(node.id, { ...config, [key]: value });
  };

  return (
    <aside className="w-[340px] h-full border-l border-white/10 flex flex-col z-[60] shadow-2xl animate-in slide-in-from-right duration-300 dotted-bg">
      <div className="p-6 border-b border-white/5 flex items-center justify-between bg-black/20 backdrop-blur-md">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-[#9DCDE8]/10 flex items-center justify-center border border-[#9DCDE8]/20">
            <Settings size={16} className="text-[#9DCDE8]" />
          </div>
          <div>
            <h2 className="text-sm font-bold text-white uppercase tracking-wider">{node.data.label}</h2>
            <p className="text-[10px] text-white/30 font-medium uppercase tracking-tight">{node.data.type} configuration</p>
          </div>
        </div>
        <button onClick={onClose} className="p-2 hover:bg-white/5 rounded-lg text-white/20 hover:text-white transition-colors">
          <X size={18} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-6 space-y-8">
        {/* General section */}
        <section className="space-y-4">
          <label className="text-[10px] font-bold text-white/20 uppercase tracking-[0.2em]">General</label>
          <div className="space-y-3">
            <div className="relative">
              <span className="absolute left-3 top-1/2 -translate-y-1/2 text-white/20"><Tag size={14} /></span>
              <input
                type="text"
                value={node.data.label}
                onChange={(e) => onUpdate(node.id, { label: e.target.value })}
                className="w-full bg-black/40 border border-white/10 rounded-xl pl-10 pr-4 py-3 text-sm focus:border-[#9DCDE8]/50 focus:outline-none transition-all text-white/80"
              />
            </div>
          </div>
        </section>

        {/* Properties section */}
        <section className="space-y-4">
          <label className="text-[10px] font-bold text-white/20 uppercase tracking-[0.2em]">Properties</label>

          <div className="space-y-5">
            {node.data.type === NodeType.MATCHER && (
              <>
                <div className="space-y-2">
                  <span className="text-xs text-white/40 font-medium px-1">Pattern (Regex)</span>
                  <input
                    type="text"
                    placeholder="^/api/v1/.*"
                    value={config.pattern || ''}
                    onChange={(e) => handleChange('pattern', e.target.value)}
                    className="w-full bg-black/60 border border-white/5 rounded-xl px-4 py-3 text-sm font-mono text-[#9DCDE8]"
                  />
                </div>
                <div className="space-y-2">
                  <span className="text-xs text-white/40 font-medium px-1">Target Component</span>
                  <select className="w-full bg-black/60 border border-white/5 rounded-xl px-4 py-3 text-sm text-white/60 focus:outline-none appearance-none cursor-pointer">
                    <option>Response Body</option>
                    <option>Request Headers</option>
                    <option>Status Code</option>
                  </select>
                </div>
              </>
            )}

            {node.data.type === NodeType.MODIFIER && (
              <>
                <div className="space-y-2">
                  <span className="text-xs text-white/40 font-medium px-1">Header Name</span>
                  <input
                    type="text"
                    placeholder="X-Custom-Header"
                    value={config.headerKey || ''}
                    onChange={(e) => handleChange('headerKey', e.target.value)}
                    className="w-full bg-black/60 border border-white/5 rounded-xl px-4 py-3 text-sm text-white/80"
                  />
                </div>
                <div className="space-y-2">
                  <span className="text-xs text-white/40 font-medium px-1">Header Value</span>
                  <textarea
                    rows={3}
                    placeholder="Value to inject..."
                    value={config.headerValue || ''}
                    onChange={(e) => handleChange('headerValue', e.target.value)}
                    className="w-full bg-black/60 border border-white/5 rounded-xl px-4 py-3 text-sm text-white/80 resize-none"
                  />
                </div>
              </>
            )}

            {node.data.type === NodeType.TRIGGER && (
              <div className="p-4 rounded-xl bg-emerald-500/5 border border-emerald-500/10 space-y-3">
                <div className="flex items-center gap-2 text-emerald-400">
                  <Globe size={14} />
                  <span className="text-xs font-bold">Proxy Listener Active</span>
                </div>
                <p className="text-[10px] text-white/30 leading-relaxed">
                  Automatically intercepts all HTTP/S traffic from port 8080 and injects it into this flow.
                </p>
              </div>
            )}
          </div>
        </section>

        {/* Local Insights */}
        <section className="pt-4 border-t border-white/5">
          <div className="flex items-center gap-2 mb-4">
            <Terminal size={12} className="text-white/20" />
            <span className="text-[10px] font-bold text-white/20 uppercase tracking-widest">Local Insights</span>
          </div>
          <div className="p-4 rounded-xl bg-black/60 font-mono text-[10px] text-white/40 leading-relaxed border border-white/5">
            {`> node_${node.id.slice(-4)}: initialized`}<br />
            {`> status: idle`}<br />
            {`> packets_sent: 0`}
          </div>
        </section>
      </div>

      <div className="p-6 bg-black/20 backdrop-blur-md border-t border-white/5 mt-auto">
        <button className="w-full py-3 bg-[#9DCDE8] text-black font-bold text-sm rounded-xl hover:bg-white transition-all active:scale-95 shadow-lg">
          Apply Changes
        </button>
      </div>
    </aside>
  );
};
