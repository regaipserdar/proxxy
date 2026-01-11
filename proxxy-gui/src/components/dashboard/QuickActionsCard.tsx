import React, { useState } from 'react';
import {
  Play, Pause, RotateCcw,
  CheckCircle, XCircle, Loader2,
  Database, LogOut
} from 'lucide-react';
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";

interface QuickActionsCardProps {
  isOrchestratorOnline?: boolean;
  onSystemAction?: (action: string) => Promise<void>;
  activeProjectName?: string;
  onSwitchProject?: () => void;
}

export const QuickActionsCard: React.FC<QuickActionsCardProps> = ({
  isOrchestratorOnline = false,
  onSystemAction,
  activeProjectName,
  onSwitchProject
}) => {
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const systemActions = [
    {
      id: 'start',
      label: 'Start Core',
      icon: Play,
      color: 'text-emerald-400',
      disabled: isOrchestratorOnline,
    },
    {
      id: 'stop',
      label: 'Stop Core',
      icon: Pause,
      color: 'text-red-400',
      disabled: !isOrchestratorOnline,
    },
    {
      id: 'restart',
      label: 'Reset',
      icon: RotateCcw,
      color: 'text-amber-400',
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

  return (
    <Card className="bg-[#111318] border-white/5 hover:border-indigo-500/40 transition-all group overflow-hidden shadow-2xl h-full flex flex-col">
      <CardContent className="p-4 relative flex-1 flex flex-col gap-4">
        {/* Workspace Management Section (New) */}
        <div className="space-y-2 relative z-10">
          <div className="flex items-center justify-between">
            <span className="text-[10px] font-white  uppercase tracking-[0.2em]">Active Scope</span>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6 rounded-md hover:bg-destructive/10 hover:text-destructive group/switch"
              onClick={onSwitchProject}
              title="Switch Workspace"
            >
              <LogOut className="w-3 h-3 group-hover/switch:-translate-x-0.5 transition-transform" />
            </Button>
          </div>
          <Badge variant="outline" className="w-full h-9 justify-start gap-2.5 bg-white/[0.02] border-white/5 text-primary-foreground/90 font-mono text-xs overflow-hidden">
            <Database className="w-3.5 h-3.5 text-white shrink-0" />
            <span className="truncate text-orange-500">{activeProjectName || "No Project Loaded"}</span>
          </Badge>
        </div>

        <Separator className="bg-white/5" />

        {/* System Actions Section */}
        <div className="space-y-2 relative z-10">
          <div className="flex items-center justify-between mb-1">
            <div className="flex items-center gap-2">
              {isOrchestratorOnline ? <CheckCircle className="h-3.5 w-3.5 text-emerald-400" /> : <XCircle className="h-3.5 w-3.5 text-red-500" />}
              <span className={`text-[10px] font-bold uppercase tracking-widest ${isOrchestratorOnline ? 'text-emerald-500' : 'text-red-500'}`}>
                {isOrchestratorOnline ? 'Active' : 'Standby'}
              </span>
            </div>
            <Badge variant="outline" className="h-4 rounded-sm text-[7px] font-black bg-black/40 border-white/5 tracking-widest px-1">SYSTEM</Badge>
          </div>

          <div className="grid grid-cols-3 gap-2">
            {systemActions.map((action) => {
              const Icon = action.icon;
              const isLoading = actionLoading === action.id;
              const isDisabled = action.disabled && !isLoading;

              return (
                <Button
                  key={action.id}
                  variant="outline"
                  size="sm"
                  onClick={() => handleSystemAction(action.id)}
                  disabled={isDisabled}
                  className={`h-12 flex-col gap-1 rounded-xl border-white/5 bg-white/[0.01] hover:bg-white/[0.05] transition-all group/btn ${isDisabled ? 'opacity-30' : 'opacity-100'
                    }`}
                >
                  {isLoading ? (
                    <Loader2 className="h-3 w-3 animate-spin text-primary" />
                  ) : (
                    <Icon className={`h-3.5 w-3.5 ${isDisabled ? 'text-muted-foreground' : action.color} transition-transform group-hover/btn:scale-110`} />
                  )}
                  <span className="text-[8px] font-black uppercase tracking-tighter leading-none">
                    {action.id === 'restart' ? 'Reset' : action.id}
                  </span>
                </Button>
              );
            })}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};