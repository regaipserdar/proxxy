import '@testing-library/jest-dom';

// Mock WebSocket for testing
(globalThis as any).WebSocket = class MockWebSocket {
  constructor(_url: string) {
    // Mock WebSocket implementation
  }
  
  close() {}
  send() {}
  
  // Mock event handlers
  onopen = null;
  onclose = null;
  onmessage = null;
  onerror = null;
};