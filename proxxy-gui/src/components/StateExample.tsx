import React from 'react';
import { useUIStore, useFiltersStore, usePreferencesStore, useConnectionStore } from '../store';

/**
 * Example component demonstrating the state management layer usage
 * This can be removed once the stores are integrated into actual components
 */
export const StateExample: React.FC = () => {
  const { sidebarCollapsed, setSidebarCollapsed, setCurrentPage } = useUIStore();
  const { trafficFilters, setTrafficFilters } = useFiltersStore();
  const { preferences, setPreferences } = usePreferencesStore();
  const { connectionStatus, setConnectionStatus } = useConnectionStore();

  return (
    <div className="p-4 space-y-4">
      <h2 className="text-xl font-bold">State Management Example</h2>
      
      <div className="space-y-2">
        <h3 className="font-semibold">UI State</h3>
        <button
          onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
          className="px-4 py-2 bg-blue-500 text-white rounded"
        >
          {sidebarCollapsed ? 'Expand' : 'Collapse'} Sidebar
        </button>
        <button
          onClick={() => setCurrentPage('traffic')}
          className="px-4 py-2 bg-green-500 text-white rounded ml-2"
        >
          Go to Traffic Page
        </button>
      </div>

      <div className="space-y-2">
        <h3 className="font-semibold">Filters</h3>
        <input
          type="text"
          placeholder="Search query"
          value={trafficFilters.searchQuery || ''}
          onChange={(e) => setTrafficFilters({ searchQuery: e.target.value })}
          className="px-3 py-2 border rounded"
        />
        <select
          value={trafficFilters.method || ''}
          onChange={(e) => setTrafficFilters({ method: e.target.value || undefined })}
          className="px-3 py-2 border rounded ml-2"
        >
          <option value="">All Methods</option>
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
          <option value="DELETE">DELETE</option>
        </select>
      </div>

      <div className="space-y-2">
        <h3 className="font-semibold">Preferences</h3>
        <label className="flex items-center space-x-2">
          <span>Theme:</span>
          <select
            value={preferences.theme}
            onChange={(e) => setPreferences({ theme: e.target.value as 'light' | 'dark' })}
            className="px-3 py-2 border rounded"
          >
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </label>
        <label className="flex items-center space-x-2">
          <span>Items per page:</span>
          <input
            type="number"
            value={preferences.itemsPerPage}
            onChange={(e) => setPreferences({ itemsPerPage: parseInt(e.target.value) })}
            className="px-3 py-2 border rounded w-20"
            min="10"
            max="200"
          />
        </label>
      </div>

      <div className="space-y-2">
        <h3 className="font-semibold">Connection Status</h3>
        <div className="flex space-x-4">
          <span className={`px-2 py-1 rounded text-sm ${
            connectionStatus.graphql === 'connected' ? 'bg-green-100 text-green-800' :
            connectionStatus.graphql === 'reconnecting' ? 'bg-yellow-100 text-yellow-800' :
            'bg-red-100 text-red-800'
          }`}>
            GraphQL: {connectionStatus.graphql}
          </span>
          <span className={`px-2 py-1 rounded text-sm ${
            connectionStatus.websocket === 'connected' ? 'bg-green-100 text-green-800' :
            connectionStatus.websocket === 'reconnecting' ? 'bg-yellow-100 text-yellow-800' :
            'bg-red-100 text-red-800'
          }`}>
            WebSocket: {connectionStatus.websocket}
          </span>
        </div>
        <button
          onClick={() => setConnectionStatus({ graphql: 'connected', websocket: 'connected' })}
          className="px-4 py-2 bg-green-500 text-white rounded"
        >
          Simulate Connected
        </button>
        <button
          onClick={() => setConnectionStatus({ graphql: 'disconnected', websocket: 'disconnected' })}
          className="px-4 py-2 bg-red-500 text-white rounded ml-2"
        >
          Simulate Disconnected
        </button>
      </div>

      <div className="mt-4 p-4 bg-gray-100 rounded">
        <h4 className="font-semibold mb-2">Current State:</h4>
        <pre className="text-sm overflow-auto">
          {JSON.stringify({
            ui: { sidebarCollapsed },
            filters: trafficFilters,
            preferences: { theme: preferences.theme, itemsPerPage: preferences.itemsPerPage },
            connection: connectionStatus,
          }, null, 2)}
        </pre>
      </div>
    </div>
  );
};