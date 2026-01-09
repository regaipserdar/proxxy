import { useUIStore } from './uiStore';
import { useFiltersStore } from './filtersStore';
import { usePreferencesStore } from './preferencesStore';
import { useConnectionStore } from './connectionStore';

/**
 * Convenience hook that provides access to all stores
 * Use this when you need multiple stores in a single component
 */
export const useStore = () => {
  const ui = useUIStore();
  const filters = useFiltersStore();
  const preferences = usePreferencesStore();
  const connection = useConnectionStore();
  
  return {
    ui,
    filters,
    preferences,
    connection,
  };
};

/**
 * Selector hook for accessing specific store slices
 * Use this for performance optimization when you only need specific data
 */
export const useStoreSelector = <T>(
  selector: (stores: ReturnType<typeof useStore>) => T
): T => {
  const stores = useStore();
  return selector(stores);
};