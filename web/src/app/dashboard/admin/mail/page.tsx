"use client"

import { useState } from "react"
import { Mail, Plus, Edit, Trash2, Eye, Send, Code } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface Template {
  id: string
  name: string
  subject: string
  body: string
  variables: string[]
  is_active: boolean
}

export default function AdminMailTemplatesPage() {
  const { user } = useAuth()
  const { toast } = useToast()
  const [activeTab, setActiveTab] = useState<"list" | "edit">("list")
  const [editingTemplate, setEditingTemplate] = useState<Template | null>(null)
  const [templates] = useState<Template[]>([
    {
      id: "1",
      name: "Welcome Email",
      subject: "Welcome to OpenHack!",
      body: "Hi {{name}},\n\nWelcome to OpenHack! We're excited to have you join us...",
      variables: ["name", "email"],
      is_active: true,
    },
    {
      id: "2",
      name: "Password Reset",
      subject: "Reset Your Password",
      body: "Hi {{name}},\n\nClick the link below to reset your password:\n{{reset_url}}...",
      variables: ["name", "reset_url"],
      is_active: true,
    },
    {
      id: "3",
      name: "Project Submission Confirmation",
      subject: "Project Submitted Successfully",
      body: "Hi {{name}},\n\nYour project {{project_title}} has been submitted...",
      variables: ["name", "project_title", "team_name"],
      is_active: true,
    },
  ])

  const [formData, setFormData] = useState({
    name: "",
    subject: "",
    body: "",
  })

  if (!user || user.role !== "admin") {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">Admin access required</p>
      </div>
    )
  }

  const handleEdit = (template: Template) => {
    setEditingTemplate(template)
    setFormData({
      name: template.name,
      subject: template.subject,
      body: template.body,
    })
    setActiveTab("edit")
  }

  const handleCreate = () => {
    setEditingTemplate(null)
    setFormData({ name: "", subject: "", body: "" })
    setActiveTab("edit")
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault()
    toast({
      title: "Template saved",
      description: "The email template has been saved",
    })
    setActiveTab("list")
  }

  const handleTestSend = async () => {
    toast({
      title: "Test email sent",
      description: "Check your inbox for the test email",
    })
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Email Templates</h1>
          <p className="text-muted-foreground">Manage automated email templates</p>
        </div>
        {activeTab === "list" && (
          <Button onClick={handleCreate}>
            <Plus className="h-4 w-4 mr-2" />
            Create Template
          </Button>
        )}
      </div>

      {activeTab === "list" ? (
        <div className="space-y-4">
          {templates.map((template) => (
            <Card key={template.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <Mail className="h-5 w-5 text-muted-foreground" />
                      <CardTitle>{template.name}</CardTitle>
                    </div>
                    <CardDescription className="font-mono text-xs">
                      Subject: {template.subject}
                    </CardDescription>
                  </div>
                  <span
                    className={`text-xs px-2 py-1 rounded ${
                      template.is_active
                        ? "bg-green-100 text-green-800"
                        : "bg-gray-100 text-gray-800"
                    }`}
                  >
                    {template.is_active ? "Active" : "Inactive"}
                  </span>
                </div>
              </CardHeader>
              <CardContent>
                <div className="mb-4">
                  <p className="text-xs text-muted-foreground mb-2">Variables:</p>
                  <div className="flex flex-wrap gap-1">
                    {template.variables.map((variable) => (
                      <span
                        key={variable}
                        className="text-xs bg-secondary px-2 py-1 rounded font-mono"
                      >
                        {`{{${variable}}}`}
                      </span>
                    ))}
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <pre className="text-xs bg-muted p-3 rounded max-w-2xl line-clamp-3">
                    {template.body}
                  </pre>
                  <div className="flex gap-2">
                    <Button size="sm" variant="outline" onClick={handleTestSend}>
                      <Send className="h-3 w-3 mr-1" />
                      Test
                    </Button>
                    <Button size="sm" variant="outline" onClick={() => handleEdit(template)}>
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
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>
              {editingTemplate ? "Edit Template" : "Create Template"}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSave} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">Template Name</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) =>
                    setFormData({ ...formData, name: e.target.value })
                  }
                  placeholder="e.g., Welcome Email"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="subject">Email Subject</Label>
                <Input
                  id="subject"
                  value={formData.subject}
                  onChange={(e) =>
                    setFormData({ ...formData, subject: e.target.value })
                  }
                  placeholder="e.g., Welcome to OpenHack!"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="body">Email Body (HTML supported)</Label>
                <Textarea
                  id="body"
                  value={formData.body}
                  onChange={(e) =>
                    setFormData({ ...formData, body: e.target.value })
                  }
                  placeholder="Hi {{name}},\n\nWelcome to OpenHack!..."
                  rows={12}
                  className="font-mono text-sm"
                  required
                />
              </div>
              <div className="p-4 bg-secondary rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <Code className="h-4 w-4" />
                  <p className="text-sm font-medium">Available Variables</p>
                </div>
                <div className="flex flex-wrap gap-1">
                  {["name", "email", "reset_url", "project_title", "team_name", "event_name"].map(
                    (variable) => (
                      <button
                        key={variable}
                        type="button"
                        onClick={() =>
                          setFormData({ ...formData, body: formData.body + `{{${variable}}}` })
                        }
                        className="text-xs bg-background border px-2 py-1 rounded font-mono hover:bg-accent"
                      >
                        {`{{${variable}}}`}
                      </button>
                    )
                  )}
                </div>
              </div>
              <div className="flex gap-2">
                <Button type="submit">Save Template</Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setActiveTab("list")}
                >
                  Cancel
                </Button>
                <Button type="button" variant="outline" onClick={handleTestSend}>
                  <Send className="h-3 w-3 mr-1" />
                  Send Test
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
