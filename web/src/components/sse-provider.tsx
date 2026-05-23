"use client";

import { createContext, useContext, ReactNode, useEffect } from "react";
import { useSSE, useEvent } from "@/hooks/useSSE";
import { SSEEvent } from "@/lib/event-bus";
import { toast } from "@/hooks/use-toast";

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

  // Listen for custom "openhack-toast" window events dispatched by SSE
  // notifications and non-SSE code that wants to trigger toasts
  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail
      if (detail?.title || detail?.message) {
        toast({
          title: detail.title,
          description: detail.message,
        })
      }
    }
    window.addEventListener("openhack-toast", handler)
    return () => window.removeEventListener("openhack-toast", handler)
  }, [])

  return (
    <SSEContext.Provider value={{ connected, connecting, error, reconnect }}>
      {children}
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
