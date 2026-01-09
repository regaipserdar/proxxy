// Main store (legacy - for backward compatibility)
export { useAppStore } from './appStore';
export type { AppState } from './appStore';

// Specialized stores
export { useUIStore } from './uiStore';
export type { UIState } from './uiStore';

export { useFiltersStore } from './filtersStore';
export type { FiltersState } from './filtersStore';

export { usePreferencesStore } from './preferencesStore';
export type { PreferencesState } from './preferencesStore';

export { useConnectionStore } from './connectionStore';
export type { ConnectionState } from './connectionStore';

// Convenience hooks
export { useStore, useStoreSelector } from './useStore';