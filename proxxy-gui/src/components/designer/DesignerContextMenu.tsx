
import { motion, AnimatePresence } from 'framer-motion';
import { Info, Copy, Trash2, Plus } from 'lucide-react';

export interface MenuConfig {
    x: number;
    y: number;
    type: 'pane' | 'node';
}

interface DesignerContextMenuProps {
    menuConfig: MenuConfig | null;
    onClose: () => void;
    onInspectNode: () => void;
    onDeleteNode: () => void;
    onDuplicateNode: () => void;
    onAddTrigger: () => void;
    onAddMatcher: () => void;
}

export const DesignerContextMenu = ({
    menuConfig,
    onClose,
    onInspectNode,
    onDeleteNode,
    onDuplicateNode,
    onAddTrigger,
    onAddMatcher
}: DesignerContextMenuProps) => {
    return (
        <AnimatePresence>
            {menuConfig && (
                <motion.div
                    initial={{ opacity: 0, scale: 0.95, y: -10 }}
                    animate={{ opacity: 1, scale: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.95 }}
                    className="absolute z-[200] w-[200px] bg-[#17181C] border border-white/10 rounded-2xl p-2 shadow-2xl dotted-bg"
                    style={{ left: menuConfig.x, top: menuConfig.y }}
                    onMouseLeave={onClose}
                >
                    {menuConfig.type === 'node' ? (
                        <>
                            <div className="px-3 py-2 border-b border-white/5 mb-1 text-[10px] font-bold uppercase tracking-widest text-white/30">Node Actions</div>
                            <MenuAction icon={<Info size={14} />} label="Inspect Node" onClick={onInspectNode} />
                            <MenuAction icon={<Copy size={14} />} label="Duplicate" onClick={onDuplicateNode} />
                            <MenuAction icon={<Trash2 size={14} />} label="Delete" color="text-red-400/70" onClick={onDeleteNode} />
                        </>
                    ) : (
                        <>
                            <div className="px-3 py-2 border-b border-white/5 mb-1 text-[10px] font-bold uppercase tracking-widest text-white/30">Workspace</div>
                            <MenuAction icon={<Plus size={14} />} label="Add Trigger" onClick={onAddTrigger} />
                            <MenuAction icon={<Plus size={14} />} label="Add Matcher" onClick={onAddMatcher} />
                        </>
                    )}
                </motion.div>
            )}
        </AnimatePresence>
    );
};

const MenuAction = ({ icon, label, color, onClick }: { icon: React.ReactNode, label: string, color?: string, onClick?: () => void }) => (
    <button
        onClick={onClick}
        className={`flex items-center gap-3 w-full px-3 py-2.5 rounded-xl hover:bg-white/5 text-[12px] font-medium transition-all ${color || 'text-white/60 hover:text-white'}`}
    >
        {icon} <span>{label}</span>
    </button>
);
