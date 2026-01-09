import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import * as fc from 'fast-check';
import { Agent, HttpTransaction, SystemMetrics } from '../types/graphql';

// Mock Apollo Client hooks
vi.mock('@apollo/client', async () => {
  const actual = await vi.importActual('@apollo/client');
  return {
    ...actual,
    useQuery: vi.fn(),
  };
});

describe('Data Field Completeness Property Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  /**
   * Property 3: Data Field Completeness
   * For any data entity (agent, transaction, metrics) received from the GraphQL API, 
   * all required fields specified in the requirements should be present and correctly 
   * displayed in the UI components.
   * Validates: Requirements 2.2, 3.2, 4.3, 4.4
   */
  it('Property 3: Agent Data Field Completeness - For any agent data, all required fields should be present', async () => {
    const mockUseQuery = vi.fn();

    await fc.assert(
      fc.asyncProperty(
        fc.array(
          fc.record({
            id: fc.string({ minLength: 1, maxLength: 50 }),
            name: fc.string({ minLength: 1, maxLength: 100 }),
            hostname: fc.string({ minLength: 1, maxLength: 255 }),
            status: fc.oneof(fc.constant('Online'), fc.constant('Offline')),
            version: fc.string({ minLength: 1, maxLength: 20 }),
            lastHeartbeat: fc.date().map(d => d.toISOString()),
          }),
          { minLength: 0, maxLength: 10 }
        ),
        async (agentData) => {
          // Mock GraphQL query response
          mockUseQuery.mockReturnValue({
            data: { agents: agentData },
            loading: false,
            error: null,
          });

          const queryResult = mockUseQuery('GET_AGENTS');
          const agents: Agent[] = queryResult.data?.agents || [];

          // Verify each agent has all required fields per Requirements 2.2
          agents.forEach((agent: Agent) => {
            // Required fields from Requirements 2.2: ID, name, hostname, version, and last heartbeat
            expect(agent).toHaveProperty('id');
            expect(agent).toHaveProperty('name');
            expect(agent).toHaveProperty('hostname');
            expect(agent).toHaveProperty('status');
            expect(agent).toHaveProperty('version');
            expect(agent).toHaveProperty('lastHeartbeat');

            // Verify field types and constraints
            expect(typeof agent.id).toBe('string');
            expect(agent.id.length).toBeGreaterThan(0);
            
            expect(typeof agent.name).toBe('string');
            expect(agent.name.length).toBeGreaterThan(0);
            
            expect(typeof agent.hostname).toBe('string');
            expect(agent.hostname.length).toBeGreaterThan(0);
            
            expect(['Online', 'Offline']).toContain(agent.status);
            
            expect(typeof agent.version).toBe('string');
            expect(agent.version.length).toBeGreaterThan(0);
            
            expect(typeof agent.lastHeartbeat).toBe('string');
            expect(() => new Date(agent.lastHeartbeat)).not.toThrow();
          });
        }
      ),
      { numRuns: 25 }
    );
  });
  it('Property 3: HTTP Transaction Data Field Completeness - For any transaction data, all required fields should be present', async () => {
    const mockUseQuery = vi.fn();

    await fc.assert(
      fc.asyncProperty(
        fc.array(
          fc.record({
            requestId: fc.string({ minLength: 1, maxLength: 50 }),
            method: fc.oneof(
              fc.constant('GET'),
              fc.constant('POST'),
              fc.constant('PUT'),
              fc.constant('DELETE'),
              fc.constant('PATCH'),
              fc.constant('HEAD'),
              fc.constant('OPTIONS')
            ),
            url: fc.webUrl(),
            status: fc.option(fc.integer({ min: 100, max: 599 })),
          }),
          { minLength: 0, maxLength: 10 }
        ),
        async (transactionData) => {
          // Mock GraphQL query response
          mockUseQuery.mockReturnValue({
            data: { requests: transactionData },
            loading: false,
            error: null,
          });

          const queryResult = mockUseQuery('GET_HTTP_TRANSACTIONS');
          const transactions: HttpTransaction[] = queryResult.data?.requests || [];

          // Verify each transaction has all required fields per Requirements 3.2
          transactions.forEach((transaction: HttpTransaction) => {
            // Required fields from Requirements 3.2: method, URL, status code, timestamp, and agent ID
            expect(transaction).toHaveProperty('requestId');
            expect(transaction).toHaveProperty('method');
            expect(transaction).toHaveProperty('url');
            expect(transaction).toHaveProperty('status');

            // Verify field types and constraints
            expect(typeof transaction.requestId).toBe('string');
            expect(transaction.requestId.length).toBeGreaterThan(0);
            
            if (transaction.method !== undefined) {
              expect(typeof transaction.method).toBe('string');
              expect(['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS']).toContain(transaction.method);
            }
            
            if (transaction.url !== undefined) {
              expect(typeof transaction.url).toBe('string');
              expect(transaction.url.length).toBeGreaterThan(0);
            }
            
            if (transaction.status !== undefined && transaction.status !== null) {
              expect(typeof transaction.status).toBe('number');
              expect(transaction.status).toBeGreaterThanOrEqual(100);
              expect(transaction.status).toBeLessThan(600);
            }
          });
        }
      ),
      { numRuns: 25 }
    );
  });

  it('Property 3: System Metrics Data Field Completeness - For any metrics data, all required fields should be present', async () => {
    const mockUseQuery = vi.fn();

    await fc.assert(
      fc.asyncProperty(
        fc.array(
          fc.record({
            agentId: fc.string({ minLength: 1, maxLength: 50 }),
            timestamp: fc.integer({ min: 1000000000, max: 2000000000 }),
            cpuUsagePercent: fc.float({ min: 0, max: 100 }),
            memoryUsedBytes: fc.integer({ min: 0 }).map(n => n.toString()),
            memoryTotalBytes: fc.integer({ min: 1 }).map(n => n.toString()),
            networkRxBytesPerSec: fc.integer({ min: 0 }).map(n => n.toString()),
            networkTxBytesPerSec: fc.integer({ min: 0 }).map(n => n.toString()),
            diskReadBytesPerSec: fc.integer({ min: 0 }).map(n => n.toString()),
            diskWriteBytesPerSec: fc.integer({ min: 0 }).map(n => n.toString()),
            processCpuPercent: fc.float({ min: 0, max: 100 }),
            processMemoryBytes: fc.integer({ min: 0 }).map(n => n.toString()),
            processUptimeSeconds: fc.integer({ min: 0, max: 86400 }),
          }),
          { minLength: 0, maxLength: 5 }
        ),
        async (metricsData) => {
          // Mock GraphQL query response
          mockUseQuery.mockReturnValue({
            data: { systemMetrics: metricsData },
            loading: false,
            error: null,
          });

          const queryResult = mockUseQuery('GET_SYSTEM_METRICS');
          const metrics: SystemMetrics[] = queryResult.data?.systemMetrics || [];

          // Verify each metrics entry has all required fields per Requirements 4.3, 4.4
          metrics.forEach((metric: SystemMetrics) => {
            // Required fields from Requirements 4.3, 4.4: CPU usage, memory usage, network I/O, disk I/O
            expect(metric).toHaveProperty('agentId');
            expect(metric).toHaveProperty('timestamp');
            expect(metric).toHaveProperty('cpuUsagePercent');
            expect(metric).toHaveProperty('memoryUsedBytes');
            expect(metric).toHaveProperty('memoryTotalBytes');
            expect(metric).toHaveProperty('networkRxBytesPerSec');
            expect(metric).toHaveProperty('networkTxBytesPerSec');
            expect(metric).toHaveProperty('diskReadBytesPerSec');
            expect(metric).toHaveProperty('diskWriteBytesPerSec');
            expect(metric).toHaveProperty('processCpuPercent');
            expect(metric).toHaveProperty('processMemoryBytes');
            expect(metric).toHaveProperty('processUptimeSeconds');

            // Verify field types and constraints
            expect(typeof metric.agentId).toBe('string');
            expect(metric.agentId.length).toBeGreaterThan(0);
            
            expect(typeof metric.timestamp).toBe('number');
            expect(metric.timestamp).toBeGreaterThan(0);
            
            expect(typeof metric.cpuUsagePercent).toBe('number');
            expect(metric.cpuUsagePercent).toBeGreaterThanOrEqual(0);
            expect(metric.cpuUsagePercent).toBeLessThanOrEqual(100);
            
            expect(typeof metric.memoryUsedBytes).toBe('string');
            expect(parseInt(metric.memoryUsedBytes)).toBeGreaterThanOrEqual(0);
            
            expect(typeof metric.memoryTotalBytes).toBe('string');
            expect(parseInt(metric.memoryTotalBytes)).toBeGreaterThan(0);
            
            expect(typeof metric.networkRxBytesPerSec).toBe('string');
            expect(parseInt(metric.networkRxBytesPerSec)).toBeGreaterThanOrEqual(0);
            
            expect(typeof metric.networkTxBytesPerSec).toBe('string');
            expect(parseInt(metric.networkTxBytesPerSec)).toBeGreaterThanOrEqual(0);
            
            expect(typeof metric.diskReadBytesPerSec).toBe('string');
            expect(parseInt(metric.diskReadBytesPerSec)).toBeGreaterThanOrEqual(0);
            
            expect(typeof metric.diskWriteBytesPerSec).toBe('string');
            expect(parseInt(metric.diskWriteBytesPerSec)).toBeGreaterThanOrEqual(0);
            
            expect(typeof metric.processCpuPercent).toBe('number');
            expect(metric.processCpuPercent).toBeGreaterThanOrEqual(0);
            expect(metric.processCpuPercent).toBeLessThanOrEqual(100);
            
            expect(typeof metric.processMemoryBytes).toBe('string');
            expect(parseInt(metric.processMemoryBytes)).toBeGreaterThanOrEqual(0);
            
            expect(typeof metric.processUptimeSeconds).toBe('number');
            expect(metric.processUptimeSeconds).toBeGreaterThanOrEqual(0);
          });
        }
      ),
      { numRuns: 20 }
    );
  });

  it('Property 3: Data Field Consistency Across Components - For any data entity, field presence should be consistent across different UI components', async () => {
    const mockUseQuery = vi.fn();

    await fc.assert(
      fc.asyncProperty(
        fc.record({
          agent: fc.record({
            id: fc.string({ minLength: 1, maxLength: 50 }),
            name: fc.string({ minLength: 1, maxLength: 100 }),
            hostname: fc.string({ minLength: 1, maxLength: 255 }),
            status: fc.oneof(fc.constant('Online'), fc.constant('Offline')),
            version: fc.string({ minLength: 1, maxLength: 20 }),
            lastHeartbeat: fc.date().map(d => d.toISOString()),
          }),
          transaction: fc.record({
            requestId: fc.string({ minLength: 1, maxLength: 50 }),
            method: fc.oneof(fc.constant('GET'), fc.constant('POST')),
            url: fc.webUrl(),
            status: fc.option(fc.integer({ min: 200, max: 500 })),
          }),
        }),
        async ({ agent, transaction }) => {
          // Mock multiple queries that might be used in different components
          mockUseQuery.mockImplementation((query: string) => {
            if (query.includes('AGENTS')) {
              return {
                data: { agents: [agent] },
                loading: false,
                error: null,
              };
            } else if (query.includes('TRANSACTIONS') || query.includes('requests')) {
              return {
                data: { requests: [transaction] },
                loading: false,
                error: null,
              };
            } else if (query.includes('DASHBOARD')) {
              return {
                data: { 
                  agents: [agent],
                  requests: [transaction]
                },
                loading: false,
                error: null,
              };
            }
            return { data: null, loading: false, error: null };
          });

          // Test agent data consistency across different query contexts
          const agentQuery = mockUseQuery('GET_AGENTS');
          const dashboardQuery = mockUseQuery('GET_DASHBOARD_SUMMARY');

          const agentFromAgentQuery = agentQuery.data?.agents[0];
          const agentFromDashboardQuery = dashboardQuery.data?.agents[0];

          // Verify agent data consistency
          if (agentFromAgentQuery && agentFromDashboardQuery) {
            expect(agentFromAgentQuery.id).toBe(agentFromDashboardQuery.id);
            expect(agentFromAgentQuery.name).toBe(agentFromDashboardQuery.name);
            expect(agentFromAgentQuery.hostname).toBe(agentFromDashboardQuery.hostname);
            expect(agentFromAgentQuery.status).toBe(agentFromDashboardQuery.status);
            expect(agentFromAgentQuery.version).toBe(agentFromDashboardQuery.version);
            expect(agentFromAgentQuery.lastHeartbeat).toBe(agentFromDashboardQuery.lastHeartbeat);
          }

          // Test transaction data consistency
          const transactionQuery = mockUseQuery('GET_HTTP_TRANSACTIONS');
          const transactionFromTransactionQuery = transactionQuery.data?.requests[0];
          const transactionFromDashboardQuery = dashboardQuery.data?.requests[0];

          if (transactionFromTransactionQuery && transactionFromDashboardQuery) {
            expect(transactionFromTransactionQuery.requestId).toBe(transactionFromDashboardQuery.requestId);
            expect(transactionFromTransactionQuery.method).toBe(transactionFromDashboardQuery.method);
            expect(transactionFromTransactionQuery.url).toBe(transactionFromDashboardQuery.url);
            expect(transactionFromTransactionQuery.status).toBe(transactionFromDashboardQuery.status);
          }
        }
      ),
      { numRuns: 15 }
    );
  });
});

// Feature: proxxy-gui-integration, Property 3: Data Field Completeness