"use client"

import { useEffect } from "react"

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  useEffect(() => {
    console.error("Global error:", error)
  }, [error])

  return (
    <html>
      <body>
        <div className="flex min-h-screen flex-col items-center justify-center bg-background p-4">
          <div className="w-full max-w-md space-y-6 text-center">
            <h1 className="text-3xl font-bold tracking-tight text-destructive">
              Something went wrong
            </h1>
            <p className="text-muted-foreground">
              An unexpected error occurred. Try refreshing the page or going back home.
            </p>
            {error.digest && (
              <p className="text-xs text-muted-foreground font-mono">
                Error digest: {error.digest}
              </p>
            )}
            <div className="flex gap-4 justify-center">
              <button
                onClick={reset}
                className="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                Try Again
              </button>
              <a
                href="/"
                className="inline-flex items-center justify-center rounded-md border border-input bg-background px-4 py-2 text-sm font-medium hover:bg-accent hover:text-accent-foreground"
              >
                Go Home
              </a>
            </div>
          </div>
        </div>
      </body>
    </html>
  )
}
