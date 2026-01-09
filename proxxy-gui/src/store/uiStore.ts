import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface UIState {
  // Layout state
  sidebarCollapsed: boolean;
  currentPage: string;
  activeTab: string;
  
  // Modal and dialog state
  modals: {
    settingsOpen: boolean;
    agentDetailsOpen: boolean;
    requestDetailsOpen: boolean;
  };
  
  // Loading states
  loading: {
    agents: boolean;
    traffic: boolean;
    metrics: boolean;
    replay: boolean;
  };
  
  // Actions
  setSidebarCollapsed: (collapsed: boolean) => void;
  setCurrentPage: (page: string) => void;
  setActiveTab: (tab: string) => void;
  setModal: (modal: keyof UIState['modals'], open: boolean) => void;
  setLoading: (key: keyof UIState['loading'], loading: boolean) => void;
  resetUI: () => void;
}

export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      currentPage: 'dashboard',
      activeTab: 'overview',
      
      modals: {
        settingsOpen: false,
        agentDetailsOpen: false,
        requestDetailsOpen: false,
      },
      
      loading: {
        agents: false,
        traffic: false,
        metrics: false,
        replay: false,
      },
      
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      setCurrentPage: (page) => set({ currentPage: page }),
      setActiveTab: (tab) => set({ activeTab: tab }),
      setModal: (modal, open) => 
        set((state) => ({
          modals: { ...state.modals, [modal]: open }
        })),
      setLoading: (key, loading) =>
        set((state) => ({
          loading: { ...state.loading, [key]: loading }
        })),
      resetUI: () => set({
        modals: {
          settingsOpen: false,
          agentDetailsOpen: false,
          requestDetailsOpen: false,
        },
        loading: {
          agents: false,
          traffic: false,
          metrics: false,
          replay: false,
        },
      }),
    }),
    {
      name: 'proxxy-ui-storage',
      partialize: (state) => ({
        sidebarCollapsed: state.sidebarCollapsed,
        currentPage: state.currentPage,
        activeTab: state.activeTab,
      }),
    }
  )
);