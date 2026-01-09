import { useEffect, useRef, useCallback } from 'react';
import { useConnectionStore } from '../store/connectionStore';
import { useErrorHandler } from './useErrorHandler';

export interface WebSocketManagerConfig {
  url: string;
  protocols?: string | string[];
  maxReconnectAttempts?: number;
  initialReconnectDelay?: number;
  maxReconnectDelay?: number;
  reconnectDecay?: number;
  timeoutInterval?: number;
  enableLogging?: boolean;
}

const DEFAULT_CONFIG: Required<Omit<WebSocketManagerConfig, 'url' | 'protocols'>> = {
  maxReconnectAttempts: 10,
  initialReconnectDelay: 1000, // 1 second
  maxReconnectDelay: 30000, // 30 seconds
  reconnectDecay: 1.5, // Exponential backoff multiplier
  timeoutInterval: 5000, // 5 seconds
  enableLogging: true,
};

export const useWebSocketManager = (config: WebSocketManagerConfig) => {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const pingTimeoutRef = useRef<number | null>(null);
  const isManuallyClosedRef = useRef(false);
  
  const {
    connectionStatus,
    connectionInfo,
    setConnectionStatus,
    incrementReconnectAttempts,
    resetReconnectAttempts,
    updateLatency,
  } = useConnectionStore();
  
  const { handleWebSocketError, clearError } = useErrorHandler();
  
  const fullConfig = { ...DEFAULT_CONFIG, ...config };
  
  const log = useCallback((message: string, data?: any) => {
    if (fullConfig.enableLogging) {
      console.log(`[WebSocketManager] ${message}`, data || '');
    }
  }, [fullConfig.enableLogging]);

  const calculateReconnectDelay = useCallback(() => {
    const { reconnectAttempts } = connectionInfo;
    const delay = Math.min(
      fullConfig.initialReconnectDelay * Math.pow(fullConfig.reconnectDecay, reconnectAttempts),
      fullConfig.maxReconnectDelay
    );
    
    // Add jitter to prevent thundering herd
    const jitter = delay * 0.1 * Math.random();
    return Math.floor(delay + jitter);
  }, [connectionInfo.reconnectAttempts, fullConfig]);

  const clearTimeouts = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    
    if (pingTimeoutRef.current) {
      clearTimeout(pingTimeoutRef.current);
      pingTimeoutRef.current = null;
    }
  }, []);

  const sendPing = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const pingStart = Date.now();
      
      // Send ping frame (if supported) or a ping message
      try {
        wsRef.current.send(JSON.stringify({ type: 'ping', timestamp: pingStart }));
        
        // Set timeout for pong response
        pingTimeoutRef.current = setTimeout(() => {
          log('Ping timeout - connection may be stale');
          // Don't close connection immediately, just log the issue
        }, fullConfig.timeoutInterval);
        
      } catch (error) {
        log('Failed to send ping:', error);
      }
    }
  }, [fullConfig.timeoutInterval, log]);

  const handlePong = useCallback((timestamp: number) => {
    const latency = Date.now() - timestamp;
    updateLatency(latency);
    
    if (pingTimeoutRef.current) {
      clearTimeout(pingTimeoutRef.current);
      pingTimeoutRef.current = null;
    }
    
    log(`Pong received - latency: ${latency}ms`);
  }, [updateLatency, log]);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.CONNECTING || 
        wsRef.current?.readyState === WebSocket.OPEN) {
      log('WebSocket already connecting or connected');
      return;
    }

    log(`Attempting to connect to ${fullConfig.url}`);
    setConnectionStatus({ websocket: 'reconnecting' });

    try {
      const ws = new WebSocket(fullConfig.url, fullConfig.protocols);
      wsRef.current = ws;

      ws.onopen = () => {
        log('WebSocket connected successfully');
        setConnectionStatus({ websocket: 'connected' });
        resetReconnectAttempts();
        clearError('websocket');
        isManuallyClosedRef.current = false;
        
        // Start ping/pong heartbeat
        sendPing();
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          
          // Handle pong responses
          if (data.type === 'pong' && data.timestamp) {
            handlePong(data.timestamp);
            return;
          }
          
          // Handle ping requests (respond with pong)
          if (data.type === 'ping' && data.timestamp) {
            ws.send(JSON.stringify({ type: 'pong', timestamp: data.timestamp }));
            return;
          }
          
          // Regular message handling would go here
          // This is handled by GraphQL WS Link in the actual implementation
          
        } catch (error) {
          // Non-JSON messages are handled by GraphQL WS Link
          log('Received non-JSON message (likely GraphQL)');
        }
      };

      ws.onerror = (event) => {
        log('WebSocket error occurred:', event);
        handleWebSocketError(
          new Error('WebSocket connection error'),
          'Connection failed'
        );
        setConnectionStatus({ websocket: 'disconnected' });
      };

      ws.onclose = (event) => {
        log('WebSocket connection closed:', {
          code: event.code,
          reason: event.reason,
          wasClean: event.wasClean,
        });
        
        setConnectionStatus({ websocket: 'disconnected' });
        clearTimeouts();
        
        // Only attempt reconnection if not manually closed and within retry limits
        if (!isManuallyClosedRef.current && 
            connectionInfo.reconnectAttempts < fullConfig.maxReconnectAttempts) {
          
          const delay = calculateReconnectDelay();
          log(`Scheduling reconnection attempt ${connectionInfo.reconnectAttempts + 1} in ${delay}ms`);
          
          incrementReconnectAttempts();
          
          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, delay);
        } else if (connectionInfo.reconnectAttempts >= fullConfig.maxReconnectAttempts) {
          log('Max reconnection attempts reached');
          handleWebSocketError(
            new Error('Maximum reconnection attempts exceeded'),
            'Connection failed permanently'
          );
        }
      };

    } catch (error) {
      log('Failed to create WebSocket:', error);
      setConnectionStatus({ websocket: 'disconnected' });
      handleWebSocketError(error as Error, 'Failed to create connection');
    }
  }, [
    fullConfig,
    setConnectionStatus,
    resetReconnectAttempts,
    clearError,
    sendPing,
    handlePong,
    handleWebSocketError,
    clearTimeouts,
    connectionInfo.reconnectAttempts,
    calculateReconnectDelay,
    incrementReconnectAttempts,
    log,
  ]);

  const disconnect = useCallback(() => {
    log('Manually disconnecting WebSocket');
    isManuallyClosedRef.current = true;
    clearTimeouts();
    
    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }
    
    setConnectionStatus({ websocket: 'disconnected' });
  }, [clearTimeouts, setConnectionStatus, log]);

  const reconnect = useCallback(() => {
    log('Manual reconnection requested');
    disconnect();
    
    // Reset manual close flag and attempt counts
    isManuallyClosedRef.current = false;
    resetReconnectAttempts();
    
    // Connect after a short delay
    setTimeout(() => {
      connect();
    }, 100);
  }, [disconnect, connect, resetReconnectAttempts, log]);

  const getConnectionInfo = useCallback(() => {
    return {
      readyState: wsRef.current?.readyState,
      url: wsRef.current?.url,
      protocol: wsRef.current?.protocol,
      extensions: wsRef.current?.extensions,
      bufferedAmount: wsRef.current?.bufferedAmount,
      connectionStatus: connectionStatus.websocket,
      reconnectAttempts: connectionInfo.reconnectAttempts,
      latency: connectionInfo.latency,
    };
  }, [connectionStatus.websocket, connectionInfo]);

  // Auto-connect on mount
  useEffect(() => {
    connect();
    
    // Cleanup on unmount
    return () => {
      isManuallyClosedRef.current = true;
      clearTimeouts();
      if (wsRef.current) {
        wsRef.current.close(1000, 'Component unmounting');
      }
    };
  }, [connect, clearTimeouts]);

  // Periodic ping for connection health
  useEffect(() => {
    if (connectionStatus.websocket === 'connected') {
      const pingInterval = setInterval(() => {
        sendPing();
      }, fullConfig.timeoutInterval);
      
      return () => clearInterval(pingInterval);
    }
  }, [connectionStatus.websocket, sendPing, fullConfig.timeoutInterval]);

  return {
    connect,
    disconnect,
    reconnect,
    getConnectionInfo,
    isConnected: connectionStatus.websocket === 'connected',
    isConnecting: connectionStatus.websocket === 'reconnecting',
    reconnectAttempts: connectionInfo.reconnectAttempts,
    latency: connectionInfo.latency,
  };
};