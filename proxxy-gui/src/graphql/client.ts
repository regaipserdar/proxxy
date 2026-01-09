import {
  ApolloClient,
  InMemoryCache,
  createHttpLink,
  split,
  from
} from '@apollo/client';
import { GraphQLWsLink } from '@apollo/client/link/subscriptions';
import { getMainDefinition } from '@apollo/client/utilities';
import { onError } from '@apollo/client/link/error';
import { createClient } from 'graphql-ws';
import { RetryLink } from '@apollo/client/link/retry';

// GraphQL endpoint configuration
const GRAPHQL_HTTP_ENDPOINT = 'http://localhost:9090/graphql';
const GRAPHQL_WS_ENDPOINT = 'ws://localhost:9090/graphql';

// Connection status management
export interface ConnectionStatus {
  graphql: 'connected' | 'disconnected' | 'reconnecting';
  websocket: 'connected' | 'disconnected' | 'reconnecting';
}

// Global connection status callbacks
let connectionStatusCallbacks: Array<(status: Partial<ConnectionStatus>) => void> = [];

export const subscribeToConnectionStatus = (callback: (status: Partial<ConnectionStatus>) => void) => {
  connectionStatusCallbacks.push(callback);
  return () => {
    connectionStatusCallbacks = connectionStatusCallbacks.filter(cb => cb !== callback);
  };
};

const notifyConnectionStatus = (status: Partial<ConnectionStatus>) => {
  connectionStatusCallbacks.forEach(callback => callback(status));
};

// HTTP Link for queries and mutations
const httpLink = createHttpLink({
  uri: GRAPHQL_HTTP_ENDPOINT,
  credentials: 'same-origin',
});

// Retry link for failed requests
const retryLink = new RetryLink({
  delay: {
    initial: 1000,
    max: 5000,
    jitter: true,
  },
  attempts: {
    max: Infinity, // Retry forever
    retryIf: (error, _operation) => {
      // Retry on network errors and 5xx server errors
      const isNetworkError = !!error && (
        (error as any).networkError?.message?.includes('fetch') ||
        (error as any).networkError?.statusCode >= 500 ||
        (error as any).message?.includes('Failed to fetch') ||
        (error as any).message?.includes('Load failed')
      );

      if (isNetworkError) {
        console.log('[GraphQL Retry] Retrying request due to network error...');
      }

      return isNetworkError;
    },
  },
});

// WebSocket client with enhanced connection management
const wsClient = createClient({
  url: GRAPHQL_WS_ENDPOINT,
  connectionParams: {
    // Add authentication headers if needed in the future
  },
  retryAttempts: Infinity, // Retry forever
  shouldRetry: (errOrCloseEvent) => {
    // Retry on connection errors but not on authentication failures
    if (errOrCloseEvent instanceof CloseEvent) {
      // 1000: Normal Closure, 1001: Going Away
      // Retry for everything else
      return errOrCloseEvent.code !== 1000 && errOrCloseEvent.code !== 1001;
    }
    return true;
  },
  on: {
    connecting: () => {
      console.log('[GraphQL WS] Connecting...');
      notifyConnectionStatus({ websocket: 'reconnecting' });
    },
    opened: () => {
      console.log('[GraphQL WS] Connected successfully');
      notifyConnectionStatus({ websocket: 'connected' });
    },
    closed: (event) => {
      console.log('[GraphQL WS] Connection closed:', event);
      notifyConnectionStatus({ websocket: 'disconnected' });
    },
    error: (error) => {
      console.error('[GraphQL WS] Connection error:', error);
      notifyConnectionStatus({ websocket: 'disconnected' });
    },
  },
});

// WebSocket Link for subscriptions
const wsLink = new GraphQLWsLink(wsClient);

// Enhanced error handling link
const errorLink = onError((errorResponse: any) => {
  const { graphQLErrors, networkError, operation } = errorResponse;

  if (graphQLErrors) {
    graphQLErrors.forEach((error: any) => {
      console.error(
        `[GraphQL Error] Message: ${error.message}, Location: ${error.locations}, Path: ${error.path}`,
        { operation: operation.operationName, variables: operation.variables }
      );

      // Handle specific GraphQL error types
      if (error.extensions?.code === 'UNAUTHENTICATED') {
        notifyConnectionStatus({ graphql: 'disconnected' });
      }
    });
  }

  if (networkError) {
    console.error(`[Network Error] ${networkError.message}`, {
      operation: operation.operationName,
      variables: operation.variables,
    });

    // Update connection status based on network error
    notifyConnectionStatus({ graphql: 'disconnected' });

    // Handle specific network errors
    if ('statusCode' in networkError) {
      const statusCode = (networkError as any).statusCode;
      switch (statusCode) {
        case 401:
          console.error('Unauthorized access - authentication required');
          break;
        case 403:
          console.error('Forbidden access - insufficient permissions');
          break;
        case 408:
        case 504:
          console.error('Request timeout');
          break;
        case 500:
        case 502:
        case 503:
          console.error('Server error - retrying may help');
          break;
        default:
          console.error(`HTTP error ${statusCode}`);
      }
    } else if (networkError.message?.includes('fetch')) {
      console.error('Network fetch error - server may be unreachable');
    }
  }
});

