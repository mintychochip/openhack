"use client"

import { useState, useEffect } from "react"
import { useRouter } from "next/navigation"
import { User, Shield, Key, Upload, Trash2 } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { api, ApiError } from "@/lib/api"
import { useAuth } from "@/contexts/auth-context"
import { useToast } from "@/hooks/use-toast"

export default function SettingsPage() {
  const { user, updateUser } = useAuth()
  const router = useRouter()
  const { toast } = useToast()
  const [isLoading, setIsLoading] = useState(false)
  const [name, setName] = useState("")
  const [githubUsername, setGithubUsername] = useState("")
  const [avatarFile, setAvatarFile] = useState<File | null>(null)
  const [avatarPreview, setAvatarPreview] = useState<string | null>(null)

  useEffect(() => {
    if (user) {
      setName(user.name || "")
      setGithubUsername(user.githubUsername || "")
      setAvatarPreview(user.avatarUrl || null)
    }
  }, [user])

  const handleProfileUpdate = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)

    try {
      const updated = await api.updateMe({ name, githubUsername })
      updateUser(updated)
      toast({
        title: "Profile updated",
        description: "Your profile has been saved",
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to update profile",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleAvatarUpload = async () => {
    if (!avatarFile) return

    setIsLoading(true)
    try {
      const result = await api.uploadAvatar(avatarFile)
      updateUser({ avatarUrl: result.url })
      setAvatarPreview(result.url)
      toast({
        title: "Avatar uploaded",
        description: "Your avatar has been updated",
      })
      setAvatarFile(null)
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to upload avatar",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleAvatarDelete = async () => {
    setIsLoading(true)
    try {
      await api.deleteAvatar()
      updateUser({ avatarUrl: undefined })
      setAvatarPreview(null)
      toast({
        title: "Avatar removed",
        description: "Your avatar has been deleted",
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to delete avatar",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Settings</h1>
        <p className="text-muted-foreground">Manage your account settings</p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <User className="h-5 w-5" />
            <CardTitle>Profile</CardTitle>
          </div>
          <CardDescription>Update your personal information</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleProfileUpdate} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Your name"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="github">GitHub Username</Label>
              <Input
                id="github"
                value={githubUsername}
                onChange={(e) => setGithubUsername(e.target.value)}
                placeholder="username"
              />
            </div>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? "Saving..." : "Save Changes"}
            </Button>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            <CardTitle>Avatar</CardTitle>
          </div>
          <CardDescription>Upload a profile picture</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-4">
            {avatarPreview ? (
              <img
                src={avatarPreview}
                alt="Avatar"
                className="w-20 h-20 rounded-full"
              />
            ) : (
              <div className="w-20 h-20 rounded-full bg-primary flex items-center justify-center text-white text-xl font-bold">
                {user?.name?.charAt(0).toUpperCase() || "U"}
              </div>
            )}
            <div className="flex-1">
              <Input
                type="file"
                accept="image/*"
                onChange={(e) => setAvatarFile(e.target.files?.[0] || null)}
                className="mb-2"
              />
              <div className="flex gap-2">
                <Button
                  onClick={handleAvatarUpload}
                  disabled={!avatarFile || isLoading}
                  size="sm"
                >
                  Upload
                </Button>
                {avatarPreview && (
                  <Button
                    onClick={handleAvatarDelete}
                    variant="destructive"
                    size="sm"
                    disabled={isLoading}
                  >
                    <Trash2 className="h-4 w-4 mr-2" />
                    Remove
                  </Button>
                )}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            <CardTitle>Security</CardTitle>
          </div>
          <CardDescription>Manage your security settings</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div>
              <p className="font-medium">Two-Factor Authentication</p>
              <p className="text-sm text-muted-foreground">
                {user?.mfaEnabled ? "Enabled" : "Disabled"}
              </p>
            </div>
            <Button variant={user?.mfaEnabled ? "destructive" : "default"}>
              {user?.mfaEnabled ? "Disable" : "Enable"}
            </Button>
          </div>
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div>
              <p className="font-medium">Active Sessions</p>
              <p className="text-sm text-muted-foreground">
                Manage your logged-in devices
              </p>
            </div>
            <Button variant="outline">View Sessions</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
