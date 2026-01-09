import React, { useEffect } from 'react';
import { usePreferencesStore } from './preferencesStore';
import { useConnectionStore } from './connectionStore';

interface StoreProviderProps {
  children: React.ReactNode;
}

/**
 * Store provider component that initializes stores and handles global store logic
 */
export const StoreProvider: React.FC<StoreProviderProps> = ({ children }) => {
  const { apiConfig } = usePreferencesStore();
  const { setConnectionStatus } = useConnectionStore();
  
  useEffect(() => {
    // Initialize connection status based on API config
    if (apiConfig.graphqlEndpoint && apiConfig.websocketEndpoint) {
      // Connection will be established by GraphQL client
      // This is just for initial state setup
    }
  }, [apiConfig, setConnectionStatus]);
  
  return <>{children}</>;
};