
import React, { useState, useCallback, useRef } from 'react';
import ReactFlow, {
    addEdge, Background, Connection, Edge, applyNodeChanges, applyEdgeChanges,
    NodeChange, EdgeChange, useReactFlow, BackgroundVariant
} from 'reactflow';
import { motion, AnimatePresence } from 'framer-motion';
import { Play, Info, Copy, Trash2, Plus } from 'lucide-react';

import { ProxxyNode, NodeType } from '@/types';
import { TriggerNode, MatcherNode, ModifierNode, RepeaterNode, SinkNode } from '@/components/Nodes';
import { RightSidebar } from '@/components/Sidebars';
import { BottomPanel } from '@/components/Panels';
import { PropertySidebar } from '@/components/PropertySidebar';
import { generateWorkflowCode } from '@/services/geminiService';

const nodeTypes = {
    trigger: TriggerNode,
    matcher: MatcherNode,
    modifier: ModifierNode,
    repeater: RepeaterNode,
    sink: SinkNode,
};

const initialNodes: ProxxyNode[] = [
    { id: 'p1', type: 'trigger', data: { label: 'Proxy Listener', type: NodeType.TRIGGER, config: {} }, position: { x: 100, y: 150 } },
    { id: 'm1', type: 'matcher', data: { label: 'Auth Filter', type: NodeType.MATCHER, config: { pattern: '/api/v2/auth' } }, position: { x: 350, y: 150 } },
    { id: 'r1', type: 'repeater', data: { label: 'Payload Fuzz', type: NodeType.REPEATER, config: { iterations: 100 } }, position: { x: 600, y: 150 } },
    { id: 's1', type: 'sink', data: { label: 'Output Logs', type: NodeType.SINK, config: {} }, position: { x: 850, y: 150 } },
];

const initialEdges: Edge[] = [
    { id: 'e1', source: 'p1', target: 'm1', animated: true, style: { stroke: '#9DCDE8' } },
    { id: 'e2', source: 'm1', target: 'r1', label: 'Match', style: { stroke: '#9DCDE8' } },
    { id: 'e3', source: 'r1', target: 's1', animated: true, style: { stroke: '#9DCDE8' } },
];

