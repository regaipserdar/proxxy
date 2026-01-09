import React from 'react';
import { useConnectionStore } from '../store/connectionStore';
import { Wifi, WifiOff, RotateCcw, AlertCircle } from 'lucide-react';

export interface ConnectionStatusIndicatorProps {
  showDetails?: boolean;
  className?: string;
}

export const ConnectionStatusIndicator: React.FC<ConnectionStatusIndicatorProps> = ({
  showDetails = false,
  className = '',
}) => {
  const { connectionStatus, connectionInfo, errors } = useConnectionStore();

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'connected':
        return 'text-green-500';
      case 'reconnecting':
        return 'text-yellow-500';
      case 'disconnected':
        return 'text-red-500';
      default:
        return 'text-gray-500';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'connected':
        return <Wifi className="h-4 w-4" />;
      case 'reconnecting':
        return <RotateCcw className="h-4 w-4 animate-spin" />;
      case 'disconnected':
        return <WifiOff className="h-4 w-4" />;
      default:
        return <AlertCircle className="h-4 w-4" />;
    }
  };

  const getStatusText = (type: 'graphql' | 'websocket', status: string) => {
    const prefix = type === 'graphql' ? 'API' : 'Real-time';
    
    switch (status) {
      case 'connected':
        return `${prefix}: Connected`;
      case 'reconnecting':
        return `${prefix}: Reconnecting...`;
      case 'disconnected':
        return `${prefix}: Disconnected`;
      default:
        return `${prefix}: Unknown`;
    }
  };

  const hasErrors = Object.values(errors).some(error => error);
  const isFullyConnected = connectionStatus.graphql === 'connected' && 
                          connectionStatus.websocket === 'connected';
  const isReconnecting = connectionStatus.graphql === 'reconnecting' || 
                        connectionStatus.websocket === 'reconnecting';

  return (
    <div className={`flex items-center space-x-2 ${className}`}>
      {/* Main status indicator */}
      <div className="flex items-center space-x-1">
        <div className={getStatusColor(isFullyConnected ? 'connected' : 
                                      isReconnecting ? 'reconnecting' : 'disconnected')}>
          {getStatusIcon(isFullyConnected ? 'connected' : 
                        isReconnecting ? 'reconnecting' : 'disconnected')}
        </div>
        
        {showDetails && (
          <div className="text-sm">
            <div className="flex flex-col space-y-1">
              {/* GraphQL Status */}
              <div className={`flex items-center space-x-1 ${getStatusColor(connectionStatus.graphql)}`}>
                {getStatusIcon(connectionStatus.graphql)}
                <span>{getStatusText('graphql', connectionStatus.graphql)}</span>
              </div>
              
              {/* WebSocket Status */}
              <div className={`flex items-center space-x-1 ${getStatusColor(connectionStatus.websocket)}`}>
                {getStatusIcon(connectionStatus.websocket)}
                <span>{getStatusText('websocket', connectionStatus.websocket)}</span>
              </div>
              
              {/* Connection Info */}
              {connectionInfo.reconnectAttempts > 0 && (
                <div className="text-xs text-gray-500">
                  Reconnect attempts: {connectionInfo.reconnectAttempts}
                </div>
              )}
              
              {connectionInfo.latency && (
                <div className="text-xs text-gray-500">
                  Latency: {connectionInfo.latency}ms
                </div>
              )}
              
              {connectionInfo.lastConnected && (
                <div className="text-xs text-gray-500">
                  Last connected: {connectionInfo.lastConnected.toLocaleTimeString()}
                </div>
              )}
            </div>
          </div>
        )}
      </div>
      
      {/* Error indicator */}
      {hasErrors && (
        <div className="text-red-500" title="Connection errors detected">
          <AlertCircle className="h-4 w-4" />
        </div>
      )}
    </div>
  );
};

export default ConnectionStatusIndicator;