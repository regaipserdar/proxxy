import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface PreferencesState {
  // Application preferences
  preferences: {
    refreshInterval: number;
    theme: 'light' | 'dark';
    chartTimeRange: '1h' | '6h' | '24h';
    itemsPerPage: number;
    autoRefresh: boolean;
    soundNotifications: boolean;
    compactView: boolean;
  };
  
  // API configuration
  apiConfig: {
    graphqlEndpoint: string;
    websocketEndpoint: string;
    timeout: number;
    retryAttempts: number;
  };
  
  // Dashboard layout preferences
  dashboardLayout: {
    showAgentCards: boolean;
    showTrafficSummary: boolean;
    showSystemHealth: boolean;
    showQuickActions: boolean;
    cardOrder: string[];
  };
  
  // Actions
  setPreferences: (preferences: Partial<PreferencesState['preferences']>) => void;
  setApiConfig: (config: Partial<PreferencesState['apiConfig']>) => void;
  setDashboardLayout: (layout: Partial<PreferencesState['dashboardLayout']>) => void;
  resetPreferences: () => void;
  resetApiConfig: () => void;
  resetDashboardLayout: () => void;
}

const defaultPreferences: PreferencesState['preferences'] = {
  refreshInterval: 5000,
  theme: 'light',
  chartTimeRange: '1h',
  itemsPerPage: 50,
  autoRefresh: true,
  soundNotifications: false,
  compactView: false,
};

const defaultApiConfig: PreferencesState['apiConfig'] = {
  graphqlEndpoint: 'http://localhost:9090/graphql',
  websocketEndpoint: 'ws://localhost:9090/graphql',
  timeout: 10000,
  retryAttempts: 3,
};

const defaultDashboardLayout: PreferencesState['dashboardLayout'] = {
  showAgentCards: true,
  showTrafficSummary: true,
  showSystemHealth: true,
  showQuickActions: true,
  cardOrder: ['agents', 'traffic', 'health', 'actions'],
};

export const usePreferencesStore = create<PreferencesState>()(
  persist(
    (set) => ({
      preferences: defaultPreferences,
      apiConfig: defaultApiConfig,
      dashboardLayout: defaultDashboardLayout,
      
      setPreferences: (preferences) =>
        set((state) => ({
          preferences: { ...state.preferences, ...preferences }
        })),
      setApiConfig: (config) =>
        set((state) => ({
          apiConfig: { ...state.apiConfig, ...config }
        })),
      setDashboardLayout: (layout) =>
        set((state) => ({
          dashboardLayout: { ...state.dashboardLayout, ...layout }
        })),
      resetPreferences: () => set({ preferences: defaultPreferences }),
      resetApiConfig: () => set({ apiConfig: defaultApiConfig }),
      resetDashboardLayout: () => set({ dashboardLayout: defaultDashboardLayout }),
    }),
    {
      name: 'proxxy-preferences-storage',
      partialize: (state) => ({
        preferences: state.preferences,
        apiConfig: state.apiConfig,
        dashboardLayout: state.dashboardLayout,
      }),
    }
  )
);