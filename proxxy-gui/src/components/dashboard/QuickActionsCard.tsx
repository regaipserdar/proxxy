import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { 
  Settings, Activity, Users, BarChart3, Play, Pause, 
  RotateCcw, Server, Database, Wifi,
  CheckCircle, XCircle, Clock
} from 'lucide-react';

interface QuickActionsCardProps {
  isOrchestratorOnline?: boolean;
  onSystemAction?: (action: string) => Promise<void>;
}

export const QuickActionsCard: React.FC<QuickActionsCardProps> = ({ 
  isOrchestratorOnline = false,
  onSystemAction 
}) => {
  const navigate = useNavigate();
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const navigationActions = [
    {
      id: 'view-agents',
      label: 'View Agents',
      icon: Users,
      color: 'text-blue-400',
      bgColor: 'bg-blue-500/10',
      borderColor: 'border-blue-500/20',
      onClick: () => navigate('/agents'),
    },
    {
      id: 'view-traffic',
      label: 'View Traffic',
      icon: Activity,
      color: 'text-emerald-400',
      bgColor: 'bg-emerald-500/10',
      borderColor: 'border-emerald-500/20',
      onClick: () => navigate('/proxy'),
    },
    {
      id: 'view-metrics',
      label: 'View Metrics',
      icon: BarChart3,
      color: 'text-purple-400',
      bgColor: 'bg-purple-500/10',
      borderColor: 'border-purple-500/20',
      onClick: () => navigate('/metrics'),
    },
    {
      id: 'settings',
      label: 'Settings',
      icon: Settings,
      color: 'text-gray-400',
      bgColor: 'bg-gray-500/10',
      borderColor: 'border-gray-500/20',
      onClick: () => navigate('/settings'),
    },
  ];

  const systemActions = [
    {
      id: 'start',
      label: 'Start System',
      icon: Play,
      color: 'text-emerald-400',
      bgColor: 'bg-emerald-500/10',
      borderColor: 'border-emerald-500/20',
      disabled: isOrchestratorOnline,
    },
    {
      id: 'stop',
      label: 'Stop System',
      icon: Pause,
      color: 'text-red-400',
      bgColor: 'bg-red-500/10',
      borderColor: 'border-red-500/20',
      disabled: !isOrchestratorOnline,
    },
    {
      id: 'restart',
      label: 'Restart',
      icon: RotateCcw,
      color: 'text-amber-400',
      bgColor: 'bg-amber-500/10',
      borderColor: 'border-amber-500/20',
      disabled: false,
    },
  ];

  const handleSystemAction = async (actionId: string) => {
    if (!onSystemAction) return;
    
    setActionLoading(actionId);
    try {
      await onSystemAction(actionId);
    } catch (error) {
      console.error(`Failed to ${actionId} system:`, error);
    } finally {
      setActionLoading(null);
    }
  };

  const getStatusIcon = () => {
    if (isOrchestratorOnline) {
      return <CheckCircle className="h-6 w-6 text-emerald-400" />;
    }
    return <XCircle className="h-6 w-6 text-red-400" />;
  };

  const getStatusColor = () => {
    if (isOrchestratorOnline) {
      return 'text-emerald-400';
    }
    return 'text-red-400';
  };

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-6 hover:border-white/10 transition-all group">
      <div className="flex items-start justify-between mb-4">
        <div className={`p-3 rounded-lg ${isOrchestratorOnline ? 'bg-emerald-500/10 border border-emerald-500/20' : 'bg-red-500/10 border border-red-500/20'}`}>
          {getStatusIcon()}
        </div>
        <div className={`px-2 py-1 rounded-full bg-white/5 text-xs font-bold ${getStatusColor()} uppercase tracking-wider`}>
          {isOrchestratorOnline ? 'Online' : 'Offline'}
        </div>
      </div>

      <div className="space-y-4">
        {/* System Status */}
        <div>
          <h3 className="text-xs font-bold text-white/40 uppercase tracking-wider">System Status</h3>
          <div className="text-lg font-bold text-white mt-1 font-mono">
            {isOrchestratorOnline ? 'Operational' : 'Disconnected'}
          </div>
          <div className="text-xs text-white/60 mt-1">
            Orchestrator: {isOrchestratorOnline ? 'Connected' : 'Offline'}
          </div>
        </div>

        {/* Navigation Actions */}
        <div>
          <div className="text-xs text-white/40 mb-2 uppercase tracking-wider">Navigation</div>
          <div className="space-y-2">
            {navigationActions.map((action) => {
              const Icon = action.icon;
              return (
                <button
                  key={action.id}
                  onClick={action.onClick}
                  className={`w-full flex items-center gap-3 p-2 rounded-lg ${action.bgColor} ${action.borderColor} border hover:bg-white/5 transition-all group/action`}
                >
                  <Icon className={`h-4 w-4 ${action.color}`} />
                  <span className="text-sm text-white/80 group-hover/action:text-white transition-colors">
                    {action.label}
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        {/* System Controls */}
        <div className="pt-2 border-t border-white/5">
          <div className="text-xs text-white/40 mb-2 uppercase tracking-wider">System Controls</div>
          <div className="space-y-2">
            {systemActions.map((action) => {
              const Icon = action.icon;
              const isLoading = actionLoading === action.id;
              const isDisabled = action.disabled && !isLoading;
              
              return (
                <button
                  key={action.id}
                  onClick={() => handleSystemAction(action.id)}
                  disabled={isDisabled}
                  className={`w-full flex items-center justify-center gap-2 p-2 rounded-lg ${action.bgColor} ${action.borderColor} border transition-all ${
                    isDisabled 
                      ? 'opacity-50 cursor-not-allowed' 
                      : 'hover:bg-white/5 hover:border-white/10'
                  }`}
                >
                  {isLoading ? (
                    <Clock className="h-4 w-4 text-white/60 animate-spin" />
                  ) : (
                    <Icon className={`h-4 w-4 ${isDisabled ? 'text-white/30' : action.color}`} />
                  )}
                  <span className={`text-sm font-bold ${isDisabled ? 'text-white/30' : action.color}`}>
                    {isLoading ? 'Processing...' : action.label}
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Service Status Indicators */}
        <div className="pt-2 border-t border-white/5">
          <div className="text-xs text-white/40 mb-2 uppercase tracking-wider">Services</div>
          <div className="space-y-1">
            <div className="flex items-center justify-between text-xs">
              <div className="flex items-center gap-2">
                <Server className="h-3 w-3 text-white/40" />
                <span className="text-white/60">Orchestrator</span>
              </div>
              <span className={`font-bold ${isOrchestratorOnline ? 'text-emerald-400' : 'text-red-400'}`}>
                {isOrchestratorOnline ? 'UP' : 'DOWN'}
              </span>
            </div>
            <div className="flex items-center justify-between text-xs">
              <div className="flex items-center gap-2">
                <Database className="h-3 w-3 text-white/40" />
                <span className="text-white/60">Database</span>
              </div>
              <span className={`font-bold ${isOrchestratorOnline ? 'text-emerald-400' : 'text-amber-400'}`}>
                {isOrchestratorOnline ? 'UP' : 'UNKNOWN'}
              </span>
            </div>
            <div className="flex items-center justify-between text-xs">
              <div className="flex items-center gap-2">
                <Wifi className="h-3 w-3 text-white/40" />
                <span className="text-white/60">WebSocket</span>
              </div>
              <span className={`font-bold ${isOrchestratorOnline ? 'text-emerald-400' : 'text-red-400'}`}>
                {isOrchestratorOnline ? 'CONNECTED' : 'DISCONNECTED'}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};