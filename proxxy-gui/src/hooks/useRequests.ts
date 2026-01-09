import { useState, useEffect } from 'react';
import { useQuery, useSubscription } from '@apollo/client';
import { GET_HTTP_TRANSACTIONS, TRAFFIC_UPDATES } from '../graphql/operations';
import { HttpTransaction } from '../types/graphql';

export interface HttpRequest {
  id: string;
  method: string;
  host: string;
  path: string;
  status: number;
  timestamp: string;
  rawRequest: string;
  rawResponse: string;
}

export const useRequests = () => {
  const [requests, setRequests] = useState<HttpRequest[]>([]);

  // Fetch initial history
  const { data: initialData, loading } = useQuery(GET_HTTP_TRANSACTIONS, {
    fetchPolicy: 'network-only' // Always fetch latest on mount
  });

  // Subscribe to live updates
  const { data: updateData } = useSubscription(TRAFFIC_UPDATES);

  // Initialize data
  useEffect(() => {
    if (initialData?.requests) {
      const mapped = initialData.requests.map(mapTransactionToRequest);
      setRequests(mapped);
    }
  }, [initialData]);

  // Handle updates
  useEffect(() => {
    if (updateData?.events) {
      // updateData.events is likely a single event or list? GraphQL subscription usually returns the payload.
      // My schema defines 'events' as list?
      // subscription TrafficUpdates { events { ... } }
      // Check backend-api.md: "trafficUpdates: [HttpTransaction!]!" (List)
      const newEvents = Array.isArray(updateData.events) ? updateData.events : [updateData.events];
      const mapped = newEvents.map(mapTransactionToRequest);

      setRequests(prev => {
        // Prepend new requests
        const updated = [...mapped, ...prev];
        // Limit to 1000 items to prevent memory issues
        return updated.slice(0, 1000);
      });
    }
  }, [updateData]);

  const clearRequests = () => setRequests([]);

  return { requests, loading, clearRequests, setRequests };
};

// Helper: Map GraphQL Transaction to UI HttpRequest
function mapTransactionToRequest(tx: HttpTransaction): HttpRequest {
  let urlObj;
  try {
    urlObj = tx.url ? new URL(tx.url) : { hostname: 'unknown', pathname: tx.url || '', search: '' };
  } catch {
    urlObj = { hostname: 'unknown', pathname: tx.url || '', search: '' };
  }

  return {
    id: tx.requestId,
    method: tx.method || 'UNKNOWN',
    host: urlObj.hostname,
    path: urlObj.pathname + (urlObj.search || ''),
    status: tx.status || 0,
    timestamp: tx.timestamp ? new Date(tx.timestamp).toLocaleTimeString() : new Date().toLocaleTimeString(),
    rawRequest: formatRawRequest(tx),
    rawResponse: formatRawResponse(tx)
  };
}

function formatRawRequest(tx: HttpTransaction): string {
  const headers = parseHeaders(tx.requestHeaders);
  let raw = `${tx.method} ${tx.url} HTTP/1.1\n`;
  Object.entries(headers).forEach(([k, v]) => {
    raw += `${k}: ${v}\n`;
  });
  raw += '\n';
  if (tx.requestBody) raw += tx.requestBody;
  return raw;
}

function formatRawResponse(tx: HttpTransaction): string {
  const headers = parseHeaders(tx.responseHeaders);
  let raw = `HTTP/1.1 ${tx.status} ${getStatusText(tx.status || 0)}\n`;
  Object.entries(headers).forEach(([k, v]) => {
    raw += `${k}: ${v}\n`;
  });
  raw += '\n';
  if (tx.responseBody) raw += tx.responseBody;
  return raw;
}

function parseHeaders(headerStr?: string): Record<string, string> {
  if (!headerStr) return {};
  try {
    return JSON.parse(headerStr);
  } catch {
    return {};
  }
}

function getStatusText(status: number): string {
  // Simple statuses
  const map: Record<number, string> = {
    200: 'OK', 201: 'Created', 204: 'No Content',
    301: 'Moved Permanently', 302: 'Found', 304: 'Not Modified',
    400: 'Bad Request', 401: 'Unauthorized', 403: 'Forbidden', 404: 'Not Found', 500: 'Internal Server Error', 502: 'Bad Gateway'
  };
  return map[status] || 'Unknown';
}
