import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { 
  useUIStore, 
  useFiltersStore, 
  usePreferencesStore, 
  useConnectionStore 
} from '../store';

// Mock localStorage for testing
const localStorageMock = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

describe('State Management Layer', () => {
  beforeEach(() => {
    localStorageMock.clear();
  });

  describe('UI Store', () => {
    it('should initialize with default values', () => {
      const { result } = renderHook(() => useUIStore());
      
      expect(result.current.sidebarCollapsed).toBe(false);
      expect(result.current.currentPage).toBe('dashboard');
      expect(result.current.activeTab).toBe('overview');
    });

    it('should update sidebar state', () => {
      const { result } = renderHook(() => useUIStore());
      
      act(() => {
        result.current.setSidebarCollapsed(true);
      });
      
      expect(result.current.sidebarCollapsed).toBe(true);
    });

    it('should update current page', () => {
      const { result } = renderHook(() => useUIStore());
      
      act(() => {
        result.current.setCurrentPage('traffic');
      });
      
      expect(result.current.currentPage).toBe('traffic');
    });

    it('should manage modal states', () => {
      const { result } = renderHook(() => useUIStore());
      
      act(() => {
        result.current.setModal('settingsOpen', true);
      });
      
      expect(result.current.modals.settingsOpen).toBe(true);
      expect(result.current.modals.agentDetailsOpen).toBe(false);
    });

    it('should manage loading states', () => {
      const { result } = renderHook(() => useUIStore());
      
      act(() => {
        result.current.setLoading('agents', true);
      });
      
      expect(result.current.loading.agents).toBe(true);
      expect(result.current.loading.traffic).toBe(false);
    });
  });

  describe('Filters Store', () => {
    it('should initialize with empty filters', () => {
      const { result } = renderHook(() => useFiltersStore());
      
      expect(result.current.trafficFilters).toEqual({});
      expect(result.current.agentFilters).toEqual({});
      expect(result.current.metricsFilters.timeRange).toBe('1h');
    });

    it('should update traffic filters', () => {
      const { result } = renderHook(() => useFiltersStore());
      
      act(() => {
        result.current.setTrafficFilters({ method: 'GET', searchQuery: 'test' });
      });
      
      expect(result.current.trafficFilters.method).toBe('GET');
      expect(result.current.trafficFilters.searchQuery).toBe('test');
    });

    it('should update agent filters', () => {
      const { result } = renderHook(() => useFiltersStore());
      
      act(() => {
        result.current.setAgentFilters({ status: 'Online', hostname: 'localhost' });
      });
      
      expect(result.current.agentFilters.status).toBe('Online');
      expect(result.current.agentFilters.hostname).toBe('localhost');
    });

    it('should clear filters', () => {
      const { result } = renderHook(() => useFiltersStore());
      
      // Set some filters first
      act(() => {
        result.current.setTrafficFilters({ method: 'POST' });
        result.current.setAgentFilters({ status: 'Offline' });
      });
      
      // Clear traffic filters
      act(() => {
        result.current.clearTrafficFilters();
      });
      
      expect(result.current.trafficFilters).toEqual({});
      expect(result.current.agentFilters.status).toBe('Offline'); // Should remain
    });
  });

  describe('Preferences Store', () => {
    it('should initialize with default preferences', () => {
      const { result } = renderHook(() => usePreferencesStore());
      
      expect(result.current.preferences.theme).toBe('light');
      expect(result.current.preferences.refreshInterval).toBe(5000);
      expect(result.current.preferences.itemsPerPage).toBe(50);
      expect(result.current.apiConfig.graphqlEndpoint).toBe('http://localhost:9090/graphql');
    });

    it('should update preferences', () => {
      const { result } = renderHook(() => usePreferencesStore());
      
      act(() => {
        result.current.setPreferences({ theme: 'dark', itemsPerPage: 100 });
      });
      
      expect(result.current.preferences.theme).toBe('dark');
      expect(result.current.preferences.itemsPerPage).toBe(100);
      expect(result.current.preferences.refreshInterval).toBe(5000); // Should remain unchanged
    });

    it('should update API configuration', () => {
      const { result } = renderHook(() => usePreferencesStore());
      
      act(() => {
        result.current.setApiConfig({ 
          graphqlEndpoint: 'http://localhost:8080/graphql',
          timeout: 15000 
        });
      });
      
      expect(result.current.apiConfig.graphqlEndpoint).toBe('http://localhost:8080/graphql');
      expect(result.current.apiConfig.timeout).toBe(15000);
    });

    it('should reset preferences to defaults', () => {
      const { result } = renderHook(() => usePreferencesStore());
      
      // Change preferences first
      act(() => {
        result.current.setPreferences({ theme: 'dark', itemsPerPage: 200 });
      });
      
      // Reset to defaults
      act(() => {
        result.current.resetPreferences();
      });
      
      expect(result.current.preferences.theme).toBe('light');
      expect(result.current.preferences.itemsPerPage).toBe(50);
    });
  });

  describe('Connection Store', () => {
    it('should initialize with disconnected status', () => {
      const { result } = renderHook(() => useConnectionStore());
      
      expect(result.current.connectionStatus.graphql).toBe('disconnected');
      expect(result.current.connectionStatus.websocket).toBe('disconnected');
      expect(result.current.connectionInfo.reconnectAttempts).toBe(0);
    });

    it('should update connection status', () => {
      const { result } = renderHook(() => useConnectionStore());
      
      act(() => {
        result.current.setConnectionStatus({ graphql: 'connected' });
      });
      
      expect(result.current.connectionStatus.graphql).toBe('connected');
      expect(result.current.connectionStatus.websocket).toBe('disconnected');
    });

    it('should track reconnection attempts', () => {
      const { result } = renderHook(() => useConnectionStore());
      
      act(() => {
        result.current.incrementReconnectAttempts();
        result.current.incrementReconnectAttempts();
      });
      
      expect(result.current.connectionInfo.reconnectAttempts).toBe(2);
      
      act(() => {
        result.current.resetReconnectAttempts();
      });
      
      expect(result.current.connectionInfo.reconnectAttempts).toBe(0);
    });

    it('should manage error states', () => {
      const { result } = renderHook(() => useConnectionStore());
      
      act(() => {
        result.current.setErrors({ 
          graphql: 'Connection failed',
          network: 'Network timeout' 
        });
      });
      
      expect(result.current.errors.graphql).toBe('Connection failed');
      expect(result.current.errors.network).toBe('Network timeout');
      
      act(() => {
        result.current.clearErrors();
      });
      
      expect(result.current.errors).toEqual({});
    });

    it('should update latency', () => {
      const { result } = renderHook(() => useConnectionStore());
      
      act(() => {
        result.current.updateLatency(150);
      });
      
      expect(result.current.connectionInfo.latency).toBe(150);
    });
  });

  describe('Persistence', () => {
    it('should maintain UI state across store updates', () => {
      const { result } = renderHook(() => useUIStore());
      
      act(() => {
        result.current.setSidebarCollapsed(true);
        result.current.setCurrentPage('agents');
      });
      
      // Verify state is maintained in the store
      expect(result.current.sidebarCollapsed).toBe(true);
      expect(result.current.currentPage).toBe('agents');
    });

    it('should maintain filters state across store updates', () => {
      const { result } = renderHook(() => useFiltersStore());
      
      act(() => {
        result.current.setTrafficFilters({ method: 'POST', searchQuery: 'api' });
      });
      
      // Verify state is maintained in the store
      expect(result.current.trafficFilters.method).toBe('POST');
      expect(result.current.trafficFilters.searchQuery).toBe('api');
    });

    it('should maintain preferences state across store updates', () => {
      const { result } = renderHook(() => usePreferencesStore());
      
      act(() => {
        result.current.setPreferences({ theme: 'dark', itemsPerPage: 75 });
      });
      
      // Verify state is maintained in the store
      expect(result.current.preferences.theme).toBe('dark');
      expect(result.current.preferences.itemsPerPage).toBe(75);
    });
  });
});