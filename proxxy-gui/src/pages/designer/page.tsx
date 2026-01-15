import React, { useState, useCallback, useRef } from 'react';
import ReactFlow, {
    addEdge, Background, Connection, Edge, applyNodeChanges, applyEdgeChanges,
    NodeChange, EdgeChange, useReactFlow, BackgroundVariant
} from 'reactflow';
import { motion, AnimatePresence } from 'framer-motion';

import { ProxxyNode, NodeType } from '@/types';
import { TriggerNode, MatcherNode, ModifierNode, RepeaterNode, SinkNode } from '@/components/Nodes';
import { RightSidebar } from '@/components/Sidebars';
import { PropertySidebar } from '@/components/PropertySidebar';
import { generateWorkflowCode } from '@/services/geminiService';
import { BottomPanel } from '@/components/Panels';

// Components
import { DesignerHeader } from '@/components/designer/DesignerHeader';
import { DesignerContextMenu, MenuConfig } from '@/components/designer/DesignerContextMenu';

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
    const [menuConfig, setMenuConfig] = useState<MenuConfig | null>(null);

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

    // Context Menu Handlers
    const handleInspectNode = () => {
        if (selectedNodeId) {
            // Already selected, just close menu
            setMenuConfig(null);
        }
    };

    const handleDuplicateNode = () => {
        // Logic to duplicate logic could be here, strict duplication of selectedNodeId
        setMenuConfig(null);
    };

    const handleDeleteNode = () => {
        setNodes(nds => nds.filter(n => n.id !== (selectedNodeId || '')));
        setMenuConfig(null);
    };

    const handleAddTrigger = () => {
        // Logic to add trigger at click position could be implemented
        setMenuConfig(null);
    };

    const handleAddMatcher = () => {
        // Logic to add matcher at click position
        setMenuConfig(null);
    };

    const selectedNode = nodes.find(n => n.id === selectedNodeId);

    // Context Menu Right Click Handler


    const onNodeContextMenu = useCallback((event: React.MouseEvent, node: any) => {
        event.preventDefault();
        setSelectedNodeId(node.id); // Select it too
        setMenuConfig({
            x: event.clientX,
            y: event.clientY,
            type: 'node'
        });
    }, []);

    const onPaneContextMenu = useCallback((event: React.MouseEvent) => {
        event.preventDefault();
        setSelectedNodeId(null);
        setMenuConfig({
            x: event.clientX,
            y: event.clientY,
            type: 'pane'
        });
    }, []);

    return (
        <div className="flex-1 flex overflow-hidden bg-[#17181C]">
            <div className="flex-1 flex flex-col relative min-w-0">
                <motion.div
                    initial={{ opacity: 0, scale: 0.98 }}
                    animate={{ opacity: 1, scale: 1 }}
                    className="flex-1 flex flex-col relative bg-[#17181C]"
                >
                    <DesignerHeader onRun={handleRun} />

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
                            onNodeContextMenu={onNodeContextMenu}
                            onPaneContextMenu={onPaneContextMenu}
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

                        <DesignerContextMenu
                            menuConfig={menuConfig}
                            onClose={() => setMenuConfig(null)}
                            onInspectNode={handleInspectNode}
                            onDuplicateNode={handleDuplicateNode}
                            onDeleteNode={handleDeleteNode}
                            onAddTrigger={handleAddTrigger}
                            onAddMatcher={handleAddMatcher}
                        />
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