// Combine all links with proper ordering
const link = from([
  errorLink,
  retryLink,
  split(
    ({ query }) => {
      const definition = getMainDefinition(query);
      return (
        definition.kind === 'OperationDefinition' &&
        definition.operation === 'subscription'
      );
    },
    wsLink,
    httpLink,
  ),
]);

// Apollo Client cache configuration
const cache = new InMemoryCache({
  typePolicies: {
    Agent: {
      keyFields: ['id'],
      fields: {
        lastHeartbeat: {
          merge(_existing, incoming) {
            return incoming;
          },
        },
      },
    },
    TrafficEventGql: {
      keyFields: ['requestId'],
    },
    SystemMetricsGql: {
      keyFields: ['agentId', 'timestamp'],
    },
    Query: {
      fields: {
        // OPTIMIZATION: Improved requests list handling
        requests: {
          // CRITICAL: Prevent duplicates during pagination and subscriptions
          merge(existing = [], incoming, { readField }) {
            // Create a Map for deduplication
            const merged = new Map();

            // Add existing items
            existing.forEach((item: any) => {
              const id = readField('requestId', item);
              if (id) merged.set(id, item);
            });

            // Add/update with incoming items (newer data wins)
            incoming.forEach((item: any) => {
              const id = readField('requestId', item);
              if (id) merged.set(id, item);
            });

            // Convert back to array, newest first
            return Array.from(merged.values());
          },
        },

        // Single request detail (no merge needed, always replace)
        request: {
          read(existing, { args, toReference }) {
            // Try to read from cache first
            if (args?.id) {
              return toReference({
                __typename: 'TrafficEventGql',
                requestId: args.id,
              });
            }
            return existing;
          },
        },

        systemMetrics: {
          keyArgs: ['agentId'],
          merge(existing = [], incoming, { readField }) {
            // OPTIMIZATION: Deduplicate by agentId + timestamp
            const merged = new Map();

            existing.forEach((item: any) => {
              const agentId = readField('agentId', item);
              const timestamp = readField('timestamp', item);
              const key = `${agentId}-${timestamp}`;
              merged.set(key, item);
            });

            incoming.forEach((item: any) => {
              const agentId = readField('agentId', item);
              const timestamp = readField('timestamp', item);
              const key = `${agentId}-${timestamp}`;
              merged.set(key, item);
            });

            // Keep last 100 entries, sorted by timestamp
            const sorted = Array.from(merged.values()).sort((a: any, b: any) => {
              const tsA = readField('timestamp', a) as number;
              const tsB = readField('timestamp', b) as number;
              return tsB - tsA; // Newest first
            });

            return sorted.slice(0, 100);
          },
        },
      },
    },
  },
});

// Create Apollo Client instance
export const apolloClient = new ApolloClient({
  link,
  cache,
  defaultOptions: {
    watchQuery: {
      errorPolicy: 'all',
      notifyOnNetworkStatusChange: true,
    },
    query: {
      errorPolicy: 'all',
    },
    mutate: {
      errorPolicy: 'all',
    },
  },
});

// Test GraphQL connection with enhanced error handling
export const testGraphQLConnection = async (): Promise<boolean> => {
  try {
    notifyConnectionStatus({ graphql: 'reconnecting' });

    const { TEST_CONNECTION } = await import('./operations');
    const result = await apolloClient.query({
      query: TEST_CONNECTION,
      fetchPolicy: 'network-only',
      errorPolicy: 'none', // Throw errors for connection testing
    });

    const isConnected = !!(result.data && (result.data as any).hello);
    notifyConnectionStatus({ graphql: isConnected ? 'connected' : 'disconnected' });

    return isConnected;
  } catch (error) {
    console.error('GraphQL connection test failed:', error);
    notifyConnectionStatus({ graphql: 'disconnected' });
    return false;
  }
};

// Initialize connection and return status
export const initializeGraphQLConnection = async (): Promise<ConnectionStatus> => {
  const status: ConnectionStatus = {
    graphql: 'disconnected',
    websocket: 'disconnected',
  };

  try {
    // Test HTTP connection
    const httpConnected = await testGraphQLConnection();
    status.graphql = httpConnected ? 'connected' : 'disconnected';

    // WebSocket connection status is managed by the wsClient callbacks
    // Initial status will be updated through the connection callbacks
    status.websocket = 'disconnected';
  } catch (error) {
    console.error('Failed to initialize GraphQL connection:', error);
    notifyConnectionStatus({ graphql: 'disconnected', websocket: 'disconnected' });
  }

  return status;
};

// Utility function to check if client is ready
export const isClientReady = (): boolean => {
  return apolloClient !== null;
};

// Utility function to reset client connection
export const resetConnection = async (): Promise<void> => {
  try {
    await apolloClient.resetStore();
    await testGraphQLConnection();
  } catch (error) {
    console.error('Failed to reset connection:', error);
    throw error;
  }
};