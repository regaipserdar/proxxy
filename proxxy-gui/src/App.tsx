import { HashRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ProjectGuard } from '@/components/ProjectGuard';
import { MainLayout } from '@/layouts/MainLayout';

// Pages
import { ProjectLauncher } from '@/pages/projects/page';
import { DashboardPage } from '@/pages/dashboard/page';
import { AgentsView } from '@/pages/agents/page';
import { AgentDetailView } from '@/pages/agents/[agentId]/page';
import { DesignerView } from '@/pages/designer/page';
import { ScopeManager } from '@/pages/scope/page';
import { TrafficTreePage } from '@/pages/traffic-tree/page';
import { RepeaterView } from '@/pages/repeater/page';
import { IntruderView } from '@/pages/intruder/page';
import { FlowRecorderView } from '@/pages/flow-recorder/page';
import { SettingsView } from '@/pages/settings/page';

function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/projects" element={<ProjectLauncher />} />

        <Route path="/*" element={
          <ProjectGuard>
            <MainLayout>
              <Routes>
                <Route path="/" element={<DashboardPage />} />
                <Route path="/agents" element={<AgentsView />} />
                <Route path="/agents/:agentId" element={<AgentDetailView />} />
                <Route path="/designer" element={<DesignerView />} />
                <Route path="/scope" element={<ScopeManager />} />
                <Route path="/traffic-tree" element={<TrafficTreePage />} />
                <Route path="/repeater" element={<RepeaterView />} />
                <Route path="/intruder" element={<IntruderView />} />
                <Route path="/flow-recorder" element={<FlowRecorderView />} />
                <Route path="/settings" element={<SettingsView />} />
                <Route path="*" element={<Navigate to="/projects" replace />} />
              </Routes>
            </MainLayout>
          </ProjectGuard>
        } />
      </Routes>
    </HashRouter>
  );
}

export default App;
