"use client"

import { Suspense, useEffect, useState } from "react"
import { useSearchParams } from "next/navigation"
import { api } from "@/lib/api"

/**
 * Page for confirming Discord-to-OpenHack account linking.
 *
 * Expected Behavior:
 *   Receives a `?token=xxx` query parameter from a Discord bot DM.
 *   On mount, reads the token, determines the current user (from localStorage JWT),
 *   and calls the confirm endpoint. Displays success or error status.
 *   If no token is present, shows an error. If the token is expired/invalid,
 *   shows a specific error message.
 */
function LinkDiscordContent() {
  const searchParams = useSearchParams()
  const token = searchParams.get("token")
  const [status, setStatus] = useState<"loading" | "success" | "error">("loading")
  const [message, setMessage] = useState("")

  useEffect(() => {
    if (!token) {
      setStatus("error")
      setMessage("No link token provided. Please use the link from your Discord DM.")
      return
    }

    const authUserId = typeof window !== "undefined"
      ? localStorage.getItem("user_id")
      : null

    if (!authUserId) {
      setStatus("error")
      setMessage("You need to be logged in to link your Discord account.")
      return
    }

    api.discordConfirmLink(token, authUserId)
      .then(() => {
        setStatus("success")
        setMessage("Your Discord account has been linked successfully! You can now use all OpenHack commands from Discord.")
      })
      .catch((err) => {
        setStatus("error")
        if (err.status === 410) {
          setMessage("This link token has expired. Please run `/openhack register` again in Discord.")
        } else if (err.status === 400) {
          setMessage(err.message || "This OpenHack account is already linked to a Discord account.")
        } else {
          setMessage(`Failed to link account: ${err.message}`)
        }
      })
  }, [token])

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="max-w-md w-full p-8 bg-white rounded-lg shadow-lg text-center">
        <h1 className="text-2xl font-bold mb-4">Link Discord Account</h1>

        {status === "loading" && (
          <div className="flex items-center justify-center space-x-2">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600" />
            <p className="text-gray-600">Linking your accounts...</p>
          </div>
        )}

        {status === "success" && (
          <div>
            <div className="text-4xl mb-4">&#10003;</div>
            <p className="text-green-600 font-medium">{message}</p>
            <a
              href="/dashboard"
              className="mt-6 inline-block px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700"
            >
              Go to Dashboard
            </a>
          </div>
        )}

        {status === "error" && (
          <div>
            <div className="text-4xl mb-4">&#10007;</div>
            <p className="text-red-600 font-medium">{message}</p>
            <a
              href="/dashboard"
              className="mt-6 inline-block px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
            >
              Back to Dashboard
            </a>
          </div>
        )}
      </div>
    </div>
  )
}

export default function LinkDiscordPage() {
  return (
    <Suspense fallback={<div className="min-h-screen flex items-center justify-center"><p className="text-gray-600">Loading...</p></div>}>
      <LinkDiscordContent />
    </Suspense>
  )
}
