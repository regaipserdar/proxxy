import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface AppState {
  // UI State
  sidebarCollapsed: boolean;
  currentPage: string;
  
  // Filters
  trafficFilters: {
    method?: string;
    statusCode?: number;
    agentId?: string;
    timeRange?: [Date, Date];
    searchQuery?: string;
  };
  
  // User Preferences
  preferences: {
    refreshInterval: number;
    theme: 'light' | 'dark';
    chartTimeRange: '1h' | '6h' | '24h';
    itemsPerPage: number;
  };
  
  // Connection Status
  connectionStatus: {
    graphql: 'connected' | 'disconnected' | 'reconnecting';
    websocket: 'connected' | 'disconnected' | 'reconnecting';
  };
  
  // Actions
  setSidebarCollapsed: (collapsed: boolean) => void;
  setCurrentPage: (page: string) => void;
  setTrafficFilters: (filters: Partial<AppState['trafficFilters']>) => void;
  setPreferences: (preferences: Partial<AppState['preferences']>) => void;
  setConnectionStatus: (status: Partial<AppState['connectionStatus']>) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      currentPage: 'dashboard',
      trafficFilters: {},
      preferences: {
        refreshInterval: 5000,
        theme: 'light',
        chartTimeRange: '1h',
        itemsPerPage: 50,
      },
      connectionStatus: {
        graphql: 'disconnected',
        websocket: 'disconnected',
      },
      
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      setCurrentPage: (page) => set({ currentPage: page }),
      setTrafficFilters: (filters) => 
        set((state) => ({ 
          trafficFilters: { ...state.trafficFilters, ...filters } 
        })),
      setPreferences: (preferences) =>
        set((state) => ({
          preferences: { ...state.preferences, ...preferences }
        })),
      setConnectionStatus: (status) =>
        set((state) => ({
          connectionStatus: { ...state.connectionStatus, ...status }
        })),
    }),
    {
      name: 'proxxy-gui-storage',
      partialize: (state) => ({
        preferences: state.preferences,
        trafficFilters: state.trafficFilters,
        sidebarCollapsed: state.sidebarCollapsed,
      }),
    }
  )
);