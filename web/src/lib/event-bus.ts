/**
 * Event Bus - Client-side event management for SSE.
 * 
 * Expected Behavior:
 *   Centralized event listener registry.
 *   Allows components to subscribe/unsubscribe to event types.
 *   Buffers events during connection gaps.
 *   Provides event history (last 100 events).
 * 
 * Side Effects:
 *   Maintains global event listener state.
 */

export type EventType =
  | 'team.created'
  | 'team.joined'
  | 'team.left'
  | 'project.submitted'
  | 'project.updated'
  | 'event.rsvp'
  | 'event.cancelled'
  | 'score.submitted'
  | 'score.updated'
  | 'leaderboard.updated'
  | 'announcement.sent'
  | 'user.registered'
  | 'user.logged_in'
  | 'phase.transitioned'
  | string;

export interface SSEEvent {
  id: string;
  type: EventType;
  channel: string;
  data: any;
  timestamp: string;
}

type EventCallback = (event: SSEEvent) => void;

class EventBus {
  private listeners: Map<EventType, Set<EventCallback>> = new Map();
  private history: SSEEvent[] = [];
  private maxHistory = 100;
  private connected = false;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  /**
   * Subscribe to an event type.
   * 
   * Expected Behavior:
   *   Registers callback for event type.
   *   Returns unsubscribe function.
   * 
   * Args:
   *   type: Event type to subscribe to.
   *   callback: Function to call on event.
   * 
   * Returns:
   *   () => void: Unsubscribe function.
   * 
   * Side Effects:
   *   Adds callback to listeners map.
   */
  subscribe(type: EventType, callback: EventCallback): () => void {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set());
    }
    this.listeners.get(type)!.add(callback);

    // Return unsubscribe function
    return () => {
      const set = this.listeners.get(type);
      if (set) {
        set.delete(callback);
        if (set.size === 0) {
          this.listeners.delete(type);
        }
      }
    };
  }

  /**
   * Subscribe to multiple event types.
   * 
   * Expected Behavior:
   *   Registers callback for all specified types.
   *   Returns unsubscribe function for all.
   * 
   * Args:
   *   types: Array of event types.
   *   callback: Function to call on any event.
   * 
   * Returns:
   *   () => void: Unsubscribe function.
   */
  subscribeMany(types: EventType[], callback: EventCallback): () => void {
    const unsubscribes = types.map(type => this.subscribe(type, callback));
    return () => unsubscribes.forEach(unsub => unsub());
  }

  /**
   * Emit an event to all listeners.
   * 
   * Expected Behavior:
   *   Calls all callbacks for event type.
   *   Adds event to history.
   * 
   * Args:
   *   event: Event object to emit.
   * 
   * Side Effects:
   *   Invokes all registered callbacks.
   *   Modifies history array.
   */
  emit(event: SSEEvent): void {
    // Add to history
    this.history.push(event);
    if (this.history.length > this.maxHistory) {
      this.history.shift();
    }

    // Emit to listeners
    const callbacks = this.listeners.get(event.type);
    if (callbacks) {
      callbacks.forEach(callback => {
        try {
          callback(event);
        } catch (error) {
          console.error(`Error in event listener for ${event.type}:`, error);
        }
      });
    }

    // Also emit to wildcard listeners
    const wildcardCallbacks = this.listeners.get('*');
    if (wildcardCallbacks) {
      wildcardCallbacks.forEach(callback => {
        try {
          callback(event);
        } catch (error) {
          console.error(`Error in wildcard listener:`, error);
        }
      });
    }
  }

  /**
   * Get event history.
   * 
   * Expected Behavior:
   *   Returns last N events.
   *   Optionally filters by type.
   * 
   * Args:
   *   type: Optional event type filter.
   *   limit: Max events to return (default 50).
   * 
   * Returns:
   *   SSEEvent[]: Array of events.
   */
  getHistory(type?: EventType, limit = 50): SSEEvent[] {
    let events = this.history;
    if (type) {
      events = events.filter(e => e.type === type);
    }
    return events.slice(-limit);
  }

  /**
   * Mark connection as connected.
   */
  setConnected(connected: boolean): void {
    this.connected = connected;
  }

  /**
   * Check if connected to SSE stream.
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Get reconnect attempts.
   */
  getReconnectAttempts(): number {
    return this.reconnectAttempts;
  }

  /**
   * Increment reconnect attempts.
   */
  incrementReconnectAttempts(): void {
    this.reconnectAttempts++;
  }

  /**
   * Reset reconnect attempts.
   */
  resetReconnectAttempts(): void {
    this.reconnectAttempts = 0;
  }

  /**
   * Check if should reconnect.
   */
  shouldReconnect(): boolean {
    return this.reconnectAttempts < this.maxReconnectAttempts;
  }

  /**
   * Clear all listeners and history.
   */
  clear(): void {
    this.listeners.clear();
    this.history = [];
  }
}

// Export singleton instance
export const eventBus = new EventBus();
