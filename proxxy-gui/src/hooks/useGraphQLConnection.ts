import { useState, useEffect } from 'react';
import {
  initializeGraphQLConnection,
  subscribeToConnectionStatus,
  testGraphQLConnection,
  resetConnection
} from '../graphql/client';
import { useConnectionStore } from '../store/connectionStore';
import { useErrorHandler } from './useErrorHandler';

// Define ConnectionStatus interface locally
interface ConnectionStatus {
  graphql: 'connected' | 'disconnected' | 'reconnecting';
  websocket: 'connected' | 'disconnected' | 'reconnecting';
}

export const useGraphQLConnection = () => {
  const [isInitializing, setIsInitializing] = useState(true);
  const { connectionStatus, setConnectionStatus } = useConnectionStore();
  const { handleError, clearError } = useErrorHandler();

  // Test connection function
  const testConnection = async () => {
    try {
      setConnectionStatus({ graphql: 'reconnecting' });
      const isConnected = await testGraphQLConnection();

      if (isConnected) {
        clearError('graphql');
        clearError('network');
      }

      return isConnected;
    } catch (error) {
      handleError(error as Error, 'Connection test failed');
      return false;
    }
  };

  // Reset and reconnect function
  const reconnect = async () => {
    try {
      setConnectionStatus({ graphql: 'reconnecting' });
      await resetConnection();
      clearError('graphql');
      clearError('network');
      return true;
    } catch (error) {
      handleError(error as Error, 'Reconnection failed');
      return false;
    }
  };

  useEffect(() => {
    const initConnection = async () => {
      setIsInitializing(true);
      try {
        const status = await initializeGraphQLConnection();
        setConnectionStatus(status);

        if (status.graphql === 'connected') {
          clearError('graphql');
          clearError('network');
        }
      } catch (error) {
        console.error('Failed to initialize GraphQL connection:', error);
        handleError(error as Error, 'Failed to initialize connection');
        setConnectionStatus({
          graphql: 'disconnected',
          websocket: 'disconnected',
        });
      } finally {
        setIsInitializing(false);
      }
    };

    initConnection();
  }, [setConnectionStatus, handleError, clearError]);

  // Subscribe to connection status changes from the GraphQL client
  useEffect(() => {
    const unsubscribe = subscribeToConnectionStatus((status: Partial<ConnectionStatus>) => {
      setConnectionStatus(status);
    });

    return unsubscribe;
  }, [setConnectionStatus]);

  // Polling for reconnection if disconnected
  useEffect(() => {
    let reconnectTimer: ReturnType<typeof setInterval>;

    if (connectionStatus.graphql === 'disconnected' && !isInitializing) {
      reconnectTimer = setInterval(async () => {
        console.log('[Auto Reconnect] Attempting to reconnect...');
        const isConnected = await testConnection();
        if (isConnected) {
          // Connection restored!
          console.log('[Auto Reconnect] Connection restored!');
        }
      }, 5000); // Try every 5 seconds
    }

    return () => {
      if (reconnectTimer) clearInterval(reconnectTimer);
    };
  }, [connectionStatus.graphql, isInitializing]);

  return {
    connectionStatus,
    isInitializing,
    isConnected: connectionStatus.graphql === 'connected',
    isWebSocketConnected: connectionStatus.websocket === 'connected',
    testConnection,
    reconnect,
  };
};