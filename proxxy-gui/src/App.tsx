
import { HashRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ReactFlowProvider } from 'reactflow';
import { LeftSidebar } from './components/Sidebars';
import { HomeView } from './components/HomeView';
import { DesignerView } from './components/DesignerView';
import { SettingsView } from './components/SettingsView';
import { ProxyView } from './components/ProxyView';
import { RepeaterView } from './components/RepeaterView';
import { IntruderView } from './components/IntruderView';
import { ScopeManager } from './components/ScopeManager';
import { AgentsView } from './components/AgentsView';
import { AgentDetailView } from './components/AgentDetailView';

const App = () => {
  return (
    <Router>
      <div className="flex h-screen w-screen bg-[#17181C] text-white/80 overflow-hidden font-inter selection:bg-[#9DCDE8]/30">
        <LeftSidebar />

        <div className="flex-1 flex flex-col relative min-w-0 h-full overflow-hidden">
          <ReactFlowProvider>
            <div className="flex-1 h-full overflow-y-auto">
              <Routes>
                <Route path="/" element={<HomeView />} />
                <Route path="/agents" element={<AgentsView />} />
                <Route path="/agents/:agentId" element={<AgentDetailView />} />
                <Route path="/designer" element={<DesignerView />} />
                <Route path="/scope" element={<ScopeManager />} />
                <Route path="/proxy" element={<ProxyView />} />
                <Route path="/repeater" element={<RepeaterView />} />
                <Route path="/intruder" element={<IntruderView />} />
                <Route path="/settings" element={<SettingsView />} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </div>
          </ReactFlowProvider>
        </div>
      </div>
    </Router>
  );
};

export default App;
