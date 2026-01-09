import React from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useSubscription } from '@apollo/client';
import { BarChart3, ExternalLink, Cpu, Database, Network, HardDrive } from 'lucide-react';
import { GET_CURRENT_SYSTEM_METRICS, SYSTEM_METRICS_UPDATES } from '../../graphql/operations';

interface SystemMetricsOverviewProps {
  isOnline?: boolean;
  agentId?: string;
}

export const SystemMetricsOverview: React.FC<SystemMetricsOverviewProps> = ({
  isOnline = false,
  agentId = 'orchestrator' // Default to orchestrator metrics
}) => {
  const navigate = useNavigate();

  // ✅ REAL DATA: Query current metrics
  const { data, loading, error } = useQuery(GET_CURRENT_SYSTEM_METRICS, {
    variables: { agentId },
    skip: !isOnline,
    pollInterval: 15000, // Poll every 15 seconds
  });

  // ✅ REAL DATA: Subscribe to real-time updates
  const { data: subscriptionData } = useSubscription(SYSTEM_METRICS_UPDATES, {
    variables: { agentId },
    skip: !isOnline,
  });

  // Use subscription data if available, otherwise use query data
  const latestMetrics = subscriptionData?.systemMetricsUpdates || data?.currentSystemMetrics;

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

  const formatBytesPerSec = (bytesPerSec: string | undefined) => {
    if (!bytesPerSec) return '0 B/s';
    return formatBytes(bytesPerSec) + '/s';
  };

  const getUsageColor = (percentage: number) => {
    if (!isOnline) return 'text-gray-400';
    if (percentage > 80) return 'text-red-400';
    if (percentage > 60) return 'text-yellow-400';
    return 'text-emerald-400';
  };

  const getUsageBarColor = (percentage: number) => {
    if (!isOnline) return 'from-gray-500/20 to-gray-600/20';
    if (percentage > 80) return 'from-red-400 to-red-500';
    if (percentage > 60) return 'from-yellow-400 to-yellow-500';
    return 'from-emerald-400 to-emerald-500';
  };

  // ✅ REAL DATA: Calculate actual memory percentage
  const memoryUsagePercent = latestMetrics?.memoryUsedBytes && latestMetrics?.memoryTotalBytes
    ? (parseInt(latestMetrics.memoryUsedBytes, 10) / parseInt(latestMetrics.memoryTotalBytes, 10)) * 100
    : 0;

  const metrics = [
    {
      id: 'cpu',
      label: 'CPU Usage',
      icon: Cpu,
      value: latestMetrics?.cpuUsagePercent || 0,
      unit: '%',
      color: isOnline ? 'text-blue-400' : 'text-gray-400',
    },
    {
      id: 'memory',
      label: 'Memory Usage',
      icon: Database,
      value: memoryUsagePercent,
      unit: '%',
      color: isOnline ? 'text-emerald-400' : 'text-gray-400',
      detail: isOnline && latestMetrics
        ? `${formatBytes(latestMetrics.memoryUsedBytes)} / ${formatBytes(latestMetrics.memoryTotalBytes)}`
        : 'N/A',
    },
    {
      id: 'network-rx',
      label: 'Network RX',
      icon: Network,
      value: latestMetrics?.networkRxBytesPerSec ? parseInt(latestMetrics.networkRxBytesPerSec, 10) : 0,
      unit: '',
      color: isOnline ? 'text-purple-400' : 'text-gray-400',
      detail: isOnline ? formatBytesPerSec(latestMetrics?.networkRxBytesPerSec) : 'N/A',
      isBytes: true,
    },
    {
      id: 'disk-read',
      label: 'Disk Read',
      icon: HardDrive,
      value: latestMetrics?.diskReadBytesPerSec ? parseInt(latestMetrics.diskReadBytesPerSec, 10) : 0,
      unit: '',
      color: isOnline ? 'text-yellow-400' : 'text-gray-400',
      detail: isOnline ? formatBytesPerSec(latestMetrics?.diskReadBytesPerSec) : 'N/A',
      isBytes: true,
    },
  ];

  return (
    <div className="bg-[#111318] border border-white/5 rounded-xl p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <div className={`p-2 rounded-lg ${isOnline ? 'bg-purple-500/10 border border-purple-500/20' : 'bg-gray-500/10 border border-gray-500/20'}`}>
            <BarChart3 className={`h-5 w-5 ${isOnline ? 'text-purple-400' : 'text-gray-400'}`} />
          </div>
          <div>
            <h2 className="text-lg font-bold text-white">System Metrics</h2>
            <p className="text-xs text-white/60">
              {isOnline ? 'Real-time performance overview' : 'Metrics unavailable - orchestrator offline'}
            </p>
          </div>
        </div>
        <button
          onClick={() => navigate('/metrics')}
          disabled={!isOnline}
          className={`flex items-center gap-2 px-3 py-2 rounded-lg transition-all text-sm ${isOnline
              ? 'bg-white/5 hover:bg-white/10 text-white/80 hover:text-white'
              : 'bg-white/5 text-white/40 cursor-not-allowed'
            }`}
        >
          View Details
          <ExternalLink className="h-4 w-4" />
        </button>
      </div>

      {!isOnline ? (
        <div className="text-center py-12">
          <BarChart3 className="h-12 w-12 text-white/20 mx-auto mb-4" />
          <p className="text-white/40 text-sm">System metrics unavailable</p>
          <p className="text-white/20 text-xs mt-1">Connect to orchestrator to view real-time metrics</p>
        </div>
      ) : loading && !latestMetrics ? (
        <div className="text-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white/20 mx-auto mb-4"></div>
          <p className="text-white/40 text-sm">Loading metrics...</p>
        </div>
      ) : error && !latestMetrics ? (
        <div className="text-center py-12">
          <BarChart3 className="h-12 w-12 text-white/20 mx-auto mb-4" />
          <p className="text-white/40 text-sm">Metrics unavailable</p>
          <p className="text-white/20 text-xs mt-1">{error.message}</p>
        </div>
      ) : (
        <div className="space-y-4">
          {metrics.map((metric) => {
            const Icon = metric.icon;
            const percentage = metric.isBytes ? 0 : metric.value;

            return (
              <div key={metric.id} className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Icon className={`h-4 w-4 ${metric.color}`} />
                    <span className="text-sm text-white/80">{metric.label}</span>
                  </div>
                  <div className="text-right">
                    <span className={`text-sm font-bold ${getUsageColor(percentage)}`}>
                      {metric.isBytes ? metric.detail : `${metric.value.toFixed(1)}${metric.unit}`}
                    </span>
                    {metric.detail && !metric.isBytes && (
                      <p className="text-xs text-white/40">{metric.detail}</p>
                    )}
                  </div>
                </div>

                {!metric.isBytes && (
                  <div className="w-full h-2 bg-white/5 rounded-full overflow-hidden">
                    <div
                      className={`h-full bg-gradient-to-r ${getUsageBarColor(percentage)} transition-all duration-500`}
                      style={{ width: `${Math.min(percentage, 100)}%` }}
                    />
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* Footer */}
      {isOnline && latestMetrics && (
        <div className="mt-6 pt-4 border-t border-white/5 flex items-center justify-between">
          <div className="text-xs text-white/40">
            Last updated: {new Date(latestMetrics.timestamp * 1000).toLocaleTimeString()}
          </div>
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-purple-400 rounded-full animate-pulse"></div>
            <span className="text-xs text-white/60">Live metrics</span>
          </div>
        </div>
      )}

      {/* Offline Footer */}
      {!isOnline && (
        <div className="mt-6 pt-4 border-t border-white/5 flex items-center justify-center">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-red-400 rounded-full"></div>
            <span className="text-xs text-red-400">Orchestrator Disconnected</span>
          </div>
        </div>
      )}
    </div>
  );
};