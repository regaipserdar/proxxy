import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { BarChart3, ExternalLink, Cpu, Database, Network, HardDrive } from 'lucide-react';
import { SystemMetrics } from '../../types/graphql';

interface SystemMetricsOverviewProps {
  isOnline?: boolean;
}

// Mock data for development
const mockMetrics: SystemMetrics = {
  agentId: 'orchestrator',
  timestamp: Date.now() / 1000,
  cpuUsagePercent: 42.5,
  memoryUsedBytes: '3221225472', // 3GB
  memoryTotalBytes: '8589934592', // 8GB
  networkRxBytesPerSec: '2097152', // 2MB/s
  networkTxBytesPerSec: '1048576', // 1MB/s
  diskReadBytesPerSec: '4194304', // 4MB/s
  diskWriteBytesPerSec: '2097152', // 2MB/s
  processCpuPercent: 15.2,
  processMemoryBytes: '1073741824', // 1GB
  processUptimeSeconds: 172800, // 48 hours
};

export const SystemMetricsOverview: React.FC<SystemMetricsOverviewProps> = ({ isOnline = false }) => {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [latestMetrics, setLatestMetrics] = useState<SystemMetrics | null>(null);

  // Simulate GraphQL query
  useEffect(() => {
    const fetchMetrics = async () => {
      try {
        setLoading(true);
        
        if (!isOnline) {
          setLatestMetrics(null);
          setError(new Error('Orchestrator offline'));
          return;
        }
        
        // Simulate network delay
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Simulate some variation in metrics
        const variationFactor = 0.7 + Math.random() * 0.6; // 0.7 to 1.3
        setLatestMetrics({
          ...mockMetrics,
          cpuUsagePercent: mockMetrics.cpuUsagePercent * variationFactor,
          networkRxBytesPerSec: (parseInt(mockMetrics.networkRxBytesPerSec) * variationFactor).toString(),
          networkTxBytesPerSec: (parseInt(mockMetrics.networkTxBytesPerSec) * variationFactor).toString(),
          timestamp: Date.now() / 1000,
        });
        setError(null);
      } catch (err) {
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    fetchMetrics();
    
    // Poll every 15 seconds
    const interval = setInterval(fetchMetrics, 15000);
    return () => clearInterval(interval);
  }, [isOnline]);
  
  const formatBytes = (bytes: string | undefined) => {
    if (!bytes) return '0 B';
    const num = parseInt(bytes);
    const units = ['B', 'KB', 'MB', 'GB'];
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

  const metrics = [
    {
      id: 'cpu',
      label: 'CPU Usage',
      icon: Cpu,
      value: isOnline && latestMetrics?.cpuUsagePercent || 0,
      unit: '%',
      color: isOnline ? 'text-blue-400' : 'text-gray-400',
    },
    {
      id: 'memory',
      label: 'Memory Usage',
      icon: Database,
      value: isOnline && latestMetrics?.memoryUsedBytes && latestMetrics?.memoryTotalBytes 
        ? (parseInt(latestMetrics.memoryUsedBytes) / parseInt(latestMetrics.memoryTotalBytes)) * 100 
        : 0,
      unit: '%',
      color: isOnline ? 'text-emerald-400' : 'text-gray-400',
      detail: isOnline && latestMetrics ? `${formatBytes(latestMetrics.memoryUsedBytes)} / ${formatBytes(latestMetrics.memoryTotalBytes)}` : 'N/A',
    },
    {
      id: 'network-rx',
      label: 'Network RX',
      icon: Network,
      value: isOnline && latestMetrics?.networkRxBytesPerSec ? parseInt(latestMetrics.networkRxBytesPerSec) : 0,
      unit: '',
      color: isOnline ? 'text-purple-400' : 'text-gray-400',
      detail: isOnline ? formatBytesPerSec(latestMetrics?.networkRxBytesPerSec) : 'N/A',
      isBytes: true,
    },
    {
      id: 'disk-read',
      label: 'Disk Read',
      icon: HardDrive,
      value: isOnline && latestMetrics?.diskReadBytesPerSec ? parseInt(latestMetrics.diskReadBytesPerSec) : 0,
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
          className={`flex items-center gap-2 px-3 py-2 rounded-lg transition-all text-sm ${
            isOnline 
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
          <p className="text-white/20 text-xs mt-1">Check system connection</p>
        </div>
      ) : (
        <div className="space-y-4">
          {metrics.map((metric) => {
            const Icon = metric.icon;
            const percentage = metric.isBytes ? 0 : metric.value; // Don't show percentage for byte values
            
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