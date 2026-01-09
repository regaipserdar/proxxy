import React from 'react';
import { LeftSidebar } from '@/components/Sidebars';
import { ReactFlowProvider } from 'reactflow';

interface MainLayoutProps {
    children: React.ReactNode;
}

export const MainLayout = ({ children }: MainLayoutProps) => {
    return (
        <div className="flex h-screen w-screen bg-[#17181C] text-white/80 overflow-hidden font-inter selection:bg-[#9DCDE8]/30 relative">
            {/* GLOBAL TAURI DRAG REGION (Non-blocking) */}
            <div
                data-tauri-drag-region
                className="absolute top-0 left-0 right-0 h-8 z-[100] pointer-events-none"
            >
                {/* 
                    "pointer-events-none" ensures clicks pass through to UI elements below,
                    BUT "data-tauri-drag-region" elements are still caught by Tauri for window dragging.
                */}
            </div>

            <LeftSidebar />

            <div className="flex-1 flex flex-col relative min-w-0 h-full overflow-hidden">
                <ReactFlowProvider>
                    <div className="flex-1 h-full overflow-y-auto">
                        {children}
                    </div>
                </ReactFlowProvider>
            </div>
        </div>
    );
};
