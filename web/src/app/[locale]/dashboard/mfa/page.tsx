"use client"

import { useState, useEffect } from "react"
import Link from "next/link"
import { useRouter } from "next/navigation"
import { Shield, Smartphone, Key } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { api } from "@/lib/api"
import { useAuth } from "@/contexts/auth-context"
import { useToast } from "@/hooks/use-toast"

export default function MFASetupPage() {
  const { user, updateUser } = useAuth()
  const router = useRouter()
  const { toast } = useToast()
  const [step, setStep] = useState<"setup" | "verify">("setup")
  const [isLoading, setIsLoading] = useState(false)
  const [totpUri, setTotpUri] = useState("")
  const [secret, setSecret] = useState("")
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([])
  const [verificationCode, setVerificationCode] = useState("")

  const startSetup = async () => {
    setIsLoading(true)
    try {
      const response = await fetch("/api/auth/mfa/totp/generate", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${api.getToken()}`,
        },
      })

      if (!response.ok) {
        throw new Error("Failed to generate TOTP")
      }

      const data = await response.json()
      setTotpUri(data.totp_uri)
      setSecret(data.secret)
      setStep("verify")
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to start MFA setup",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const verifyCode = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)

    try {
      const response = await fetch("/api/auth/mfa/totp/verify", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${api.getToken()}`,
        },
        body: JSON.stringify({ code: verificationCode }),
      })

      if (!response.ok) {
        throw new Error("Invalid code")
      }

      const data = await response.json()
      setRecoveryCodes(data.recovery_codes || [])
      updateUser({ mfaEnabled: true })
      toast({
        title: "MFA Enabled",
        description: "Two-factor authentication has been set up",
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Invalid verification code",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const downloadRecoveryCodes = () => {
    const content = `OpenHack Recovery Codes\n\n${recoveryCodes.join("\n")}\n\nKeep these codes safe. Each code can only be used once.`
    const blob = new Blob([content], { type: "text/plain" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = "recovery-codes.txt"
    a.click()
    URL.revokeObjectURL(url)
  }

  if (user?.mfaEnabled) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold">Two-Factor Authentication</h1>
          <p className="text-muted-foreground">MFA is already enabled on your account</p>
        </div>
        <Card>
          <CardHeader>
            <CardTitle>MFA Status</CardTitle>
            <CardDescription>
              Your account is protected with two-factor authentication
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Alert>
              <Shield className="h-4 w-4" />
              <AlertDescription>
                MFA is active. You&apos;ll need to enter a code from your authenticator app when logging in.
              </AlertDescription>
            </Alert>
            <Button className="mt-4" variant="destructive">
              Disable MFA
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Two-Factor Authentication</h1>
        <p className="text-muted-foreground">Secure your account with MFA</p>
      </div>

      {step === "setup" && (
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              <CardTitle>Enable MFA</CardTitle>
            </div>
            <CardDescription>
              Set up two-factor authentication with an authenticator app
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Alert>
              <Smartphone className="h-4 w-4" />
              <AlertDescription>
                Install an authenticator app like Google Authenticator, Authy, or Microsoft Authenticator
              </AlertDescription>
            </Alert>
            <Button onClick={startSetup} disabled={isLoading} className="w-full">
              {isLoading ? "Generating..." : "Start Setup"}
            </Button>
          </CardContent>
        </Card>
      )}

      {step === "verify" && totpUri && (
        <Card>
          <CardHeader>
            <CardTitle>Scan QR Code</CardTitle>
            <CardDescription>
              Scan this QR code with your authenticator app
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-center">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={`https://api.qrserver.com/v1/create-qr-code/?data=${encodeURIComponent(totpUri)}&size=200x200`}
                alt="TOTP QR Code"
                className="border rounded-lg"
              />
            </div>
            <div className="text-center">
              <p className="text-sm text-muted-foreground mb-2">Or enter this code manually:</p>
              <code className="bg-muted px-3 py-2 rounded text-sm">{secret}</code>
            </div>
            <form onSubmit={verifyCode} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="code">Verification Code</Label>
                <Input
                  id="code"
                  value={verificationCode}
                  onChange={(e) => setVerificationCode(e.target.value)}
                  placeholder="Enter 6-digit code"
                  maxLength={6}
                  required
                />
              </div>
              <Button type="submit" disabled={isLoading} className="w-full">
                {isLoading ? "Verifying..." : "Verify & Enable"}
              </Button>
            </form>
          </CardContent>
        </Card>
      )}

      {recoveryCodes.length > 0 && (
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Key className="h-5 w-5" />
              <CardTitle>Recovery Codes</CardTitle>
            </div>
            <CardDescription>
              Save these codes in a safe place. Each code can only be used once.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-2">
              {recoveryCodes.map((code) => (
                <code key={code} className="bg-muted px-3 py-2 rounded text-sm">
                  {code}
                </code>
              ))}
            </div>
            <Button onClick={downloadRecoveryCodes} variant="outline" className="w-full">
              Download Recovery Codes
            </Button>
            <Link href="/dashboard">
              <Button className="w-full">Done</Button>
            </Link>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
