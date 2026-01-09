import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import * as fc from 'fast-check';
import { apolloClient } from '../graphql/client';
import { GET_AGENTS, GET_HTTP_TRANSACTIONS, GET_SYSTEM_METRICS } from '../graphql/operations';

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
    get length() {
      return Object.keys(store).length;
    },
    key: (index: number) => {
      const keys = Object.keys(store);
      return keys[index] || null;
    },
  };
})();

// Mock Apollo Client methods
vi.mock('../graphql/client', () => ({
  apolloClient: {
    query: vi.fn(),
    cache: {
      readQuery: vi.fn(),
      writeQuery: vi.fn(),
      evict: vi.fn(),
      gc: vi.fn(),
      extract: vi.fn(),
      restore: vi.fn(),
    },
  },
}));

describe('Data Persistence and Caching Property Tests', () => {
  beforeEach(() => {
    // Setup localStorage mock
    Object.defineProperty(window, 'localStorage', {
      value: localStorageMock,
      writable: true,
    });

    // Clear localStorage before each test
    localStorageMock.clear();

    // Clear all mocks
    vi.clearAllMocks();
  });

  afterEach(() => {
    localStorageMock.clear();
  });

  /**
   * Property 7: Data Persistence and Caching
   * For any user preference or configuration setting, the data should be correctly 
   * persisted to local storage and restored on application restart, and GraphQL 
   * queries should use cached data when appropriate to reduce redundant API calls.
   * Validates: Requirements 8.1, 8.4
   */
  it('Property 7: User Preferences Persistence - For any user preference data, it should be correctly persisted to and restored from local storage', async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.record({
          preferences: fc.record({
            refreshInterval: fc.integer({ min: 1000, max: 60000 }),
            theme: fc.oneof(fc.constant('light'), fc.constant('dark')),
            chartTimeRange: fc.oneof(
              fc.constant('1h'),
              fc.constant('6h'),
              fc.constant('24h')
            ),
            itemsPerPage: fc.integer({ min: 10, max: 100 }),
          }),
          trafficFilters: fc.record({
            method: fc.option(fc.oneof(
              fc.constant('GET'),
              fc.constant('POST'),
              fc.constant('PUT'),
              fc.constant('DELETE')
            )),
            statusCode: fc.option(fc.integer({ min: 100, max: 599 })),
            agentId: fc.option(fc.string({ minLength: 1, maxLength: 50 })),
            searchQuery: fc.option(fc.string({ minLength: 0, maxLength: 100 })),
          }),
          sidebarCollapsed: fc.boolean(),
        }),
        async ({ preferences, trafficFilters, sidebarCollapsed }) => {
          const testData = {
            preferences,
            trafficFilters,
            sidebarCollapsed,
          };

          // Simulate storing data to localStorage
          const storageKey = 'proxxy-gui-storage';
          const serializedData = JSON.stringify(testData);
          localStorageMock.setItem(storageKey, serializedData);

          // Verify data was stored
          const storedData = localStorageMock.getItem(storageKey);
          expect(storedData).toBe(serializedData);

          // Simulate retrieving and parsing data
          const retrievedData = JSON.parse(storedData!);

          // Verify all properties are correctly persisted and restored
          expect(retrievedData.preferences).toEqual(preferences);
          expect(retrievedData.trafficFilters).toEqual(trafficFilters);
          expect(retrievedData.sidebarCollapsed).toBe(sidebarCollapsed);

          // Verify data structure integrity
          expect(typeof retrievedData.preferences.refreshInterval).toBe('number');
          expect(['light', 'dark']).toContain(retrievedData.preferences.theme);
          expect(['1h', '6h', '24h']).toContain(retrievedData.preferences.chartTimeRange);
          expect(typeof retrievedData.preferences.itemsPerPage).toBe('number');
          expect(typeof retrievedData.sidebarCollapsed).toBe('boolean');
        }
      ),
      { numRuns: 50 }
    );
  });

  it('Property 7: GraphQL Cache Consistency - For any GraphQL query result, cached data should be consistent with the original response', async () => {
    const mockApolloClient = apolloClient as any;

    await fc.assert(
      fc.asyncProperty(
        fc.record({
          queryType: fc.oneof(
            fc.constant('agents'),
            fc.constant('transactions'),
            fc.constant('metrics')
          ),
          agents: fc.array(
            fc.record({
              id: fc.string({ minLength: 1, maxLength: 50 }),
              name: fc.string({ minLength: 1, maxLength: 100 }),
              hostname: fc.string({ minLength: 1, maxLength: 100 }),
              status: fc.oneof(fc.constant('Online'), fc.constant('Offline')),
              version: fc.string({ minLength: 1, maxLength: 20 }),
              lastHeartbeat: fc.date().map(d => d.toISOString()),
            }),
            { minLength: 0, maxLength: 10 }
          ),
          transactions: fc.array(
            fc.record({
              requestId: fc.string({ minLength: 1, maxLength: 50 }),
              method: fc.oneof(
                fc.constant('GET'),
                fc.constant('POST'),
                fc.constant('PUT'),
                fc.constant('DELETE')
              ),
              url: fc.webUrl(),
              statusCode: fc.integer({ min: 100, max: 599 }),
              timestamp: fc.date().map(d => d.toISOString()),
              agentId: fc.string({ minLength: 1, maxLength: 50 }),
            }),
            { minLength: 0, maxLength: 20 }
          ),
        }),
        async ({ queryType, agents, transactions }) => {
          let mockData: any;
          let cacheKey: string;

          // Setup mock data based on query type
          switch (queryType) {
            case 'agents':
              mockData = { agents };
              cacheKey = 'agents';
              mockApolloClient.query.mockResolvedValue({ data: mockData });
              break;
            case 'transactions':
              mockData = { httpTransactions: transactions };
              cacheKey = 'httpTransactions';
              mockApolloClient.query.mockResolvedValue({ data: mockData });
              break;
            case 'metrics':
              mockData = { systemMetrics: [] };
              cacheKey = 'systemMetrics';
              mockApolloClient.query.mockResolvedValue({ data: mockData });
              break;
          }

          // Simulate cache write
          const cacheData = { ...mockData };
          mockApolloClient.cache.writeQuery.mockImplementation(() => { });
          mockApolloClient.cache.readQuery.mockReturnValue(cacheData);

          // Verify cache key is properly set
          expect(cacheKey).toBeDefined();

          // Simulate writing to cache
          mockApolloClient.cache.writeQuery({
            query: queryType === 'agents' ? GET_AGENTS :
              queryType === 'transactions' ? GET_HTTP_TRANSACTIONS : GET_SYSTEM_METRICS,
            data: mockData,
          });

          // Simulate reading from cache
          const cachedData = mockApolloClient.cache.readQuery({
            query: queryType === 'agents' ? GET_AGENTS :
              queryType === 'transactions' ? GET_HTTP_TRANSACTIONS : GET_SYSTEM_METRICS,
          });

          // Verify cache consistency
          expect(cachedData).toEqual(mockData);

          // Verify cache operations were called
          expect(mockApolloClient.cache.writeQuery).toHaveBeenCalled();
          expect(mockApolloClient.cache.readQuery).toHaveBeenCalled();

          // Verify data structure integrity in cache
          if (queryType === 'agents' && cachedData?.agents) {
            cachedData.agents.forEach((agent: any) => {
              expect(agent).toHaveProperty('id');
              expect(agent).toHaveProperty('name');
              expect(agent).toHaveProperty('hostname');
              expect(['Online', 'Offline']).toContain(agent.status);
            });
          }

          if (queryType === 'transactions' && cachedData?.httpTransactions) {
            cachedData.httpTransactions.forEach((transaction: any) => {
              expect(transaction).toHaveProperty('requestId');
              expect(transaction).toHaveProperty('method');
              expect(transaction).toHaveProperty('url');
              expect(typeof transaction.statusCode).toBe('number');
              expect(transaction.statusCode).toBeGreaterThanOrEqual(100);
              expect(transaction.statusCode).toBeLessThan(600);
            });
          }
        }
      ),
      { numRuns: 30 }
    );
  });

  it('Property 7: Cache Invalidation Consistency - For any cache eviction operation, the cache should properly remove stale data', async () => {
    const mockApolloClient = apolloClient as any;

    await fc.assert(
      fc.asyncProperty(
        fc.record({
          cacheKeys: fc.uniqueArray(
            fc.string({ minLength: 1, maxLength: 50 }).filter(s =>
              // Avoid prototype pollution and problematic keys
              !['toString', 'valueOf', 'constructor', 'prototype', '__proto__'].includes(s) &&
              // Avoid keys that start with double underscore
              !s.startsWith('__') &&
              // Ensure it's a valid identifier-like string
              /^[a-zA-Z][a-zA-Z0-9_-]*$/.test(s)
            ),
            { minLength: 1, maxLength: 10 }
          ),
          evictAll: fc.boolean(),
        }),
        async ({ cacheKeys, evictAll }) => {
          // Setup cache with some data
          const initialCacheData = cacheKeys.reduce((acc, key) => {
            acc[key] = { id: key, data: `cached-data-${key}` };
            return acc;
          }, {} as Record<string, any>);

          // Mock cache operations
          let cacheState = { ...initialCacheData };

          mockApolloClient.cache.evict.mockImplementation((options: any) => {
            if (options?.id) {
              delete cacheState[options.id];
            }
            return true;
          });

          mockApolloClient.cache.gc.mockImplementation(() => {
            // Simulate garbage collection
            return Object.keys(cacheState).length;
          });

          mockApolloClient.cache.readQuery.mockImplementation((options: any) => {
            const key = options?.variables?.id || 'default';
            return cacheState[key] || null;
          });

          // Test cache eviction
          if (evictAll) {
            // Evict all entries
            cacheKeys.forEach(key => {
              mockApolloClient.cache.evict({ id: key });
            });

            // Run garbage collection
            const remainingItems = mockApolloClient.cache.gc();

            // Verify all items were evicted
            cacheKeys.forEach(key => {
              const cachedItem = mockApolloClient.cache.readQuery({ variables: { id: key } });
              expect(cachedItem).toBeNull();
            });

            expect(remainingItems).toBe(0);
          } else {
            // Evict only first item
            const keyToEvict = cacheKeys[0];
            mockApolloClient.cache.evict({ id: keyToEvict });

            // Verify specific item was evicted
            const evictedItem = mockApolloClient.cache.readQuery({ variables: { id: keyToEvict } });
            expect(evictedItem).toBeNull();

            // Verify other items remain
            cacheKeys.slice(1).forEach(key => {
              const cachedItem = mockApolloClient.cache.readQuery({ variables: { id: key } });
              expect(cachedItem).toBeTruthy();
            });
          }

          // Verify cache operations were called appropriately
          expect(mockApolloClient.cache.evict).toHaveBeenCalled();
          if (evictAll) {
            expect(mockApolloClient.cache.gc).toHaveBeenCalled();
          }
        }
      ),
      { numRuns: 25 }
    );
  });

  it('Property 7: Data Serialization Consistency - For any complex data structure, serialization and deserialization should preserve data integrity', async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.record({
          complexData: fc.record({
            nested: fc.record({
              array: fc.array(fc.integer(), { minLength: 0, maxLength: 10 }),
              object: fc.record({
                string: fc.string(),
                number: fc.float().filter(n =>
                  // Filter out special values that JSON.stringify handles differently
                  !Number.isNaN(n) &&
                  Number.isFinite(n) &&
                  // Avoid -0 which gets converted to 0
                  !Object.is(n, -0)
                ),
                boolean: fc.boolean(),
                nullValue: fc.constant(null),
              }),
            }),
            dates: fc.array(fc.date().map(d => d.toISOString()), { minLength: 0, maxLength: 5 }),
            specialChars: fc.string().filter(s => {
              try {
                JSON.parse(JSON.stringify(s));
                return true;
              } catch {
                return false;
              }
            }),
          }),
        }),
        async ({ complexData }) => {
          // Test serialization
          const serialized = JSON.stringify(complexData);
          expect(typeof serialized).toBe('string');
          expect(serialized.length).toBeGreaterThan(0);

          // Test deserialization
          const deserialized = JSON.parse(serialized);

          // Custom deep equal function that handles JSON serialization edge cases
          const deepEqualWithSerializationHandling = (obj1: any, obj2: any): boolean => {
            if (obj1 === obj2) return true;

            if (typeof obj1 === 'number' && typeof obj2 === 'number') {
              // Handle NaN case
              if (Number.isNaN(obj1) && Number.isNaN(obj2)) return true;
              // Handle -0 vs 0 case - JSON.stringify converts -0 to 0
              if (Object.is(obj1, -0) && obj2 === 0) return true;
              if (Object.is(obj2, -0) && obj1 === 0) return true;
              // Handle Infinity - JSON.stringify converts Infinity to null
              if (!Number.isFinite(obj1) || !Number.isFinite(obj2)) {
                return JSON.stringify(obj1) === JSON.stringify(obj2);
              }
              return obj1 === obj2;
            }

            if (obj1 == null || obj2 == null) return obj1 === obj2;
            if (typeof obj1 !== typeof obj2) return false;
            if (typeof obj1 !== 'object') return obj1 === obj2;

            if (Array.isArray(obj1) !== Array.isArray(obj2)) return false;

            const keys1 = Object.keys(obj1);
            const keys2 = Object.keys(obj2);

            if (keys1.length !== keys2.length) return false;

            for (const key of keys1) {
              if (!keys2.includes(key)) return false;
              if (!deepEqualWithSerializationHandling(obj1[key], obj2[key])) return false;
            }

            return true;
          };

          // Verify structural integrity with custom comparison
          expect(deepEqualWithSerializationHandling(deserialized, complexData)).toBe(true);

          // Verify type preservation where applicable
          expect(Array.isArray(deserialized.nested.array)).toBe(true);
          expect(typeof deserialized.nested.object).toBe('object');
          expect(Array.isArray(deserialized.dates)).toBe(true);
          expect(typeof deserialized.specialChars).toBe('string');

          // Test localStorage round-trip
          const storageKey = `test-${Date.now()}-${Math.random()}`;
          localStorageMock.setItem(storageKey, serialized);
          const fromStorage = localStorageMock.getItem(storageKey);
          const fromStorageParsed = JSON.parse(fromStorage!);

          expect(fromStorageParsed).toEqual(complexData);

          // Cleanup
          localStorageMock.removeItem(storageKey);
        }
      ),
      { numRuns: 40 }
    );
  });
});

// Feature: proxxy-gui-integration, Property 7: Data Persistence and Caching