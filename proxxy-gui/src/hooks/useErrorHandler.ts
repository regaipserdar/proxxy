import { useCallback } from 'react';
// import { ApolloError } from '@apollo/client';
import { useConnectionStore } from '../store/connectionStore';

// Define ApolloError interface locally since it's not available in the current Apollo Client setup
interface ApolloError extends Error {
  networkError?: any;
  graphQLErrors?: Array<{
    message: string;
    extensions?: {
      code?: string;
    };
  }>;
}

export interface ErrorNotification {
  id: string;
  type: 'error' | 'warning' | 'info';
  title: string;
  message: string;
  timestamp: Date;
  dismissible?: boolean;
  autoHide?: boolean;
  duration?: number;
}

// Error classification and user-friendly messages
const ERROR_MESSAGES = {
  NETWORK_ERROR: {
    title: 'Connection Error',
    message: 'Unable to connect to the server. Please check your network connection and try again.',
  },
  GRAPHQL_ERROR: {
    title: 'Server Error',
    message: 'The server encountered an error processing your request.',
  },
  WEBSOCKET_ERROR: {
    title: 'Real-time Connection Error',
    message: 'Lost connection to real-time updates. Attempting to reconnect...',
  },
  TIMEOUT_ERROR: {
    title: 'Request Timeout',
    message: 'The request took too long to complete. Please try again.',
  },
  VALIDATION_ERROR: {
    title: 'Validation Error',
    message: 'Please check your input and try again.',
  },
  AUTHENTICATION_ERROR: {
    title: 'Authentication Required',
    message: 'Please log in to continue.',
  },
  AUTHORIZATION_ERROR: {
    title: 'Access Denied',
    message: 'You do not have permission to perform this action.',
  },
  SERVER_ERROR: {
    title: 'Server Error',
    message: 'An internal server error occurred. Please try again later.',
  },
  UNKNOWN_ERROR: {
    title: 'Unexpected Error',
    message: 'An unexpected error occurred. Please try again.',
  },
} as const;

export const useErrorHandler = () => {
  const { setErrors, setConnectionStatus } = useConnectionStore();

  const classifyError = useCallback((error: Error | ApolloError): keyof typeof ERROR_MESSAGES => {
    // Check if error has Apollo-specific properties
    const hasNetworkError = 'networkError' in error && error.networkError;
    const hasGraphQLErrors = 'graphQLErrors' in error && error.graphQLErrors;

    if (hasNetworkError) {
      const networkError = (error as any).networkError;

      // Check for specific network error types
      if (networkError.code === 'ECONNREFUSED' || networkError.code === 'ENOTFOUND') {
        return 'NETWORK_ERROR';
      }

      if ('statusCode' in networkError) {
        switch (networkError.statusCode) {
          case 401:
            return 'AUTHENTICATION_ERROR';
          case 403:
            return 'AUTHORIZATION_ERROR';
          case 408:
          case 504:
            return 'TIMEOUT_ERROR';
          case 500:
          case 502:
          case 503:
            return 'SERVER_ERROR';
          default:
            return 'NETWORK_ERROR';
        }
      }

      return 'NETWORK_ERROR';
    }

    if (hasGraphQLErrors && (error as any).graphQLErrors.length > 0) {
      const gqlError = (error as any).graphQLErrors[0];

      // Check for specific GraphQL error types
      if (gqlError.extensions?.code === 'VALIDATION_ERROR') {
        return 'VALIDATION_ERROR';
      }

      if (gqlError.extensions?.code === 'UNAUTHENTICATED') {
        return 'AUTHENTICATION_ERROR';
      }

      if (gqlError.extensions?.code === 'FORBIDDEN') {
        return 'AUTHORIZATION_ERROR';
      }

      return 'GRAPHQL_ERROR';
    }

    // Check for WebSocket errors
    if (error.message?.includes('WebSocket') || error.message?.includes('websocket')) {
      return 'WEBSOCKET_ERROR';
    }

    // Check for timeout errors
    if (error.message?.includes('timeout') || error.message?.includes('Timeout')) {
      return 'TIMEOUT_ERROR';
    }

    return 'UNKNOWN_ERROR';
  }, []);

  const handleError = useCallback((error: Error | ApolloError, context?: string) => {
    const errorType = classifyError(error);
    const errorInfo = ERROR_MESSAGES[errorType];

    // Create error notification
    const notification: ErrorNotification = {
      id: `error-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'error',
      title: errorInfo.title,
      message: context ? `${context}: ${errorInfo.message}` : errorInfo.message,
      timestamp: new Date(),
      dismissible: true,
      autoHide: errorType !== 'NETWORK_ERROR' && errorType !== 'WEBSOCKET_ERROR',
      duration: 5000,
    };

    // Update connection store with error information
    const hasNetworkError = 'networkError' in error && error.networkError;
    const hasGraphQLErrors = 'graphQLErrors' in error && error.graphQLErrors;

    if (hasNetworkError) {
      setErrors({ network: errorInfo.message });
      setConnectionStatus({ graphql: 'disconnected' });
    }

    if (hasGraphQLErrors && (error as any).graphQLErrors.length > 0) {
      setErrors({ graphql: (error as any).graphQLErrors[0].message });
    }

    // Log error for debugging
    console.error(`[ErrorHandler] ${errorType}:`, {
      error,
      context,
      notification,
      originalError: hasNetworkError || hasGraphQLErrors ? {
        networkError: (error as any).networkError,
        graphQLErrors: (error as any).graphQLErrors,
      } : error,
    });

    // Return notification for UI handling
    return notification;
  }, [classifyError, setErrors, setConnectionStatus]);

  const handleWebSocketError = useCallback((error: Event | Error, context?: string) => {
    const errorMessage = error instanceof Error ? error.message : 'WebSocket connection error';

    setErrors({ websocket: errorMessage });
    setConnectionStatus({ websocket: 'disconnected' });

    const notification: ErrorNotification = {
      id: `ws-error-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type: 'warning',
      title: ERROR_MESSAGES.WEBSOCKET_ERROR.title,
      message: context ? `${context}: ${ERROR_MESSAGES.WEBSOCKET_ERROR.message}` : ERROR_MESSAGES.WEBSOCKET_ERROR.message,
      timestamp: new Date(),
      dismissible: true,
      autoHide: false, // Keep visible until reconnected
    };

    console.error('[ErrorHandler] WebSocket Error:', {
      error,
      context,
      notification,
    });

    return notification;
  }, [setErrors, setConnectionStatus]);

  const clearError = useCallback((errorType: 'graphql' | 'websocket' | 'network') => {
    setErrors({ [errorType]: undefined });
  }, [setErrors]);

  const clearAllErrors = useCallback(() => {
    setErrors({});
  }, [setErrors]);

  return {
    handleError,
    handleWebSocketError,
    clearError,
    clearAllErrors,
    classifyError,
  };
};