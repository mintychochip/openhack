import type { Metadata } from "next"
import { Inter } from "next/font/google"
import "./globals.css"
import { AuthProvider } from "@/contexts/auth-context"
import { SSEProvider } from "@/components/sse-provider"
import { Toaster } from "@/components/toaster"

const inter = Inter({ subsets: ["latin"] })

export const metadata: Metadata = {
  title: "OpenHack - Self-Hosted Hackathon Platform",
  description: "Manage your hackathon with ease - teams, projects, judging, and leaderboards",
  keywords: ["hackathon", "platform", "teams", "judging", "open source"],
  authors: [{ name: "OpenHack Team" }],
  openGraph: {
    title: "OpenHack",
    description: "Self-Hosted Hackathon Platform",
    type: "website",
  },
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={inter.className}>
        <AuthProvider>
          <SSEProvider>
            <a href="#main-content" className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-blue-600 focus:text-white focus:rounded">
              Skip to main content
            </a>
            {children}
            <Toaster />
          </SSEProvider>
        </AuthProvider>
      </body>
    </html>
  )
}
