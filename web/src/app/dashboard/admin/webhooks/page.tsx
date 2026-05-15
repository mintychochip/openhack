"use client"

import { useState } from "react"
import { Webhook, Plus, Edit, Trash2, Check, X, Activity } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Badge } from "@/components/ui/badge"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface WebhookConfig {
  id: string
  name: string
  url: string
  events: string[]
  is_active: boolean
  secret?: string
  created_at: string
  last_triggered?: string
  success_count: number
  failure_count: number
}

export default function AdminWebhooksPage() {
  const { user } = useAuth()
  const { toast } = useToast()
  const [showForm, setShowForm] = useState(false)
  const [webhooks, setWebhooks] = useState<WebhookConfig[]>([
    {
      id: "1",
      name: "Discord Notifications",
      url: "https://discord.com/api/webhooks/...",
      events: ["user.registered", "project.submitted"],
      is_active: true,
      created_at: new Date().toISOString(),
      success_count: 150,
      failure_count: 2,
    },
  ])

  const [formData, setFormData] = useState({
    name: "",
    url: "",
    events: [] as string[],
  })

  const availableEvents = [
    "user.registered",
    "user.logged_in",
    "team.created",
    "project.submitted",
    "project.updated",
    "score.submitted",
    "vote.cast",
    "email.delivered",
  ]

  if (!user || user.role !== "admin") {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">Admin access required</p>
      </div>
    )
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    toast({
      title: "Webhook created",
      description: "The webhook has been configured",
    })
    setShowForm(false)
    setFormData({ name: "", url: "", events: [] })
  }

  const handleTestWebhook = async (webhookId: string) => {
    toast({
      title: "Webhook test sent",
      description: "Check the delivery logs for results",
    })
  }

  const handleToggle = async (webhookId: string, currentStatus: boolean) => {
    toast({
      title: currentStatus ? "Webhook disabled" : "Webhook enabled",
      description: `The webhook is now ${currentStatus ? "inactive" : "active"}`,
    })
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Webhook Management</h1>
          <p className="text-muted-foreground">Configure outbound webhooks</p>
        </div>
        <Button onClick={() => setShowForm(!showForm)}>
          <Plus className="h-4 w-4 mr-2" />
          {showForm ? "Cancel" : "Create Webhook"}
        </Button>
      </div>

      {showForm && (
        <Card>
          <CardHeader>
            <CardTitle>Create Webhook</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">Webhook Name</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) =>
                    setFormData({ ...formData, name: e.target.value })
                  }
                  placeholder="e.g., Discord Notifications"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="url">Webhook URL</Label>
                <Input
                  id="url"
                  type="url"
                  value={formData.url}
                  onChange={(e) =>
                    setFormData({ ...formData, url: e.target.value })
                  }
                  placeholder="https://example.com/webhook"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label>Events to Subscribe</Label>
                <div className="grid grid-cols-2 gap-2">
                  {availableEvents.map((event) => (
                    <label
                      key={event}
                      className="flex items-center gap-2 p-2 border rounded cursor-pointer hover:bg-secondary"
                    >
                      <input
                        type="checkbox"
                        checked={formData.events.includes(event)}
                        onChange={(e) => {
                          if (e.target.checked) {
                            setFormData({
                              ...formData,
                              events: [...formData.events, event],
                            })
                          } else {
                            setFormData({
                              ...formData,
                              events: formData.events.filter((e) => e !== event),
                            })
                          }
                        }}
                        className="rounded"
                      />
                      <span className="text-sm font-mono">{event}</span>
                    </label>
                  ))}
                </div>
              </div>
              <div className="flex gap-2">
                <Button type="submit">Create Webhook</Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    setShowForm(false)
                    setFormData({ name: "", url: "", events: [] })
                  }}
                >
                  Cancel
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      )}

      {webhooks.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            <Webhook className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No webhooks configured yet</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {webhooks.map((webhook) => (
            <Card key={webhook.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <Webhook className="h-5 w-5 text-muted-foreground" />
                      <CardTitle>{webhook.name}</CardTitle>
                    </div>
                    <CardDescription className="font-mono text-xs break-all">
                      {webhook.url}
                    </CardDescription>
                  </div>
                  <div className="flex items-center gap-2">
                    <Label htmlFor={`webhook-${webhook.id}`} className="text-sm">
                      Active
                    </Label>
                    <Switch
                      id={`webhook-${webhook.id}`}
                      checked={webhook.is_active}
                      onCheckedChange={() => handleToggle(webhook.id, webhook.is_active)}
                    />
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="mb-4">
                  <p className="text-xs text-muted-foreground mb-2">Subscribed Events:</p>
                  <div className="flex flex-wrap gap-1">
                    {webhook.events.map((event) => (
                      <Badge key={event} variant="secondary" className="font-mono text-xs">
                        {event}
                      </Badge>
                    ))}
                  </div>
                </div>
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-4 text-sm">
                    <div className="flex items-center gap-1 text-green-600">
                      <Check className="h-4 w-4" />
                      <span>{webhook.success_count} successful</span>
                    </div>
                    <div className="flex items-center gap-1 text-red-600">
                      <X className="h-4 w-4" />
                      <span>{webhook.failure_count} failed</span>
                    </div>
                    {webhook.last_triggered && (
                      <div className="flex items-center gap-1 text-muted-foreground">
                        <Activity className="h-4 w-4" />
                        <span>Last: {new Date(webhook.last_triggered).toLocaleString()}</span>
                      </div>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <Button size="sm" variant="outline" onClick={() => handleTestWebhook(webhook.id)}>
                      Test
                    </Button>
                    <Button size="sm" variant="outline">
                      <Edit className="h-3 w-3 mr-1" />
                      Edit
                    </Button>
                    <Button size="sm" variant="destructive">
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
