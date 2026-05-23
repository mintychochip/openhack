/**
 * useSSE - React hook for Server-Sent Events.
 * 
 * Expected Behavior:
 *   Establishes SSE connection to gateway.
 *   Reconnects on disconnect with exponential backoff.
 *   Emits events to event bus.
 *   Provides connection status.
 *   Supports event type filtering.
 * 
 * Args:
 *   options: Connection options (url, types, autoConnect).
 * 
 * Returns:
 *   { connected, connecting, error, reconnect }: Connection state and controls.
 * 
 * Side Effects:
 *   Opens EventSource connection.
 *   Emits events to global event bus.
 */
'use client';

import { useEffect, useCallback, useState, useRef } from 'react';
import { eventBus, SSEEvent } from '@/lib/event-bus';

interface UseSSEOptions {
  url?: string;
  types?: string[];
  autoConnect?: boolean;
}

interface UseSSEReturn {
  connected: boolean;
  connecting: boolean;
  error: string | null;
  reconnect: () => void;
  disconnect: () => void;
}

export function useSSE(options: UseSSEOptions = {}): UseSSEReturn {
  const {
    url = '/api/events/stream',
    types = [],
    autoConnect = true,
  } = options;

  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const eventSourceRef = useRef<EventSource | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const connectedAtRef = useRef<number>(0);
  const rapidFailCountRef = useRef(0);
  const lastFailTimeRef = useRef<number>(0);
  const maxReconnectAttempts = 10;
  const minReconnectDelay = 3000;
  const rapidFailWindow = 2000; // ms
  const maxRapidFails = 3;

  const buildUrl = useCallback(() => {
    const baseUrl = url.startsWith('http') ? url : `${window.location.origin}${url}`;
    if (types.length > 0) {
      return `${baseUrl}?types=${types.join(',')}`;
    }
    return baseUrl;
  }, [url, types]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    setConnected(false);
    setConnecting(false);
    eventBus.setConnected(false);
  }, []);

  const connect = useCallback(() => {
    if (eventSourceRef.current) {
      return;
    }

    setConnecting(true);
    setError(null);

    const eventSource = new EventSource(buildUrl());
    eventSourceRef.current = eventSource;

    eventSource.onopen = () => {
      setConnecting(false);
      setConnected(true);
      setError(null);
      connectedAtRef.current = Date.now();
      reconnectAttemptsRef.current = 0;
      eventBus.setConnected(true);
      eventBus.resetReconnectAttempts();
    };

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        const sseEvent: SSEEvent = {
          id: data.id || `msg-${Date.now()}`,
          type: data.type || 'message',
          channel: data.channel || 'default',
          data: data.data || data,
          timestamp: data.timestamp || new Date().toISOString(),
        };
        eventBus.emit(sseEvent);
      } catch {
        // ignore parse errors for non-JSON messages
      }
    };

    eventSource.addEventListener('event', (event) => {
      try {
        const data = JSON.parse(event.data);
        eventBus.emit(data as SSEEvent);
      } catch {
        // ignore
      }
    });

    eventSource.addEventListener('connected', () => {
      // server acknowledged connection
    });

    eventSource.addEventListener('ping', () => {
      // heartbeat received
    });

    eventSource.addEventListener('error', () => {
      // server-sent error event (not connection error)
    });

    eventSource.onerror = () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
        eventSourceRef.current = null;
      }

      setConnected(false);
      setConnecting(false);
      eventBus.setConnected(false);

      const now = Date.now();
      const timeSinceLastFail = now - lastFailTimeRef.current;
      lastFailTimeRef.current = now;

      if (timeSinceLastFail < rapidFailWindow) {
        rapidFailCountRef.current++;
      } else {
        rapidFailCountRef.current = 1;
      }

      // Circuit breaker: if the server keeps rejecting us immediately,
      // stop reconnecting to avoid a "connected/disconnected" loop.
      if (rapidFailCountRef.current >= maxRapidFails) {
        setError('Live updates unavailable. The server does not support SSE.');
        return;
      }

      const uptime = Date.now() - connectedAtRef.current;
      const wasBrief = uptime < 5000 && connectedAtRef.current > 0;

      if (reconnectAttemptsRef.current < maxReconnectAttempts) {
        let delay: number;
        if (wasBrief) {
          delay = Math.min(5000 + reconnectAttemptsRef.current * 5000, 60000);
        } else {
          delay = Math.max(Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 30000), minReconnectDelay);
        }
        reconnectAttemptsRef.current++;

        reconnectTimeoutRef.current = setTimeout(() => {
          connect();
        }, delay);
      } else {
        setError('Failed to connect. Please refresh the page.');
      }
    };
  }, [buildUrl]);

  const reconnect = useCallback(() => {
    disconnect();
    reconnectAttemptsRef.current = 0;
    rapidFailCountRef.current = 0;
    lastFailTimeRef.current = 0;
    setTimeout(connect, 500);
  }, [connect, disconnect]);

  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    connected,
    connecting,
    error,
    reconnect,
    disconnect,
  };
}

/**
 * useEvent - Hook to subscribe to specific event types.
 * 
 * Expected Behavior:
 *   Subscribes to event types on mount.
 *   Unsubscribes on unmount.
 *   Calls callback on event.
 * 
 * Args:
 *   types: Event types to subscribe to.
 *   callback: Function to call on event.
 * 
 * Side Effects:
 *   Registers event listeners.
 */
export function useEvent(
  types: string[],
  callback: (event: SSEEvent) => void
): void {
  useEffect(() => {
    const unsubscribe = eventBus.subscribeMany(types, callback);
    return () => unsubscribe();
  }, [types, callback]);
}
