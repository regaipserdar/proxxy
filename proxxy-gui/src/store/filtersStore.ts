import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface FiltersState {
  // Traffic filters
  trafficFilters: {
    method?: string;
    statusCode?: number;
    agentId?: string;
    timeRange?: [Date, Date];
    searchQuery?: string;
    urlPattern?: string;
  };
  
  // Agent filters
  agentFilters: {
    status?: 'Online' | 'Offline';
    searchQuery?: string;
    hostname?: string;
  };
  
  // Metrics filters
  metricsFilters: {
    agentId?: string;
    timeRange: '1h' | '6h' | '24h';
    metricType?: 'cpu' | 'memory' | 'network' | 'disk';
  };
  
  // Actions
  setTrafficFilters: (filters: Partial<FiltersState['trafficFilters']>) => void;
  setAgentFilters: (filters: Partial<FiltersState['agentFilters']>) => void;
  setMetricsFilters: (filters: Partial<FiltersState['metricsFilters']>) => void;
  clearTrafficFilters: () => void;
  clearAgentFilters: () => void;
  clearMetricsFilters: () => void;
  clearAllFilters: () => void;
}

export const useFiltersStore = create<FiltersState>()(
  persist(
    (set) => ({
      trafficFilters: {},
      agentFilters: {},
      metricsFilters: {
        timeRange: '1h',
      },
      
      setTrafficFilters: (filters) => 
        set((state) => ({ 
          trafficFilters: { ...state.trafficFilters, ...filters } 
        })),
      setAgentFilters: (filters) =>
        set((state) => ({
          agentFilters: { ...state.agentFilters, ...filters }
        })),
      setMetricsFilters: (filters) =>
        set((state) => ({
          metricsFilters: { ...state.metricsFilters, ...filters }
        })),
      clearTrafficFilters: () => set({ trafficFilters: {} }),
      clearAgentFilters: () => set({ agentFilters: {} }),
      clearMetricsFilters: () => set({ metricsFilters: { timeRange: '1h' } }),
      clearAllFilters: () => set({
        trafficFilters: {},
        agentFilters: {},
        metricsFilters: { timeRange: '1h' },
      }),
    }),
    {
      name: 'proxxy-filters-storage',
      partialize: (state) => ({
        trafficFilters: state.trafficFilters,
        agentFilters: state.agentFilters,
        metricsFilters: state.metricsFilters,
      }),
    }
  )
);