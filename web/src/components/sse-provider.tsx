"use client";

import { createContext, useContext, ReactNode, useState, useEffect, useRef } from "react";
import { useSSE, useEvent } from "@/hooks/useSSE";
import { SSEEvent } from "@/lib/event-bus";

const SSE_ROUTES = ["/dashboard"];

function getSSEUrl(): string {
  if (typeof window === "undefined") return "/api/events/stream";
  const gatewayUrl = process.env.NEXT_PUBLIC_GATEWAY_URL || "";
  if (gatewayUrl) return `${gatewayUrl}/api/events/stream`;
  return "/api/events/stream";
}

function shouldConnectSSE(): boolean {
  if (typeof window === "undefined") return false;
  return SSE_ROUTES.some((r) => window.location.pathname.startsWith(r));
}

interface SSEContextType {
  connected: boolean;
  connecting: boolean;
  error: string | null;
  reconnect: () => void;
}

const SSEContext = createContext<SSEContextType | undefined>(undefined);

export function SSEProvider({ children }: { children: ReactNode }) {
  const { connected, connecting, error, reconnect } = useSSE({
    url: getSSEUrl(),
    autoConnect: shouldConnectSSE(),
  });

  useEvent(
    ["announcement.sent", "project.submitted", "team.created", "leaderboard.updated"],
    (event: SSEEvent) => {
      showToast(event);
    }
  );

  return (
    <SSEContext.Provider value={{ connected, connecting, error, reconnect }}>
      {children}
      {shouldConnectSSE() && (
        <ConnectionIndicator connected={connected} connecting={connecting} error={error} />
      )}
    </SSEContext.Provider>
  );
}

export function useSSEContext(): SSEContextType {
  const context = useContext(SSEContext);
  if (context === undefined) {
    throw new Error("useSSEContext must be used within SSEProvider");
  }
  return context;
}

function ConnectionIndicator({ connected, connecting, error }: { connected: boolean; connecting: boolean; error: string | null }) {
  const [displayState, setDisplayState] = useState<"connected" | "connecting" | "error" | "idle">("idle");
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }

    if (connected) {
      timerRef.current = setTimeout(() => setDisplayState("connected"), 1000);
    } else if (error) {
      timerRef.current = setTimeout(() => setDisplayState("error"), 3000);
    } else if (connecting) {
      timerRef.current = setTimeout(() => setDisplayState("connecting"), 2000);
    } else {
      setDisplayState("idle");
    }

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [connected, connecting, error]);

  if (displayState === "error") {
    return (
      <div className="fixed bottom-4 right-4 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-200 px-4 py-2 rounded-lg shadow-lg flex items-center gap-2">
        <span className="text-lg">&#x274C;</span>
        <span>Connection lost</span>
      </div>
    );
  }

  if (displayState === "connecting") {
    return (
      <div className="fixed bottom-4 right-4 bg-yellow-100 dark:bg-yellow-900 border border-yellow-400 dark:border-yellow-700 text-yellow-700 dark:text-yellow-200 px-4 py-2 rounded-lg shadow-lg flex items-center gap-2">
        <span className="text-lg">&#x1F4E1;</span>
        <span>Connecting...</span>
      </div>
    );
  }

  if (displayState === "connected") {
    return (
      <div className="fixed bottom-4 right-4 bg-green-100 dark:bg-green-900 border border-green-400 dark:border-green-700 text-green-700 dark:text-green-200 px-4 py-2 rounded-lg shadow-lg flex items-center gap-2">
        <span className="text-lg">&#x1F4E1;</span>
        <span>Live</span>
      </div>
    );
  }

  return null;
}

function showToast(event: SSEEvent): void {
  let title = "";
  let message = "";

  switch (event.type) {
    case "announcement.sent":
      title = "New Announcement";
      message = event.data.title || "New announcement received";
      break;
    case "project.submitted":
      title = "Project Submitted";
      message = `${event.data.team_name || "A team"} just submitted their project`;
      break;
    case "team.created":
      title = "New Team";
      message = `${event.data.team_name || "A new team"} has been created`;
      break;
    case "leaderboard.updated":
      title = "Leaderboard Updated";
      message = "Rankings have been updated";
      break;
    default:
      return;
  }

  window.dispatchEvent(new CustomEvent("openhack-toast", {
    detail: { title, message },
  }));
}