export const DesignerView = () => {
    const reactFlowWrapper = useRef<HTMLDivElement>(null);
    const [nodes, setNodes] = useState<ProxxyNode[]>(initialNodes);
    const [edges, setEdges] = useState<Edge[]>(initialEdges);
    const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
    const [isRunning, setIsRunning] = useState(false);
    const [code, setCode] = useState<string>(`// Proxxy Flow v1.0\n// Listening on 0.0.0.0:8080...`);
    const [menuConfig, setMenuConfig] = useState<{ x: number, y: number, type: 'pane' | 'node' } | null>(null);

    const { project } = useReactFlow();

    const onNodesChange = useCallback((changes: NodeChange[]) => setNodes((nds) => applyNodeChanges(changes, nds) as ProxxyNode[]), []);
    const onEdgesChange = useCallback((changes: EdgeChange[]) => setEdges((eds) => applyEdgeChanges(changes, eds)), []);
    const onConnect = useCallback((params: Connection) => setEdges((eds) => addEdge({ ...params, animated: true, style: { stroke: '#9DCDE8' } }, eds)), []);

    const handleRun = async () => {
        setIsRunning(true);
        const result = await generateWorkflowCode(nodes, edges);
        if (result) {
            setCode(result);
        }
        setTimeout(() => setIsRunning(false), 1200);
    };

    const onDrop = useCallback((event: React.DragEvent) => {
        event.preventDefault();
        const type = event.dataTransfer.getData('application/reactflow-type') as NodeType;
        const label = event.dataTransfer.getData('application/reactflow-label');
        const bounds = reactFlowWrapper.current?.getBoundingClientRect();
        if (type && bounds) {
            const position = project({ x: event.clientX - bounds.left, y: event.clientY - bounds.top });
            setNodes((nds) => nds.concat({
                id: `node_${Date.now()}`,
                type,
                position,
                data: { label, type, config: {}, subLabel: type.toUpperCase() }
            }));
        }
    }, [project]);

    const selectedNode = nodes.find(n => n.id === selectedNodeId);

    return (
        <div className="flex-1 flex overflow-hidden bg-[#17181C]">
            <div className="flex-1 flex flex-col relative min-w-0">
                <motion.div
                    initial={{ opacity: 0, scale: 0.98 }}
                    animate={{ opacity: 1, scale: 1 }}
                    className="flex-1 flex flex-col relative bg-[#17181C]"
                >
                    <header className="absolute top-6 left-6 right-[340px] flex items-center justify-between z-40 pointer-events-none">
                        <div className="flex items-center gap-3 bg-[#17181C]/80 backdrop-blur-xl border border-white/10 rounded-xl px-4 py-2 pointer-events-auto">
                            <span className="text-xs font-medium text-white/40 tracking-wide uppercase">Workspace: Default</span>
                        </div>
                        <div className="flex items-center gap-4 pointer-events-auto">
                            <button onClick={handleRun} className="p-3 bg-black/40 backdrop-blur-2xl border border-white/10 rounded-xl text-[#9DCDE8] hover:bg-white/5 transition-all shadow-2xl">
                                <Play size={18} fill="currentColor" />
                            </button>
                            <button className="bg-[#9DCDE8] text-black px-6 py-2.5 rounded-xl font-bold text-sm shadow-[0_0_30px_rgba(157,205,232,0.4)] transition-transform active:scale-95">
                                Export Flow
                            </button>
                        </div>
                    </header>

                    <main className="flex-1 relative w-full h-full" ref={reactFlowWrapper}>
                        <ReactFlow
                            nodes={nodes}
                            edges={edges}
                            onNodesChange={onNodesChange}
                            onEdgesChange={onEdgesChange}
                            onConnect={onConnect}
                            nodeTypes={nodeTypes}
                            onDrop={onDrop}
                            onDragOver={(e) => e.preventDefault()}
                            onNodeClick={(_, node) => setSelectedNodeId(node.id)}
                            onPaneClick={() => setSelectedNodeId(null)}
                            fitView
                            style={{ width: '100%', height: '100%' }}
                        >
                            <Background
                                color="#ffffff"
                                gap={20}
                                size={1}
                                variant={BackgroundVariant.Dots}
                                style={{ opacity: 0.05 }}
                            />
                        </ReactFlow>

                        <AnimatePresence>
                            {menuConfig && (
                                <motion.div
                                    initial={{ opacity: 0, scale: 0.95, y: -10 }}
                                    animate={{ opacity: 1, scale: 1, y: 0 }}
                                    exit={{ opacity: 0, scale: 0.95 }}
                                    className="absolute z-[200] w-[200px] bg-[#17181C] border border-white/10 rounded-2xl p-2 shadow-2xl dotted-bg"
                                    style={{ left: menuConfig.x, top: menuConfig.y }}
                                    onMouseLeave={() => setMenuConfig(null)}
                                >
                                    {menuConfig.type === 'node' ? (
                                        <>
                                            <div className="px-3 py-2 border-b border-white/5 mb-1 text-[10px] font-bold uppercase tracking-widest text-white/30">Node Actions</div>
                                            <MenuAction icon={<Info size={14} />} label="Inspect Node" onClick={() => setSelectedNodeId(selectedNodeId)} />
                                            <MenuAction icon={<Copy size={14} />} label="Duplicate" />
                                            <MenuAction icon={<Trash2 size={14} />} label="Delete" color="text-red-400/70" onClick={() => {
                                                setNodes(nds => nds.filter(n => n.id !== (selectedNodeId || '')));
                                                setMenuConfig(null);
                                            }} />
                                        </>
                                    ) : (
                                        <>
                                            <div className="px-3 py-2 border-b border-white/5 mb-1 text-[10px] font-bold uppercase tracking-widest text-white/30">Workspace</div>
                                            <MenuAction icon={<Plus size={14} />} label="Add Trigger" />
                                            <MenuAction icon={<Plus size={14} />} label="Add Matcher" />
                                        </>
                                    )}
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </main>
                    <BottomPanel code={code} isRunning={isRunning} />
                </motion.div>
            </div>

            <AnimatePresence mode="wait">
                {selectedNode ? (
                    <PropertySidebar
                        key="property-sidebar"
                        node={selectedNode}
                        onUpdate={(id, up) => setNodes(nds => nds.map(n => n.id === id ? { ...n, data: { ...n.data, ...up } } : n))}
                        onClose={() => setSelectedNodeId(null)}
                    />
                ) : (
                    <RightSidebar key="tool-sidebar" />
                )}
            </AnimatePresence>
        </div>
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
