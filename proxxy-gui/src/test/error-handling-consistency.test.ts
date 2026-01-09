import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import * as fc from 'fast-check';
import { useErrorHandler } from '../hooks/useErrorHandler';
import { useConnectionStore } from '../store/connectionStore';

// Create a mock ApolloError class since the real one has import issues in tests
class MockApolloError extends Error {
  public networkError?: any;
  public graphQLErrors?: any[];

  constructor(options: { networkError?: any; graphQLErrors?: any[] } = {}) {
    super('Apollo Error');
    this.name = 'ApolloError';
    this.networkError = options.networkError;
    this.graphQLErrors = options.graphQLErrors || [];
  }
}

// Mock the connection store
vi.mock('../store/connectionStore', () => ({
  useConnectionStore: vi.fn(),
}));

describe('Error Handling Consistency Property Tests', () => {
  let mockSetErrors: ReturnType<typeof vi.fn>;
  let mockSetConnectionStatus: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockSetErrors = vi.fn();
    mockSetConnectionStatus = vi.fn();

    (useConnectionStore as any).mockReturnValue({
      setErrors: mockSetErrors,
      setConnectionStatus: mockSetConnectionStatus,
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  /**
   * Property 5: Error Handling Consistency
   * For any error condition (GraphQL errors, network failures, subscription errors, validation errors), 
   * the GUI should display appropriate user-friendly error messages without crashing the interface.
   * **Validates: Requirements 1.4, 9.4, 10.4, 11.5**
   */
  it('should handle all error types consistently without crashing', () => {
    const errorGenerator = fc.oneof(
      // Network errors
      fc.record({
        type: fc.constant('network'),
        message: fc.string({ minLength: 1, maxLength: 100 }).filter(s => s.trim().length > 0),
        statusCode: fc.option(fc.integer({ min: 400, max: 599 })),
        code: fc.option(fc.constantFrom('ECONNREFUSED', 'ENOTFOUND', 'TIMEOUT')),
      }),
      // GraphQL errors
      fc.record({
        type: fc.constant('graphql'),
        message: fc.string({ minLength: 1, maxLength: 100 }).filter(s => s.trim().length > 0),
        extensions: fc.option(fc.record({
          code: fc.constantFrom('VALIDATION_ERROR', 'UNAUTHENTICATED', 'FORBIDDEN'),
        })),
      }),
      // WebSocket errors
      fc.record({
        type: fc.constant('websocket'),
        message: fc.string({ minLength: 1, maxLength: 100 }).filter(s => s.trim().length > 0),
      }),
      // Generic errors
      fc.record({
        type: fc.constant('generic'),
        message: fc.string({ minLength: 1, maxLength: 100 }).filter(s => s.trim().length > 0),
      })
    );

    const contextGenerator = fc.option(fc.string({ minLength: 1, maxLength: 50 }));

    fc.assert(
      fc.property(errorGenerator, contextGenerator, (errorSpec, context) => {
        const { result } = renderHook(() => useErrorHandler());

        let error: Error | MockApolloError;

        // Create appropriate error based on type
        switch (errorSpec.type) {
          case 'network':
            const networkError = new Error(errorSpec.message) as any;
            if (errorSpec.statusCode) {
              networkError.statusCode = errorSpec.statusCode;
            }
            if (errorSpec.code) {
              networkError.code = errorSpec.code;
            }
            error = new MockApolloError({
              networkError,
            });
            break;

          case 'graphql':
            error = new MockApolloError({
              graphQLErrors: [{
                message: errorSpec.message,
                extensions: errorSpec.extensions,
              }],
            });
            break;

          case 'websocket':
            error = new Error(`WebSocket ${errorSpec.message}`);
            break;

          default:
            error = new Error(errorSpec.message);
        }

        let notification: any = null;
        let threwError = false;

        try {
          act(() => {
            notification = result.current.handleError(error, context || undefined);
          });
        } catch (e) {
          threwError = true;
        }

        // Property: Error handling should never crash the interface
        expect(threwError).toBe(false);

        // Property: All errors should produce a notification
        expect(notification).toBeDefined();
        if (notification) {
          expect(notification).toHaveProperty('id');
          expect(notification).toHaveProperty('type');
          expect(notification).toHaveProperty('title');
          expect(notification).toHaveProperty('message');
          expect(notification).toHaveProperty('timestamp');

          // Property: Notification should have valid structure
          expect(typeof notification.id).toBe('string');
          expect(notification.id.length).toBeGreaterThan(0);
          expect(['error', 'warning', 'info']).toContain(notification.type);
          expect(typeof notification.title).toBe('string');
          expect(notification.title.length).toBeGreaterThan(0);
          expect(typeof notification.message).toBe('string');
          expect(notification.message.length).toBeGreaterThan(0);
          expect(notification.timestamp).toBeInstanceOf(Date);

          // Property: Context should be included in message when provided
          if (context && notification) {
            expect(notification.message).toContain(context);
          }
        }

        // Property: Connection store should be updated for connection-related errors
        if (errorSpec.type === 'network' || errorSpec.type === 'graphql') {
          expect(mockSetErrors).toHaveBeenCalled();
        }
      }),
      {
        numRuns: 50, // Reduced for faster execution
        verbose: true,
      }
    );
  });

  it('should classify errors consistently', () => {
    const errorSpecGenerator = fc.oneof(
      // Test Apollo errors with network errors
      fc.record({
        networkError: fc.record({
          message: fc.string({ minLength: 1 }).filter(s => s.trim().length > 0),
          statusCode: fc.option(fc.integer({ min: 400, max: 599 })),
          code: fc.option(fc.constantFrom('ECONNREFUSED', 'ENOTFOUND')),
        }),
      }),
      // Test Apollo errors with GraphQL errors
      fc.record({
        graphQLErrors: fc.array(fc.record({
          message: fc.string({ minLength: 1 }).filter(s => s.trim().length > 0),
          extensions: fc.option(fc.record({
            code: fc.constantFrom('VALIDATION_ERROR', 'UNAUTHENTICATED', 'FORBIDDEN'),
          })),
        }), { minLength: 1, maxLength: 3 }),
      }),
      // Test regular errors
      fc.record({
        message: fc.oneof(
          fc.string({ minLength: 1 }).filter(s => s.trim().length > 0),
          fc.constantFrom('WebSocket error', 'timeout occurred', 'Timeout exceeded')
        ),
      })
    );

    fc.assert(
      fc.property(errorSpecGenerator, (errorSpec) => {
        const { result } = renderHook(() => useErrorHandler());

        let error: Error | MockApolloError;

        if ('networkError' in errorSpec) {
          error = new MockApolloError({ networkError: errorSpec.networkError as any });
        } else if ('graphQLErrors' in errorSpec) {
          error = new MockApolloError({ graphQLErrors: errorSpec.graphQLErrors as any });
        } else {
          error = new Error(errorSpec.message);
        }

        let classification: string = '';
        let threwError = false;

        try {
          act(() => {
            classification = result.current.classifyError(error);
          });
        } catch (e) {
          threwError = true;
          classification = 'UNKNOWN_ERROR'; // Default value if error occurs
        }

        // Property: Classification should never throw
        expect(threwError).toBe(false);

        // Property: Classification should always return a valid error type
        expect(classification).toBeDefined();
        expect(typeof classification).toBe('string');
        expect(classification.length).toBeGreaterThan(0);

        // Property: Classification should be consistent for same error types
        const validClassifications = [
          'NETWORK_ERROR',
          'GRAPHQL_ERROR',
          'WEBSOCKET_ERROR',
          'TIMEOUT_ERROR',
          'VALIDATION_ERROR',
          'AUTHENTICATION_ERROR',
          'AUTHORIZATION_ERROR',
          'SERVER_ERROR',
          'UNKNOWN_ERROR'
        ];
        expect(validClassifications).toContain(classification);
      }),
      {
        numRuns: 50,
        verbose: true,
      }
    );
  });

  it('should handle WebSocket errors consistently', () => {
    const wsErrorGenerator = fc.oneof(
      fc.record({
        type: fc.constant('error'),
        message: fc.string({ minLength: 1, maxLength: 100 }).filter(s => s.trim().length > 0),
      }),
      fc.record({
        type: fc.constant('event'),
        code: fc.integer({ min: 1000, max: 4999 }),
        reason: fc.string({ maxLength: 100 }),
      })
    );

    const contextGenerator = fc.option(fc.string({ minLength: 1, maxLength: 50 }));

    fc.assert(
      fc.property(wsErrorGenerator, contextGenerator, (errorSpec, context) => {
        const { result } = renderHook(() => useErrorHandler());

        let error: Error | Event;

        if (errorSpec.type === 'error') {
          error = new Error(errorSpec.message);
        } else {
          // Create a mock CloseEvent
          error = new Event('close') as any;
          (error as any).code = errorSpec.code;
          (error as any).reason = errorSpec.reason;
        }

        let notification: any = null;
        let threwError = false;

        try {
          act(() => {
            notification = result.current.handleWebSocketError(error, context || undefined);
          });
        } catch (e) {
          threwError = true;
        }

        // Property: WebSocket error handling should never crash
        expect(threwError).toBe(false);

        // Property: Should always produce a notification
        expect(notification).toBeDefined();
        expect(notification).toHaveProperty('id');
        expect(notification).toHaveProperty('type');
        expect(notification).toHaveProperty('title');
        expect(notification).toHaveProperty('message');

        // Property: WebSocket errors should update connection status
        expect(mockSetErrors).toHaveBeenCalledWith({ websocket: expect.any(String) });
        expect(mockSetConnectionStatus).toHaveBeenCalledWith({ websocket: 'disconnected' });

        // Property: Context should be included when provided
        if (context) {
          expect(notification.message).toContain(context);
        }
      }),
      {
        numRuns: 50,
        verbose: true,
      }
    );
  });

  it('should clear errors consistently', () => {
    const errorTypeGenerator = fc.constantFrom('graphql', 'websocket', 'network');

    fc.assert(
      fc.property(errorTypeGenerator, (errorType) => {
        const { result } = renderHook(() => useErrorHandler());

        let threwError = false;

        try {
          act(() => {
            result.current.clearError(errorType);
          });
        } catch (e) {
          threwError = true;
        }

        // Property: Clearing errors should never throw
        expect(threwError).toBe(false);

        // Property: Should call setErrors with the correct parameter
        expect(mockSetErrors).toHaveBeenCalledWith({ [errorType]: undefined });
      }),
      {
        numRuns: 25,
        verbose: true,
      }
    );
  });
});