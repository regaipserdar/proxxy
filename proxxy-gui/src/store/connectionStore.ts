import { create } from 'zustand';

export interface ConnectionState {
  // Connection status
  connectionStatus: {
    graphql: 'connected' | 'disconnected' | 'reconnecting';
    websocket: 'connected' | 'disconnected' | 'reconnecting';
  };
  
  // Connection metadata
  connectionInfo: {
    lastConnected?: Date;
    lastDisconnected?: Date;
    reconnectAttempts: number;
    latency?: number;
  };
  
  // Error tracking
  errors: {
    graphql?: string;
    websocket?: string;
    network?: string;
  };
  
  // Actions
  setConnectionStatus: (status: Partial<ConnectionState['connectionStatus']>) => void;
  setConnectionInfo: (info: Partial<ConnectionState['connectionInfo']>) => void;
  setErrors: (errors: Partial<ConnectionState['errors']>) => void;
  clearErrors: () => void;
  incrementReconnectAttempts: () => void;
  resetReconnectAttempts: () => void;
  updateLatency: (latency: number) => void;
}

export const useConnectionStore = create<ConnectionState>((set) => ({
  connectionStatus: {
    graphql: 'disconnected',
    websocket: 'disconnected',
  },
  
  connectionInfo: {
    reconnectAttempts: 0,
  },
  
  errors: {},
  
  setConnectionStatus: (status) =>
    set((state) => {
      const newStatus = { ...state.connectionStatus, ...status };
      const now = new Date();
      
      // Update connection timestamps
      let connectionInfo = { ...state.connectionInfo };
      
      Object.entries(status).forEach(([key, value]) => {
        if (value === 'connected' && state.connectionStatus[key as keyof typeof state.connectionStatus] !== 'connected') {
          connectionInfo.lastConnected = now;
          connectionInfo.reconnectAttempts = 0;
        } else if (value === 'disconnected' && state.connectionStatus[key as keyof typeof state.connectionStatus] === 'connected') {
          connectionInfo.lastDisconnected = now;
        }
      });
      
      return {
        connectionStatus: newStatus,
        connectionInfo,
      };
    }),
    
  setConnectionInfo: (info) =>
    set((state) => ({
      connectionInfo: { ...state.connectionInfo, ...info }
    })),
    
  setErrors: (errors) =>
    set((state) => ({
      errors: { ...state.errors, ...errors }
    })),
    
  clearErrors: () => set({ errors: {} }),
  
  incrementReconnectAttempts: () =>
    set((state) => ({
      connectionInfo: {
        ...state.connectionInfo,
        reconnectAttempts: state.connectionInfo.reconnectAttempts + 1,
      }
    })),
    
  resetReconnectAttempts: () =>
    set((state) => ({
      connectionInfo: {
        ...state.connectionInfo,
        reconnectAttempts: 0,
      }
    })),
    
  updateLatency: (latency) =>
    set((state) => ({
      connectionInfo: {
        ...state.connectionInfo,
        latency,
      }
    })),
}));