"use client"

import { useState, useEffect } from "react"
import { Settings as SettingsIcon, Save, Bot } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Switch } from "@/components/ui/switch"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"
import { api } from "@/lib/api"

export default function AdminHackathonPage() {
  const { user } = useAuth()
  const { toast } = useToast()
  const [isLoading, setIsLoading] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [openaiKeyModified, setOpenaiKeyModified] = useState(false)
  const [anthropicKeyModified, setAnthropicKeyModified] = useState(false)

  const [formData, setFormData] = useState({
    name: "OpenHack",
    tagline: "Build something amazing",
    description: "",
    start_date: "",
    end_date: "",
    timezone: "UTC",
    max_teams: "100",
    max_team_size: "4",
    registration_open: true,
    submission_open: true,
    voting_open: true,
    ai_provider: "openai",
    ai_enabled: false,
    openai_api_key: "",
    openai_base_url: "https://api.openai.com/v1",
    openai_model: "gpt-4-turbo",
    anthropic_api_key: "",
    anthropic_model: "claude-3-5-sonnet-20241022",
  })

  useEffect(() => {
    if (!user || !user.roles.includes("admin")) return

    const load = async () => {
      setIsLoading(true)
      try {
        const config = await api.get("/api/core/info")
        setFormData((prev) => ({
          ...prev,
          name: config.name || prev.name,
          tagline: config.tagline || prev.tagline,
          description: config.description || "",
          timezone: config.timezone || "UTC",
          registration_open: config.registration_open ?? true,
          ai_provider: config.ai_provider || "openai",
          ai_enabled: config.ai_enabled ?? false,
          openai_api_key: config.openai_api_key || "",
          openai_base_url: config.openai_base_url || "https://api.openai.com/v1",
          openai_model: config.openai_model || "gpt-4-turbo",
          anthropic_api_key: config.anthropic_api_key || "",
          anthropic_model: config.anthropic_model || "claude-3-5-sonnet-20241022",
        }))
      } catch (err: any) {
        toast({
          title: "Failed to load config",
          description: err.message || "Could not fetch hackathon settings",
          variant: "destructive",
        })
      } finally {
        setIsLoading(false)
      }
    }

    load()
  }, [user, toast])

  if (!user || !user.roles.includes("admin")) {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">Admin access required</p>
      </div>
    )
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsSaving(true)

    try {
      const payload: Record<string, any> = {
        name: formData.name,
        tagline: formData.tagline,
        timezone: formData.timezone,
        registration_open: formData.registration_open,
        ai_provider: formData.ai_provider,
        ai_enabled: formData.ai_enabled,
        openai_base_url: formData.openai_base_url,
        openai_model: formData.openai_model,
        anthropic_model: formData.anthropic_model,
      }

      // Only send API keys if the user actually typed a new value
      // (the backend returns masked keys like "***xxxx")
      if (openaiKeyModified && formData.openai_api_key && !formData.openai_api_key.startsWith("***")) {
        payload.openai_api_key = formData.openai_api_key
      }
      if (anthropicKeyModified && formData.anthropic_api_key && !formData.anthropic_api_key.startsWith("***")) {
        payload.anthropic_api_key = formData.anthropic_api_key
      }

      await api.put("/api/core/info", payload)
      toast({
        title: "Configuration saved",
        description: "Hackathon settings have been updated",
      })
      setOpenaiKeyModified(false)
      setAnthropicKeyModified(false)
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to save configuration",
        variant: "destructive",
      })
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Hackathon Configuration</h1>
        <p className="text-muted-foreground">Global settings for your hackathon</p>
      </div>

      {isLoading && (
        <div className="flex items-center gap-2 text-muted-foreground">
          <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary" />
          Loading configuration...
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-6">
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <SettingsIcon className="h-5 w-5" />
              <CardTitle>Basic Information</CardTitle>
            </div>
            <CardDescription>
              Core details about your hackathon event
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="name">Hackathon Name</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                placeholder="e.g., OpenHack 2026"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="tagline">Tagline</Label>
              <Input
                id="tagline"
                value={formData.tagline}
                onChange={(e) =>
                  setFormData({ ...formData, tagline: e.target.value })
                }
                placeholder="e.g., Build Something Amazing"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={formData.description}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
                placeholder="Describe your hackathon..."
                rows={4}
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Dates & Schedule</CardTitle>
            <CardDescription>
              When your hackathon takes place
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="start_date">Start Date</Label>
                <Input
                  id="start_date"
                  type="date"
                  value={formData.start_date}
                  onChange={(e) =>
                    setFormData({ ...formData, start_date: e.target.value })
                  }
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="end_date">End Date</Label>
                <Input
                  id="end_date"
                  type="date"
                  value={formData.end_date}
                  onChange={(e) =>
                    setFormData({ ...formData, end_date: e.target.value })
                  }
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="timezone">Timezone</Label>
              <select
                id="timezone"
                value={formData.timezone}
                onChange={(e) =>
                  setFormData({ ...formData, timezone: e.target.value })
                }
                className="w-full rounded-md border border-input bg-background px-3 py-2"
              >
                <option value="UTC">UTC</option>
                <option value="America/New_York">Eastern Time</option>
                <option value="America/Chicago">Central Time</option>
                <option value="America/Denver">Mountain Time</option>
                <option value="America/Los_Angeles">Pacific Time</option>
                <option value="Europe/London">London (GMT)</option>
                <option value="Europe/Paris">Paris (CET)</option>
                <option value="Asia/Tokyo">Tokyo (JST)</option>
              </select>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Capacity & Limits</CardTitle>
            <CardDescription>
              Maximum participants and team sizes
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="max_teams">Max Teams</Label>
                <Input
                  id="max_teams"
                  type="number"
                  value={formData.max_teams}
                  onChange={(e) =>
                    setFormData({ ...formData, max_teams: e.target.value })
                  }
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="max_team_size">Max Team Size</Label>
                <Input
                  id="max_team_size"
                  type="number"
                  value={formData.max_team_size}
                  onChange={(e) =>
                    setFormData({ ...formData, max_team_size: e.target.value })
                  }
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Feature Toggles</CardTitle>
            <CardDescription>
              Enable or disable hackathon features
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="registration_open">Registration</Label>
                <p className="text-sm text-muted-foreground">
                  Allow new user registrations
                </p>
              </div>
              <Switch
                id="registration_open"
                checked={formData.registration_open}
                onCheckedChange={(checked) =>
                  setFormData({ ...formData, registration_open: checked })
                }
              />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="submission_open">Project Submissions</Label>
                <p className="text-sm text-muted-foreground">
                  Allow teams to submit projects
                </p>
              </div>
              <Switch
                id="submission_open"
                checked={formData.submission_open}
                onCheckedChange={(checked) =>
                  setFormData({ ...formData, submission_open: checked })
                }
              />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="voting_open">Public Voting</Label>
                <p className="text-sm text-muted-foreground">
                  Allow public to vote on projects
                </p>
              </div>
              <Switch
                id="voting_open"
                checked={formData.voting_open}
                onCheckedChange={(checked) =>
                  setFormData({ ...formData, voting_open: checked })
                }
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Bot className="h-5 w-5" />
              <CardTitle>AI Provider</CardTitle>
            </div>
            <CardDescription>
              Configure LLM for AI assistant, embeddings, and idea generation
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="ai_enabled">Enable AI</Label>
                <p className="text-sm text-muted-foreground">
                  Turn on AI-powered features across the platform
                </p>
              </div>
              <Switch
                id="ai_enabled"
                checked={formData.ai_enabled}
                onCheckedChange={(checked) =>
                  setFormData({ ...formData, ai_enabled: checked })
                }
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="ai_provider">Provider</Label>
              <select
                id="ai_provider"
                value={formData.ai_provider}
                onChange={(e) =>
                  setFormData({ ...formData, ai_provider: e.target.value })
                }
                className="w-full rounded-md border border-input bg-background px-3 py-2"
              >
                <option value="openai">OpenAI / OpenAI-compatible</option>
                <option value="anthropic">Anthropic (Claude)</option>
              </select>
            </div>

            {formData.ai_provider === "openai" && (
              <>
                <div className="space-y-2">
                  <Label htmlFor="openai_base_url">Base URL</Label>
                  <Input
                    id="openai_base_url"
                    value={formData.openai_base_url}
                    onChange={(e) =>
                      setFormData({ ...formData, openai_base_url: e.target.value })
                    }
                    placeholder="https://api.openai.com/v1"
                  />
                  <p className="text-xs text-muted-foreground">
                    For Ollama use http://localhost:11434/v1. For vLLM or LM Studio, use their local URL.
                  </p>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="openai_api_key">API Key</Label>
                  <Input
                    id="openai_api_key"
                    type="password"
                    value={formData.openai_api_key}
                    onChange={(e) => {
                      setFormData({ ...formData, openai_api_key: e.target.value })
                      setOpenaiKeyModified(true)
                    }}
                    placeholder={formData.openai_api_key.startsWith("***") ? "Key is set (enter new to change)" : "sk-..."}
                  />
                  <p className="text-xs text-muted-foreground">
                    Leave blank to keep existing key. The key is stored securely in the database.
                  </p>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="openai_model">Model</Label>
                  <Input
                    id="openai_model"
                    value={formData.openai_model}
                    onChange={(e) =>
                      setFormData({ ...formData, openai_model: e.target.value })
                    }
                    placeholder="gpt-4-turbo"
                  />
                </div>
              </>
            )}

            {formData.ai_provider === "anthropic" && (
              <>
                <div className="space-y-2">
                  <Label htmlFor="anthropic_api_key">API Key</Label>
                  <Input
                    id="anthropic_api_key"
                    type="password"
                    value={formData.anthropic_api_key}
                    onChange={(e) => {
                      setFormData({ ...formData, anthropic_api_key: e.target.value })
                      setAnthropicKeyModified(true)
                    }}
                    placeholder={formData.anthropic_api_key.startsWith("***") ? "Key is set (enter new to change)" : "sk-ant-..."}
                  />
                  <p className="text-xs text-muted-foreground">
                    Leave blank to keep existing key.
                  </p>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="anthropic_model">Model</Label>
                  <Input
                    id="anthropic_model"
                    value={formData.anthropic_model}
                    onChange={(e) =>
                      setFormData({ ...formData, anthropic_model: e.target.value })
                    }
                    placeholder="claude-3-5-sonnet-20241022"
                  />
                </div>
              </>
            )}
          </CardContent>
        </Card>

        <div className="flex gap-2">
          <Button type="submit" disabled={isSaving}>
            {isSaving ? (
              <>
                <Save className="h-4 w-4 mr-2 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="h-4 w-4 mr-2" />
                Save Configuration
              </>
            )}
          </Button>
        </div>
      </form>
    </div>
  )
}
