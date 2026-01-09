import React from 'react';
import { useQuery, useSubscription } from '@apollo/client';
import { Cpu, Database, Zap, MemoryStick } from 'lucide-react';
import { GET_CURRENT_SYSTEM_METRICS, SYSTEM_METRICS_UPDATES } from '../../graphql/operations';

interface SystemHealthCardProps {
  isOnline?: boolean;
  agentId?: string;
}

export const SystemHealthCard: React.FC<SystemHealthCardProps> = ({
  isOnline = false,
  agentId = 'orchestrator'
}) => {
  // ✅ REAL DATA: Query current metrics
  const { data, loading, error } = useQuery(GET_CURRENT_SYSTEM_METRICS, {
    variables: { agentId },
    skip: !isOnline,
    pollInterval: 10000, // Poll every 10 seconds
  });

  // ✅ REAL DATA: Subscribe to real-time updates
  const { data: subscriptionData } = useSubscription(SYSTEM_METRICS_UPDATES, {
    variables: { agentId },
    skip: !isOnline,
  });

  // Use subscription data if available, otherwise use query data
  const metrics = subscriptionData?.systemMetricsUpdates || data?.currentSystemMetrics;

  const getHealthStatus = () => {
    if (!isOnline) return { status: 'Offline', color: 'text-red-400' };
    if (loading || error || !metrics) return { status: 'Unknown', color: 'text-gray-400' };

    const cpuUsage = metrics.cpuUsagePercent || 0;
    const memoryUsage = metrics.memoryUsedBytes && metrics.memoryTotalBytes
      ? (parseInt(metrics.memoryUsedBytes, 10) / parseInt(metrics.memoryTotalBytes, 10)) * 100
      : 0;

    if (cpuUsage > 80 || memoryUsage > 90) {
      return { status: 'Critical', color: 'text-red-400' };
    } else if (cpuUsage > 60 || memoryUsage > 75) {
      return { status: 'Warning', color: 'text-yellow-400' };
    } else {
      return { status: 'Healthy', color: 'text-emerald-400' };
    }
  };

  const healthStatus = getHealthStatus();

  const formatBytes = (bytes: string | undefined) => {
    if (!bytes) return '0 B';
    const num = parseInt(bytes, 10);
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let unitIndex = 0;
    let value = num;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }

    return `${value.toFixed(1)} ${units[unitIndex]}`;
  };

  const getHealthColor = (value: number) => {
    if (!isOnline) return 'text-gray-400';
    if (value < 30) return 'text-emerald-400';
    if (value < 70) return 'text-amber-400';
    return 'text-red-400';
  };

  const getHealthBg = (value: number) => {
    if (!isOnline) return 'from-gray-500/20 to-gray-600/20';
    if (value < 30) return 'from-emerald-500/20 to-emerald-600/20';
    if (value < 70) return 'from-amber-500/20 to-amber-600/20';
    return 'from-red-500/20 to-red-600/20';
  };

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-6 hover:border-white/10 transition-all group">
      <div className="flex items-start justify-between mb-4">
        <div className={`p-3 rounded-lg ${isOnline ? 'bg-purple-500/10 border border-purple-500/20' : 'bg-gray-500/10 border border-gray-500/20'}`}>
          <Cpu className={`h-6 w-6 ${isOnline ? 'text-purple-400' : 'text-gray-400'}`} />
        </div>
        <div className={`px-2 py-1 rounded-full bg-white/5 text-xs font-bold ${healthStatus.color} uppercase tracking-wider`}>
          {healthStatus.status}
        </div>
      </div>

      <div className="space-y-3">
        <div>
          <h3 className="text-xs font-bold text-white/40 uppercase tracking-wider">System Health</h3>
          <div className="text-2xl font-bold text-white mt-1 font-mono">
            {!isOnline ? 'OFFLINE' : loading ? '...' : error ? 'N/A' : 'OK'}
          </div>
          <div className="text-xs text-white/60 mt-1">
            {!isOnline ? 'System unavailable' : 'Overall system performance'}
          </div>
        </div>

        {/* Quick Stats */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Cpu className="h-4 w-4 text-blue-400" />
              <span className="text-sm text-white/60">CPU Usage</span>
            </div>
            <span className="text-sm font-bold text-white">
              {!isOnline ? 'N/A' : loading ? '...' : error || !metrics ? 'N/A' : `${metrics.cpuUsagePercent?.toFixed(1) || 0}%`}
            </span>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Database className="h-4 w-4 text-emerald-400" />
              <span className="text-sm text-white/60">Memory</span>
            </div>
            <span className="text-sm font-bold text-white">
              {!isOnline ? 'N/A' : loading ? '...' : error || !metrics ? 'N/A' : formatBytes(metrics.memoryUsedBytes)}
            </span>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Zap className="h-4 w-4 text-yellow-400" />
              <span className="text-sm text-white/60">Uptime</span>
            </div>
            <span className="text-sm font-bold text-white">
              {!isOnline ? 'N/A' : loading ? '...' : error || !metrics ? 'N/A' :
                metrics?.processUptimeSeconds ?
                  `${Math.floor(metrics.processUptimeSeconds / 3600)}h ${Math.floor((metrics.processUptimeSeconds % 3600) / 60)}m` :
                  'N/A'
              }
            </span>
          </div>
        </div>

        {/* Detailed Health Indicators */}
        {isOnline && !loading && !error && metrics && (
          <div className="mt-4 pt-3 border-t border-white/5">
            <div className="text-xs text-white/40 mb-3 uppercase tracking-wider">Resource Usage</div>
            <div className="space-y-3">
              {/* CPU Usage */}
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Cpu className="h-3 w-3 text-white/40" />
                    <span className="text-xs text-white/60">CPU</span>
                  </div>
                  <span className={`text-xs font-bold ${getHealthColor(metrics.cpuUsagePercent || 0)}`}>
                    {metrics.cpuUsagePercent?.toFixed(1) || 0}%
                  </span>
                </div>
                <div className="w-full h-1.5 bg-white/5 rounded-full overflow-hidden">
                  <div
                    className={`h-full bg-gradient-to-r ${getHealthBg(metrics.cpuUsagePercent || 0)} transition-all duration-1000 rounded-full`}
                    style={{ width: `${Math.min(metrics.cpuUsagePercent || 0, 100)}%` }}
                  />
                </div>
              </div>

              {/* Memory Usage */}
              {metrics.memoryUsedBytes && metrics.memoryTotalBytes && (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <MemoryStick className="h-3 w-3 text-white/40" />
                      <span className="text-xs text-white/60">Memory</span>
                    </div>
                    <span className={`text-xs font-bold ${getHealthColor((parseInt(metrics.memoryUsedBytes, 10) / parseInt(metrics.memoryTotalBytes, 10)) * 100)}`}>
                      {((parseInt(metrics.memoryUsedBytes, 10) / parseInt(metrics.memoryTotalBytes, 10)) * 100).toFixed(1)}%
                    </span>
                  </div>
                  <div className="w-full h-1.5 bg-white/5 rounded-full overflow-hidden">
                    <div
                      className={`h-full bg-gradient-to-r ${getHealthBg((parseInt(metrics.memoryUsedBytes, 10) / parseInt(metrics.memoryTotalBytes, 10)) * 100)} transition-all duration-1000 rounded-full`}
                      style={{
                        width: `${Math.min((parseInt(metrics.memoryUsedBytes, 10) / parseInt(metrics.memoryTotalBytes, 10)) * 100, 100)}%`
                      }}
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Offline State */}
        {!isOnline && (
          <div className="mt-4 pt-3 border-t border-white/5">
            <div className="text-xs text-red-400/80 text-center">
              Connect to orchestrator to view system metrics
            </div>
          </div>
        )}
      </div>
    </div>
  );
};