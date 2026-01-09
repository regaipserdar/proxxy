import React, { useState, useEffect } from 'react';
import { Cpu, Database, Zap, MemoryStick } from 'lucide-react';
import { SystemMetrics } from '../../types/graphql';

interface SystemHealthCardProps {
  isOnline?: boolean;
}

// Mock data for development
const mockMetrics: SystemMetrics = {
  agentId: 'orchestrator',
  timestamp: Date.now() / 1000,
  cpuUsagePercent: 45.2,
  memoryUsedBytes: '2147483648', // 2GB
  memoryTotalBytes: '8589934592', // 8GB
  networkRxBytesPerSec: '1048576', // 1MB/s
  networkTxBytesPerSec: '524288', // 512KB/s
  diskReadBytesPerSec: '2097152', // 2MB/s
  diskWriteBytesPerSec: '1048576', // 1MB/s
  processCpuPercent: 12.5,
  processMemoryBytes: '536870912', // 512MB
  processUptimeSeconds: 86400, // 24 hours
};

export const SystemHealthCard: React.FC<SystemHealthCardProps> = ({ isOnline = false }) => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);

  // Simulate GraphQL query
  useEffect(() => {
    const fetchMetrics = async () => {
      try {
        setLoading(true);
        
        if (!isOnline) {
          setMetrics(null);
          setError(new Error('Orchestrator offline'));
          return;
        }
        
        // Simulate network delay
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Simulate some variation in metrics
        const variationFactor = 0.8 + Math.random() * 0.4; // 0.8 to 1.2
        setMetrics({
          ...mockMetrics,
          cpuUsagePercent: mockMetrics.cpuUsagePercent * variationFactor,
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
    
    // Poll every 10 seconds
    const interval = setInterval(fetchMetrics, 10000);
    return () => clearInterval(interval);
  }, [isOnline]);

  const getHealthStatus = () => {
    if (!isOnline) return { status: 'Offline', color: 'text-red-400' };
    if (loading || error || !metrics) return { status: 'Unknown', color: 'text-gray-400' };
    
    const cpuUsage = metrics.cpuUsagePercent || 0;
    const memoryUsage = metrics.memoryUsedBytes && metrics.memoryTotalBytes 
      ? (parseInt(metrics.memoryUsedBytes) / parseInt(metrics.memoryTotalBytes)) * 100 
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
                    <span className={`text-xs font-bold ${getHealthColor((parseInt(metrics.memoryUsedBytes) / parseInt(metrics.memoryTotalBytes)) * 100)}`}>
                      {((parseInt(metrics.memoryUsedBytes) / parseInt(metrics.memoryTotalBytes)) * 100).toFixed(1)}%
                    </span>
                  </div>
                  <div className="w-full h-1.5 bg-white/5 rounded-full overflow-hidden">
                    <div
                      className={`h-full bg-gradient-to-r ${getHealthBg((parseInt(metrics.memoryUsedBytes) / parseInt(metrics.memoryTotalBytes)) * 100)} transition-all duration-1000 rounded-full`}
                      style={{ 
                        width: `${Math.min((parseInt(metrics.memoryUsedBytes) / parseInt(metrics.memoryTotalBytes)) * 100, 100)}%` 
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